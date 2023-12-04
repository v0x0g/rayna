use std::ops::Range;
use crate::shared::intersect::Intersection;
use crate::shared::Num;
use crate::shared::ray::Ray;

pub mod sphere;

pub trait Object {
    /// Attempts to perform an intersection between the given ray and the target object
    ///
    /// # Return Value
    ///     This should return the *first* intersection that is within the given range, else [`None`]
    ///
    /// # Parameters
    ///     - ray: The
    fn intersect(&self, ray: Ray, dist_bounds: Range<Num>) -> Option<Intersection>;

    /// Returns (possibly) an iterator over all of the intersections for the given object.
    ///
    /// # Return Value
    ///     This should return a (boxed) iterator that iterates over all the (unbounded) intersections,
    ///     unbounded by distance.
    fn intersect_all(&self, ray: Ray) -> Option<Box<dyn Iterator<Item=Intersection>>>;
}