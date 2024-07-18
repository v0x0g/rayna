use std::fmt::Debug;

pub mod aabb;
pub mod intersect;
pub mod interval;
pub mod math;
pub mod ray;
pub mod rng;
pub mod simd_math;
pub mod token;
pub mod validate;

/// A simple marker trait that enforces a few other traits we need
/// in the engine
// TODO: Add a requirement for `valuable::Valuable`
pub trait ComponentRequirements: Debug + Send + Sync {}
impl<T: Debug + Send + Sync> ComponentRequirements for T {}
