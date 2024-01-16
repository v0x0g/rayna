use dyn_clone::DynClone;
use std::fmt::Debug;

pub mod aabb;
pub mod bounds;
pub mod camera;
pub mod generic_bvh;
pub mod intersect;
pub mod math;
pub mod ray;
pub mod rng;
pub mod transform_utils;
pub mod validate;

// NOTE: We have to use [`DynClone`] instead of plain old [`Clone`],
// Since we will be using `Box<dyn Rt>` and we need to clone those boxes
dyn_clone::clone_trait_object!(RtRequirement);
//TODO: Remove the Clone/DynClone requirement
/// A simple marker trait that enforces a few other traits we need
/// in the engine
pub trait RtRequirement: DynClone + Debug + Send + Sync {}
impl<T: DynClone + Debug + Send + Sync> RtRequirement for T {}
