use crate::shared::bounds::Bounds;
use crate::shared::ray::Ray;
use getset::*;
use itertools::multizip;
use rayna_shared::def::types::{Number, Point3};

/// An **Axis-Aligned Bounding Box** (AABB)
///
/// The box spans between the two corners `min` and `max`'
#[derive(CopyGetters, Copy, Clone, Debug, PartialEq)]
#[getset(get_copy)]
pub struct Aabb {
    /// The lower corner of the [Aabb]; the corner with the smallest coordinates
    min: Point3,
    /// The upper corner of the [Aabb]; the corner with the largest coordinates
    max: Point3,
}

impl Aabb {
    /// Creates a new [Aabb] from two points, which do *not* have to be sorted by min/max
    pub fn new(a: Point3, b: Point3) -> Self {
        let min = Point3::min(a, b);
        let max = Point3::max(a, b);
        Self { min, max }
    }

    pub fn hit(&self, ray: Ray, bounds: Bounds<Number>) -> bool {
        let ro = ray.pos().to_array();
        let rd = ray.dir().to_array();
        let min = self.min.to_array();
        let max = self.max.to_array();

        for (ro_i, rd_i, min_i, max_i) in multizip((ro, rd, min, max)) {
            let inv_d = 1. / rd_i;
            let mut t0 = (min_i - ro_i) * inv_d;
            let mut t1 = (max_i - ro_i) * inv_d;
            if inv_d < 0. {
                (t1, t0) = (t0, t1);
            }

            // The range in which the ray is 'inside' the AABB
            // Is not within the valid range for the ray,
            // so there is no valid intersection
            if !bounds.range_overlaps(&t0, &t1) {
                return false;
            }
        }
        return true;
    }
}
