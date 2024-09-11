use enum_dispatch::enum_dispatch;
use rand_core::RngCore;

use crate::core::aabb::Aabb;
use crate::core::aabb::Bounded;
use crate::core::component::Component;
use crate::core::intersect::ObjectIntersection;
use crate::core::interval::Interval;
use crate::core::ray::Ray;
use crate::core::token::generate_component_token;
use crate::core::types::Number;
use crate::scene::Scene;

mod list;
pub mod simple;
pub mod transform;
pub mod volumetric;
// TODO: Should objects (as well as other traits) have some sort of identifier?

#[enum_dispatch]
pub trait Object: Component + Bounded {
    /// Attempts to perform an intersection between the given ray and the target object
    ///
    /// # Return Value
    /// This should return the *first* intersection that is within the given range, else [`None`]
    // TODO: rename to intersect
    fn intersect(
        &self,
        scene: &Scene,
        ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<ObjectIntersection>;

    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info (shadow checks)
}

#[derive(Clone, Debug)]
#[enum_dispatch(Object, Bounded)]
pub enum ObjectInstance {
    SimpleObject(simple::SimpleObject),
    VolumetricObject(volumetric::VolumetricObject),
}

generate_component_token!(ObjectToken for ObjectInstance);
