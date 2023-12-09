use crate::obj::Object;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::{Num, Vec3};
use std::ops::Range;

#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    pos: Vec3,
    radius: Num,
}

impl Object for Sphere {
    fn intersect(&self, ray: Ray, dist_bounds: Range<Num>) -> Option<Intersection> {
        None
    }

    fn intersect_all(&self, ray: Ray) -> Option<Box<dyn Iterator<Item = Intersection>>> {
        None
    }
}
