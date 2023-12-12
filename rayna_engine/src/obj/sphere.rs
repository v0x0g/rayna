use crate::def::types::{Num, Vec3};
use crate::obj::Object;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use std::ops::Range;

#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    pub pos: Vec3,
    pub radius: Num,
}

#[profiling::all_functions]
impl Object for Sphere {
    fn intersect(&self, _ray: Ray, _dist_bounds: Range<Num>) -> Option<Intersection> {
        None
    }

    fn intersect_all(&self, _ray: Ray) -> Option<Box<dyn Iterator<Item = Intersection>>> {
        None
    }
}
