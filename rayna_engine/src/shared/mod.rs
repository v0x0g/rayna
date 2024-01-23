use std::fmt::Debug;

pub mod aabb;
pub mod bounds;
pub mod camera;
mod colour;
pub mod generic_bvh;
mod image;
pub mod impl_utils;
pub mod intersect;
pub mod math;
pub mod ray;
pub mod rng;
pub mod validate;

/// A simple marker trait that enforces a few other traits we need
/// in the engine
pub trait RtRequirement: Debug + Send + Sync {}
impl<T: Debug + Send + Sync> RtRequirement for T {}
