use crate::core::types::{Number, Point3};

pub mod polygonised;
pub mod raymarched;

pub trait SdfFunction: Fn(Point3) -> Number + Send + Sync {}
impl<T: Fn(Point3) -> Number + Send + Sync> SdfFunction for T {}
