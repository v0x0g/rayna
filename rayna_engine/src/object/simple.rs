use crate::material::Material;
use crate::mesh::Mesh as MeshTrait;
use crate::object::Object;
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;
use crate::shared::transform_utils::{transform_incoming_ray, transform_outgoing_intersection};
use getset::Getters;
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Transform3};
use smallvec::SmallVec;

/// The main struct that encapsulates all the different "components" that make up an mesh
///
/// Very similar to a `GameObject` in a game engine, where the `ObjectInstance` and `Material` are components attached
/// to that mesh.
///
/// # Important Note
/// If using a rotating/scaling transform, ensure that the mesh you are transforming is positioned
/// at the origin (`[0., 0., 0.]`), and use the transform matrix to do the translation.
///
/// Otherwise, the centre of the mesh will be rotated/scaled around the origin as well, which will move the mesh.
///
/// Alternatively, you can also apply a post and pre-transform, to counteract the mesh's position offset:
/// ```
/// # use rayna_engine::material::lambertian::LambertianMaterial;
/// # use rayna_engine::mesh::axis_box::{AxisBoxBuilder, AxisBoxMesh};
/// # use rayna_shared::def::types::{Angle, Point3, Transform3, Vector3};
/// #
/// # let a: Point3 = [5., 1., 2.].into();
/// # let b: Point3 = [3., 4., -7.].into();
/// # let mesh: AxisBoxMesh = AxisBoxBuilder {
/// #     corner_1: a,
/// #     corner_2: b,
/// # }.into();
///
/// let transform = Transform3::from_axis_angle(Vector3::Y, Angle::from_degrees(69.0));
///
/// // Fix the transform so it scales/rotates around the mesh's centre and not the origin
/// //  1. Move centre to origin
/// //  2. Apply rotate/scale, while it is centred at origin
/// //  3. Move centre back to original position
/// let transform = Transform3::from_translation(-mesh.centre().to_vector())
///     .then(transform)
///     .then_translate(mesh.centre().to_vector());
/// ```
///
/// This pre/post transform is encapsulated in [SimpleObject::new_with_correction()]
#[derive(Getters, Clone, Debug)]
#[get = "pub"]
pub struct SimpleObject<Mesh, Mat>
where
    Mesh: MeshTrait,
    Mat: Material,
{
    object: Mesh,
    material: Mat,
    transform: Option<Transform3>,
    inv_transform: Option<Transform3>,
    #[get(skip)]
    aabb: Option<Aabb>,
    // TODO: Add a string identifier to this (name?)
}

impl<Mesh, Mat> Object for SimpleObject<Mesh, Mat>
where
    Mesh: MeshTrait,
    Mat: Material,
{
    type Mesh = Mesh;
    type Mat = Mat;

    fn full_intersect<'o>(
        &'o self,
        orig_ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Mat>> {
        if let (Some(transform), Some(inv_transform)) = (&self.transform, &self.inv_transform) {
            let trans_ray = transform_incoming_ray(orig_ray, inv_transform);
            let inner = self.object.intersect(&trans_ray, bounds, rng)?;
            let intersect = transform_outgoing_intersection(orig_ray, inner, transform);
            Some(intersect.make_full(&self.material))
        } else {
            Some(self.object.intersect(orig_ray, bounds, rng)?.make_full(&self.material))
        }
    }

    fn full_intersect_all<'o>(
        &'o self,
        orig_ray: &Ray,
        output: &mut SmallVec<[FullIntersection<'o, Mat>; 32]>,
        rng: &mut dyn RngCore,
    ) {
        if let (Some(transform), Some(inv_transform)) = (&self.transform, &self.inv_transform) {
            let trans_ray = transform_incoming_ray(orig_ray, inv_transform);
            let mut inner_intersects = SmallVec::new();
            self.object.intersect_all(&trans_ray, &mut inner_intersects, rng);

            output.extend(inner_intersects.into_iter().map(|mut inner| {
                inner = transform_outgoing_intersection(orig_ray, inner, transform);
                inner.make_full(&self.material)
            }));
        } else {
            let mut inner_intersects = SmallVec::new();
            self.object.intersect_all(&orig_ray, &mut inner_intersects, rng);

            output.extend(
                inner_intersects
                    .into_iter()
                    .map(|inner| inner.make_full(&self.material)),
            );
        }
    }
}

impl<Mesh, Mat> HasAabb for SimpleObject<Mesh, Mat>
where
    Mesh: MeshTrait,
    Mat: Material,
{
    fn aabb(&self) -> Option<&Aabb> { self.aabb.as_ref() }

    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info (shadow checks)
}

// region Constructors

impl<Mesh, Mat> SimpleObject<Mesh, Mat>
where
    Mesh: MeshTrait,
    Mat: Material,
{
    /// Creates a new transformed mesh instance, using the given mesh and transform matrix.
    ///
    /// Unlike [Self::new()], this *does* account for the mesh's translation from the origin,
    /// using the `obj_centre` parameter. See field documentation ([Self::transform]) for explanation
    /// and example of this position offset correction
    pub fn new_with_correction(object: impl Into<Mesh>, material: impl Into<Mat>, transform: Transform3) -> Self {
        let object = object.into();

        let obj_centre = object.centre();
        let correct_transform = Transform3::from_translation(-obj_centre.to_vector())
            .then(transform)
            .then_translate(obj_centre.to_vector());

        Self::new_without_correction(object, material, correct_transform)
    }

    /// Creates a new transformed mesh instance, using the given mesh and transform
    ///
    /// It is assumed that the mesh is either centred at the origin and the translation is stored in
    /// the transform, or that the transform correctly accounts for the mesh's translation.
    /// See field documentation ([Self::transform]) for explanation
    pub fn new_without_correction(object: impl Into<Mesh>, material: impl Into<Mat>, transform: Transform3) -> Self {
        let object = object.into();

        // Calculate the resulting AABB by transforming the corners of the input AABB.
        // And then we encompass those
        let aabb = object
            .aabb()
            .map(Aabb::corners)
            .map(|corners| corners.map(|c| transform.map_point(c)))
            .map(Aabb::encompass_points);

        let inv_transform = transform.inverse();

        Self {
            object,
            aabb,
            transform: Some(transform),
            inv_transform: Some(inv_transform),
            material: material.into(),
        }
    }

    /// Creates a new transformed mesh instance, using the given mesh. This method does not transform the [SimpleObject]
    pub fn new(object: impl Into<Mesh>, material: impl Into<Mat>) -> Self {
        // Calculate the resulting AABB by transforming the corners of the input AABB.
        let object = object.into();
        Self {
            aabb: object.aabb().copied(),
            transform: None,
            inv_transform: None,
            material: material.into(),
            object,
        }
    }
}

// endregion Constructors
