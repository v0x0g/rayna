use std::fmt::Debug;

pub mod aabb;
pub mod generic_bvh;
pub mod intersect;
pub mod interval;
pub mod math;
pub mod ray;
pub mod rng;
pub mod simd_math;
pub mod validate;

/// A simple marker trait that enforces a few other traits we need
/// in the engine
pub trait RtRequirement: Debug + Send + Sync {}
impl<T: Debug + Send + Sync> RtRequirement for T {}
