use crate::obj::Object;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use rayna_shared::def::types::{Number, Vector};
use std::ops::Range;

#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    pub pos: Vector,
    pub radius: Number,
}

impl Object for Sphere {
    fn intersect(&self, _ray: Ray, _dist_bounds: Range<Number>) -> Option<Intersection> {
        None
    }

    fn intersect_all(&self, _ray: Ray) -> Option<Box<dyn Iterator<Item = Intersection>>> {
        None
    }
}
