use std::borrow::Borrow;

use getset::*;

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
    /// The difference between [min](fn@Self::min) and [max](fn@Self::max); how large the [Aabb] is
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

    pub fn new_centred(centre: Point3, size: Vector3) -> Self {
        let min = centre - size / 2.;
        let max = centre + size / 2.;
        Self::new(min, max)
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
        iter.into_iter()
            .fold(Self::default(), |a: Self, b: B| Self::encompass(&a, b.borrow()))
    }

    /// [Self::encompass] but for an arbitrary number of points
    pub fn encompass_points<B: Borrow<Point3>>(iter: impl IntoIterator<Item = B>) -> Self {
        let mut min = Point3::splat(Number::INFINITY);
        let mut max = Point3::splat(Number::NEG_INFINITY);
        for p in iter.into_iter() {
            let p = *p.borrow();
            min = min.min(p);
            max = max.max(p);
        }
        Self::new(min, max)
    }

    /// Ensures that an AABB has all sides of at least `thresh` thickness.
    /// If any side widths between corners are less than this threshold, the [Aabb] will
    /// be expanded (away from the centre) to fit.
    pub fn min_padded(&self, thresh: Number) -> Self {
        let mut dims = self.size();
        let centre = self.min + dims / 2.;
        dims.as_array_mut().iter_mut().for_each(|d| *d = d.max(thresh));
        return Self::new_centred(centre, dims);
    }
}

// endregion Constructors

// region Helper
impl Aabb {
    // Returns the corners of the AABB
    pub fn corners(&self) -> [Point3; 8] {
        let (l, h) = (self.min, self.max);
        [
            [l.x, l.y, l.z].into(),
            [l.x, l.y, h.z].into(),
            [l.x, h.y, l.z].into(),
            [l.x, h.y, h.z].into(),
            [h.x, l.y, l.z].into(),
            [h.x, l.y, h.z].into(),
            [h.x, h.y, l.z].into(),
            [h.x, h.y, h.z].into(),
        ]
    }
}

// endregion Helper

// region Impl
impl Aabb {
    /// Checks whether the given ray intersects with the AABB at any point within the given distance bounds
    pub fn hit(&self, ray: &Ray, bounds: &Bounds<Number>) -> bool {
        /*
        CREDITS:

        Author: Tavianator
        URL:
            - <https://tavianator.com/cgit/dimension.git/tree/libdimension/bvh/bvh.c#n196>
            - <https://tavianator.com/2011/ray_box.html>
        */

        // This is actually correct, even though it appears not to handle edge cases
        // (ray.n.{x,y,z} == 0). It works because the infinities that result from
        // dividing by zero will still behave correctly in the comparisons. Rays
        // which are parallel to an axis and outside the box will have tmin == inf
        // or tmax == -inf, while rays inside the box will have tmin and tmax
        // unchanged.

        let tx1 = (self.min.x - ray.pos().x) * ray.inv_dir().x;
        let tx2 = (self.max.x - ray.pos().x) * ray.inv_dir().x;

        let mut tmin = Number::min(tx1, tx2);
        let mut tmax = Number::max(tx1, tx2);

        let ty1 = (self.min.y - ray.pos().y) * ray.inv_dir().y;
        let ty2 = (self.max.y - ray.pos().y) * ray.inv_dir().y;

        tmin = Number::max(tmin, Number::min(ty1, ty2));
        tmax = Number::min(tmax, Number::max(ty1, ty2));

        let tz1 = (self.min.z - ray.pos().z) * ray.inv_dir().z;
        let tz2 = (self.max.z - ray.pos().z) * ray.inv_dir().z;

        tmin = Number::max(tmin, Number::min(tz1, tz2));
        tmax = Number::min(tmax, Number::max(tz1, tz2));

        return bounds.range_overlaps(&tmin, &tmax);
    }
}
// endregion Impl