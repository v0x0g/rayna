use crate::core::types::Number;
use crate::material::Material;
use crate::mesh::Mesh as MeshTrait;
use crate::object::transform::ObjectTransform;
use crate::object::Object;
use crate::shared::aabb::{Aabb, Bounded};
use crate::shared::intersect::FullIntersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use getset::Getters;
use rand_core::RngCore;

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
/// # use rayna_engine::mesh::primitive::axis_box::AxisBoxMesh;
/// # use rayna_engine::core::types::{Angle, Point3, Transform3, Vector3};
/// #
/// # let a: Point3 = [5., 1., 2.].into();
/// # let b: Point3 = [3., 4., -7.].into();
/// # let mesh = AxisBoxMesh::new(a,b);
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
/// This pre/post transform is encapsulated in [`ObjectTransform::new_corrected()`]
#[derive(Getters, Clone, Debug)]
#[get = "pub"]
pub struct SimpleObject<Mesh: MeshTrait, Mat: Material> {
    mesh: Mesh,
    material: Mat,
    transform: ObjectTransform,
    #[get(skip)]
    aabb: Option<Aabb>,
}

// region Constructors

impl<Mesh, Mat> SimpleObject<Mesh, Mat>
where
    Mesh: MeshTrait,
    Mat: Material,
{
    /// Creates a new transformed mesh instance, using the given mesh and transform
    ///
    /// This will apply translation-correction to the given transform (see field [Self::transform]), using the
    /// mesh's [`crate::mesh::MeshProperties::centre()`]
    pub fn new(mesh: impl Into<Mesh>, material: impl Into<Mat>, transform: impl Into<ObjectTransform>) -> Self {
        let mesh = mesh.into();
        let transform = transform.into().with_correction(mesh.centre());
        Self::new_uncorrected(mesh, material, transform)
    }

    /// Creates a new transformed mesh instance, using the given mesh and transform
    ///
    /// It is assumed that the mesh is either centred at the origin and the translation is stored in
    /// the transform, or that the transform correctly accounts for the mesh's translation.
    /// See field documentation ([Self::transform]) for explanation
    pub fn new_uncorrected(
        mesh: impl Into<Mesh>,
        material: impl Into<Mat>,
        transform: impl Into<ObjectTransform>,
    ) -> Self {
        let (mesh, material, transform) = (mesh.into(), material.into(), transform.into());
        let aabb = transform.calculate_aabb(mesh.aabb());

        Self {
            mesh,
            aabb,
            transform,
            material,
        }
    }
}

// endregion Constructors

// region Object Impl

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
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Mat>> {
        let trans_ray = self.transform.incoming_ray(orig_ray);
        let inner = self.mesh.intersect(&trans_ray, interval, rng)?;
        let intersect = self.transform.outgoing_intersection(orig_ray, inner);
        Some(intersect.make_full(&self.material))
    }
}

impl<Mesh, Mat> Bounded for SimpleObject<Mesh, Mat>
where
    Mesh: MeshTrait,
    Mat: Material,
{
    fn aabb(&self) -> Aabb { self.aabb.as_ref() }

    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info (shadow checks)
}

// endregion Object Impl
