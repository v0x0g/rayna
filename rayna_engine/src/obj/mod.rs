use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use rayna_shared::def::types::Number;

pub mod sphere;

dyn_clone::clone_trait_object!(Object);
pub trait Object: RtRequirement {
    /// Attempts to perform an intersection between the given ray and the target object
    ///
    /// # Return Value
    ///     This should return the *first* intersection that is within the given range, else [`None`]
    ///
    /// # Parameters
    ///     - ray: The
    fn intersect(&self, ray: Ray, bounds: Bounds<Number>) -> Option<Intersection> {
        self.intersect_all(ray)?
            .filter(|i| bounds.contains(&i.dist))
            .min_by(|a, b| Number::total_cmp(&a.dist, &b.dist))
    }

    /// Returns (possibly) an iterator over all of the intersections for the given object.
    ///
    /// # Return Value
    ///     This should return a (boxed) iterator that iterates over all the (unbounded) intersections,
    ///     unbounded by distance.
    fn intersect_all(&self, ray: Ray) -> Option<Box<dyn Iterator<Item = Intersection> + '_>>;

    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info
}
