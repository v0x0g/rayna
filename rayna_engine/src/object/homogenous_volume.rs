// use crate::accel::aabb::Aabb;
// use crate::object::{Object, ObjectProperties};
// use crate::shared::bounds::Bounds;
// use crate::shared::intersect::{FullIntersection, Intersection};
// use crate::shared::ray::Ray;
// use derivative::Derivative;
// use getset::Getters;
// use rayna_shared::def::types::{Number, Point3, Transform3, Vector3};
// use smallvec::SmallVec;
//
// /// An object wrapper that treats the wrapped object as a constant-density volume
// #[derive(Derivative, Getters)]
// #[derivative(Debug(bound = ""), Clone(bound = ""), Copy)]
// #[get = "pub"]
// pub struct HomogenousVolumeObject<Obj: Object + Clone> {
//     object: Obj,
//     transform: Transform3,
//     inv_transform: Transform3,
//     aabb: Option<Aabb>,
//     /// The transformed centre of the object
//     centre: Point3,
// }
//
// impl<Obj: Object + Clone> HomogenousVolumeObject<Obj> {
//     /// Creates a new transformed object instance, using the given properties
//     pub fn new(object: Obj, transform: Transform3) -> Self {
//         // Calculate the resulting AABB by transforming the corners of the input AABB.
//         // And then we encompass those
//         let aabb = object
//             .aabb()
//             .map(Aabb::corners)
//             .map(|corners| corners.map(|c| transform.map_point(c)))
//             .map(Aabb::encompass_points);
//
//         let inv_transform = transform.inverse();
//         let centre = transform.map_point(object.centre());
//
//         Self {
//             object,
//             aabb,
//             transform,
//             inv_transform,
//             centre,
//         }
//     }
//
//     /// Applies the transform matrix in `self` to the given ray.
//     ///
//     /// # Note
//     /// This actually uses the inverse transform to go from world -> object space
//     /// (the plain `transform` is object -> world space
//     #[inline(always)]
//     fn transform_ray(&self, ray: &Ray) -> Ray {
//         let (pos, dir) = (ray.pos(), ray.dir());
//         let tf = &self.inv_transform;
//         Ray::new(tf.map_point(pos), tf.map_vector(dir))
//     }
//
//     /// Applies the transform matrix in `self` to the given intersection.
//     #[inline(always)]
//     fn transform_intersection(&self, orig_ray: &Ray, intersection: &Intersection) -> Intersection {
//         // PANICS:
//         // We use `.unwrap()` on the results of the transformations
//         // Since it is of type `Transform3`, it must be an invertible matrix and can't collapse
//         // the dimensional space to <3 dimensions,
//         // Therefore all transformation should be valid (and for vectors: nonzero), so we can unwrap
//
//         let mut intersection = intersection.clone();
//
//         let tf = &self.transform;
//         let point = |p: &mut Point3| *p = tf.matrix.transform_point(*p);
//         let normal = |n: &mut Vector3| {
//             *n = tf.map_vector(*n).try_normalize().expect(&format!(
//                 "transformation failed: vector {n:?} transformed to {t:?} couldn't be normalised",
//                 t = tf.map_vector(*n)
//             ))
//         };
//
//         normal(&mut intersection.normal);
//         normal(&mut intersection.ray_normal);
//         point(&mut intersection.pos_l);
//         point(&mut intersection.pos_w);
//
//         // Minor hack, calculate the intersection distance instead of transforming it
//         // I don't know how else to do this lol
//         intersection.dist = (intersection.pos_w - orig_ray.pos()).length();
//
//         return intersection;
//     }
// }
//
// impl<Obj: Object + Clone> Object for TransformedObject<Obj> {
//     fn intersect<'o>(&'o self, orig_ray: &Ray, bounds: &Bounds<Number>) -> Option<FullIntersection<'o>> {
//         let trans_ray = self.transform_ray(orig_ray);
//         let mut i = self.object.intersect(&trans_ray, bounds)?;
//         i.intersection = self.transform_intersection(orig_ray, &i.intersection);
//         Some(i)
//     }
//
//     fn intersect_all<'o>(&'o self, orig_ray: &Ray, output: &mut SmallVec<[FullIntersection<'o>; 32]>) {
//         let trans_ray = self.transform_ray(orig_ray);
//         let initial_len = output.len();
//         self.object.intersect_all(&trans_ray, output);
//         let new_len = output.len();
//
//         // Tracking length means we can find the intersections that were added
//         let inner_intersects = &mut output[initial_len..new_len];
//         inner_intersects
//             .into_iter()
//             .for_each(|i| i.intersection = self.transform_intersection(orig_ray, &i.intersection))
//     }
// }
//
// impl<Obj: Object + Clone> ObjectProperties for TransformedObject<Obj> {
//     fn aabb(&self) -> Option<&Aabb> { self.aabb.as_ref() }
//     fn centre(&self) -> Point3 { self.centre }
// }
