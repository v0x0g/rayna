use std::borrow::Borrow;

use getset::*;
use itertools::multizip;

use rayna_shared::def::types::{Number, Point3, Vector3};

use crate::shared::bounds::Bounds;
use crate::shared::ray::Ray;

/// An **Axis-Aligned Bounding Box** (AABB)
///
/// The box spans between the two corners `min` and `max`'
#[derive(CopyGetters, Copy, Clone, Debug, PartialEq, Default)]
#[getset(get_copy = "pub")]
pub struct Aabb {
    /// The lower corner of the [Aabb]; the corner with the smallest coordinates
    min: Point3,
    /// The upper corner of the [Aabb]; the corner with the largest coordinates
    max: Point3,
    size: Vector3,
    area: Number,
    volume: Number,
}

// region Constructors
impl Aabb {
    /// Creates a new [Aabb] from two points, which do *not* have to be sorted by min/max
    pub fn new(a: Point3, b: Point3) -> Self {
        let min = Point3::min(a, b);
        let max = Point3::max(a, b);
        let size = max - min;
        let area = ((size.x * size.y) + (size.y * size.z) + (size.z * size.x)) * 2.;
        let volume = size.x * size.y * size.z;
        Self {
            min,
            max,
            size,
            area,
            volume,
        }
    }

    /// Returns an [Aabb] that surrounds the two given boxes
    pub fn encompass<B: Borrow<Self>>(a: B, b: B) -> Self {
        let (a, b) = (a.borrow(), b.borrow());
        let min = Point3::min(a.min, b.min);
        let max = Point3::max(a.max, b.max);
        Self::new(min, max)
    }

    /// [Self::encompass] but for an arbitrary number of boxes
    pub fn encompass_iter<B: Borrow<Self>>(iter: impl IntoIterator<Item = B>) -> Self {
        iter.into_iter().fold(Self::default(), |a: Self, b: B| {
            Self::encompass(&a, b.borrow())
        })
    }

    /// [Self::encompass] but for an arbitrary number of points
    pub fn encompass_points<B: Borrow<Point3>>(iter: impl IntoIterator<Item = B>) -> Self {
        let mut min = Point3::ZERO;
        let mut max = Point3::ZERO;
        for p in iter.into_iter() {
            let p = p.borrow();
            min = min.min(*p);
            max = max.max(*p);
        }
        Self::new(min, max)
    }
}

// endregion Constructors

// region Impl
impl Aabb {
    /// Checks whether the given ray intersects with the AABB at any point within the given distance bounds
    pub fn hit(&self, ray: &Ray, bounds: &Bounds<Number>) -> bool {
        let ro = ray.pos().to_array();
        let rd = ray.dir().to_array();
        let min = self.min.to_array();
        let max = self.max.to_array();

        for i in 0..3_usize {
            let (ro_i, rd_i, min_i, max_i) = (ro[i], rd[i], min[i], max[i]);
            let inv_d = 1. / rd_i;
            let mut t0 = (min_i - ro_i) * inv_d;
            let mut t1 = (max_i - ro_i) * inv_d;
            if inv_d < 0. {
                std::mem::swap(&mut t0, &mut t1);
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
// endregion Impl
