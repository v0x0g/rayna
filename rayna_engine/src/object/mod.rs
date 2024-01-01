use crate::accel::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use enum_dispatch::enum_dispatch;
use rayna_shared::def::types::Number;
// noinspection ALL - Used by enum_dispatch macro
#[allow(unused_imports)]
use self::{dynamic::DynamicObject, sphere::SphereObject};

mod r#box;
pub mod dynamic;
pub mod sphere;

dyn_clone::clone_trait_object!(Object);
#[enum_dispatch]
pub trait Object: RtRequirement {
    /// Attempts to perform an intersection between the given ray and the target object
    ///
    /// # Return Value
    ///     This should return the *first* intersection that is within the given range, else [`None`]
    ///
    /// # Parameters
    ///     - ray: The
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        self.intersect_all(ray)?
            .filter(|i| bounds.contains(&i.dist))
            .min_by(|a, b| Number::total_cmp(&a.dist, &b.dist))
    }

    /// Returns (possibly) an iterator over all of the intersections for the given object.
    ///
    /// # Return Value
    ///     This should return a (boxed) iterator that iterates over all the (unbounded) intersections,
    ///     unbounded by distance.
    fn intersect_all<'a>(
        &'a self,
        ray: &'a Ray,
    ) -> Option<Box<dyn Iterator<Item = Intersection> + 'a>>;

    fn bounding_box(&self) -> &Aabb;

    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info (shadow checks)
}

/// An optimised implementation of [Object].
///
/// See [crate::material::MaterialType] for an explanation of the [enum_dispatch] macro usage
#[enum_dispatch(Object)]
#[derive(Clone, Debug)]
pub enum ObjectType {
    SphereObject,
    DynamicObject,
}
