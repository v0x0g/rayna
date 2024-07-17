use enum_dispatch::enum_dispatch;
use rand_core::RngCore;

use crate::core::types::Number;
use crate::scene::Scene;
use crate::shared::aabb::Bounded;
use crate::shared::intersect::ObjectIntersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use crate::shared::ComponentRequirements;

pub mod simple;
pub mod transform;
pub mod volumetric;

// TODO: Should objects (as well as other traits) have some sort of identifier?

#[doc(notable_trait)]
pub trait Object: ComponentRequirements + Bounded {
    /// Attempts to perform an intersection between the given ray and the target object
    ///
    /// # Return Value
    /// This should return the *first* intersection that is within the given range, else [`None`]
    fn full_intersect(
        &self,
        scene: &Scene,
        ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<ObjectIntersection>;

    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info (shadow checks)
}

#[derive(Clone, Debug)]
#[enum_dispatch(Object)]
pub enum ObjectInstance {
    SimpleObject(simple::SimpleObject),
    VolumetricObject(volumetric::VolumetricObject),
}
