use crate::core::types::{Number, Point3};
use dyn_clone::DynClone;

pub mod polygonised;
pub mod raymarched;

pub trait SdfGeneratorFunction: Fn(Point3) -> Number + Send + Sync + DynClone {}
impl<T: Fn(Point3) -> Number + Send + Sync + Clone> SdfGeneratorFunction for T {}
dyn_clone::clone_trait_object!(SdfGeneratorFunction);
