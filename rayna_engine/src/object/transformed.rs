use crate::accel::aabb::Aabb;
use crate::object::{Object, ObjectProperties};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::{FullIntersection, Intersection};
use crate::shared::ray::Ray;
use derivative::Derivative;
use getset::Getters;
use rayna_shared::def::types::{Number, Point3, Transform3, Vector3};
use smallvec::SmallVec;

/// An object wrapper that applies a transformation matrix (3D affine transform, see [Transform3]),
/// to the wrapped object
///
/// # Important Note
/// If using a rotating/scaling transform, ensure that the object you are transforming is positioned
/// at the origin (`[0., 0., 0.]`), and use the transform matrix to do the translation.
///
/// Otherwise, the centre of the object will be rotated/scaled around the origin as well, which will move the object.
///
/// Alternatively, you can also apply a post and pre-transform, to counteract the object's position offset:
/// ```///
/// # use rayna_engine::material::lambertian::LambertianMaterial;
/// # use rayna_engine::object::axis_box::{AxisBoxBuilder, AxisBoxObject};
/// # use rayna_shared::def::types::{Angle, Point3, Transform3, Vector3};
/// #
/// # let a: Point3 = [5., 1., 2.].into();
/// # let b: Point3 = [3., 4., -7.].into();
/// # let object: AxisBoxObject = AxisBoxBuilder {
/// #     corner_1: a,
/// #     corner_2: b,
/// #     material: Default::default(),
/// # }.into();
///
/// let transform = Transform3::from_axis_angle(Vector3::Y, Angle::from_degrees(69.0));
///
/// // Fix the transform so it scales/rotates around the object's centre and not the origin
/// //  1. Move centre to origin
/// //  2. Apply rotate/scale, while it is centred at origin
/// //  3. Move centre back to original position
/// let transform = Transform3::from_translation(-object.centre().to_vector())
///     .then(transform)
///     .then_translate(object.centre().to_vector());
/// ```
#[derive(Derivative, Getters)]
#[derivative(Debug(bound = ""), Clone(bound = ""), Copy)]
#[get = "pub"]
pub struct TransformedObject<Obj: Object + Clone> {
    object: Obj,
    transform: Transform3,
    inv_transform: Transform3,
    aabb: Option<Aabb>,
    /// The transformed centre of the object
    centre: Point3,
}

impl<Obj: Object + Clone> TransformedObject<Obj> {
    /// Creates a new transformed object instance, using the given object and transform matrix.
    ///
    /// Unlike [Self::new()], this *does* account for the object's translation from the origin,
    /// using the `obj_centre` parameter. See type documentation ([TransformedObject]) for explanation
    /// and example of this position offset correction
    pub fn new_with_correction(obj_centre: Point3, object: Obj, transform: Transform3) -> Self {
        let correct_transform = Transform3::from_translation(-obj_centre.to_vector())
            .then(transform)
            .then_translate(obj_centre.to_vector());

        Self::new(object, correct_transform)
    }

    /// Creates a new transformed object instance, using the given object and transform
    ///
    /// It is assumed that the object is either centred at the origin and the translation is stored in
    /// the transform, or that the transform correctly accounts for the object's translation.
    /// See type documentation ([TransformedObject]) for explanation
    pub fn new(object: Obj, transform: Transform3) -> Self {
        // Calculate the resulting AABB by transforming the corners of the input AABB.
        // And then we encompass those
        let aabb = object
            .aabb()
            .map(Aabb::corners)
            .map(|corners| corners.map(|c| transform.map_point(c)))
            .map(Aabb::encompass_points);

        let inv_transform = transform.inverse();
        let centre = transform.map_point(object.centre());

        Self {
            object,
            aabb,
            transform,
            inv_transform,
            centre,
        }
    }

    /// Applies the transform matrix in `self` to the given ray.
    ///
    /// # Note
    /// This actually uses the inverse transform to go from world -> object space
    /// (the plain `transform` is object -> world space
    #[inline(always)]
    fn transform_ray(&self, ray: &Ray) -> Ray {
        let (pos, dir) = (ray.pos(), ray.dir());
        let tf = &self.inv_transform;
        Ray::new(tf.map_point(pos), tf.map_vector(dir))
    }

    /// Applies the transform matrix in `self` to the given intersection.
    #[inline(always)]
    fn transform_intersection(&self, orig_ray: &Ray, intersection: &Intersection) -> Intersection {
        // PANICS:
        // We use `.unwrap()` on the results of the transformations
        // Since it is of type `Transform3`, it must be an invertible matrix and can't collapse
        // the dimensional space to <3 dimensions,
        // Therefore all transformation should be valid (and for vectors: nonzero), so we can unwrap

        let mut intersection = intersection.clone();

        let tf = &self.transform;
        let point = |p: &mut Point3| *p = tf.matrix.transform_point(*p);
        let normal = |n: &mut Vector3| {
            *n = tf.map_vector(*n).try_normalize().expect(&format!(
                "transformation failed: vector {n:?} transformed to {t:?} couldn't be normalised",
                t = tf.map_vector(*n)
            ))
        };

        normal(&mut intersection.normal);
        normal(&mut intersection.ray_normal);
        point(&mut intersection.pos_l);
        point(&mut intersection.pos_w);

        // Minor hack, calculate the intersection distance instead of transforming it
        // I don't know how else to do this lol
        intersection.dist = (intersection.pos_w - orig_ray.pos()).length();

        return intersection;
    }
}

impl<Obj: Object + Clone> Object for TransformedObject<Obj> {
    fn intersect<'o>(&'o self, orig_ray: &Ray, bounds: &Bounds<Number>) -> Option<FullIntersection<'o>> {
        let trans_ray = self.transform_ray(orig_ray);
        let mut i = self.object.intersect(&trans_ray, bounds)?;
        i.intersection = self.transform_intersection(orig_ray, &i.intersection);
        Some(i)
    }

    fn intersect_all<'o>(&'o self, orig_ray: &Ray, output: &mut SmallVec<[FullIntersection<'o>; 32]>) {
        let trans_ray = self.transform_ray(orig_ray);
        let initial_len = output.len();
        self.object.intersect_all(&trans_ray, output);
        let new_len = output.len();

        // Tracking length means we can find the intersections that were added
        let inner_intersects = &mut output[initial_len..new_len];
        inner_intersects
            .into_iter()
            .for_each(|i| i.intersection = self.transform_intersection(orig_ray, &i.intersection))
    }
}

impl<Obj: Object + Clone> ObjectProperties for TransformedObject<Obj> {
    fn aabb(&self) -> Option<&Aabb> { self.aabb.as_ref() }
    fn centre(&self) -> Point3 { self.centre }
}
