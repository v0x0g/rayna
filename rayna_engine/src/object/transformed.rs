use crate::accel::aabb::Aabb;
use crate::object::Object;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use derivative::Derivative;
use rayna_shared::def::types::{Number, Point3, Transform3, Vector3};
use smallvec::SmallVec;

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""), Copy)]
pub struct TransformedObject<Obj: Object + Clone> {
    inner: Obj,
    transform: Transform3,
    inv_transform: Transform3,
    aabb: Option<Aabb>,
}

impl<Obj: Object + Clone> TransformedObject<Obj> {
    pub fn new(object: Obj, transform: Transform3) -> Self {
        // Calculate the resulting AABB by transforming the corners of the input AABB.
        // And then we encompass those
        let aabb = object
            .aabb()
            .map(Aabb::corners)
            .map(|corners| corners.map(|c| transform.map_point(c)))
            .map(Aabb::encompass_points);

        Self {
            inner: object,
            aabb,
            transform,
            inv_transform: transform.inverse(),
        }
    }

    /// Applies the transform matrix in `self` to the given ray.
    ///
    /// # Note
    /// This actually uses the inverse transform to go from world -> object space
    /// (the plain `transform` is object -> world space
    #[inline(always)]
    fn transform_ray(&self, ray: Ray) -> Ray {
        let (pos, dir) = (ray.pos(), ray.dir());
        Ray::new(self.inv_transform.map_point(pos), self.inv_transform.map_vector(dir))
    }

    /// Applies the transform matrix in `self` to the given ray.
    ///
    /// # Note
    /// This actually uses the inverse transform to go from world -> object space
    /// (the plain `transform` is object -> world space
    #[inline(always)]
    fn transform_intersection(&self, trans_ray: &Ray, intersection: &Intersection) -> Intersection {
        // PANICS:
        // We use `.unwrap()` on the results of the transformations
        // Since it is of type `Transform3`, it must be an invertible matrix and can't collapse
        // the dimensional space to <3 dimensions,
        // Therefore all transformation should be valid (and for vectors: nonzero), so we can unwrap

        let mut intersection = intersection.clone();

        let tf = &self.transform;
        let point = |p: &mut Point3| {
            // TODO: I'm unsure why it mentions "perspective projection", we don't want perspective
            //  This should just be a plain affine 3D transform (scale + rotate + translate)
            //  This divides by the result's `W` axis, which should always be one since it's an affine transform???
            //  Also might be faster if it skips steps
            // tf.matrix.transform_point(p)

            *p = tf.map_point(*p);
        };

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
        intersection.dist = (intersection.pos_w - trans_ray.pos()).length();

        return intersection;
    }
}

impl<Obj: Object + Clone> Object for TransformedObject<Obj> {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        let trans_ray = self.transform_ray(*ray);
        self.inner
            .intersect(&trans_ray, bounds)
            .map(|i| self.transform_intersection(&trans_ray, &i))
    }

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>) {
        let trans_ray = self.transform_ray(*ray);
        let initial_len = output.len();
        self.inner.intersect_all(&trans_ray, output);
        let new_len = output.len();

        // Tracking length means we can find the intersections that were added
        let inner_intersects = &mut output[initial_len..new_len];
        inner_intersects
            .into_iter()
            .for_each(|i| *i = self.transform_intersection(&trans_ray, i))
    }

    fn aabb(&self) -> Option<&Aabb> { self.aabb.as_ref() }
}
