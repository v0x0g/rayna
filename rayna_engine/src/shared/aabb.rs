use crate::shared::ComponentRequirements;
use enum_dispatch::enum_dispatch;
use std::borrow::Borrow;

use crate::core::types::{Number, Point3, Size3, Vector3};
use getset::*;
use itertools::Itertools;

use crate::shared::interval::Interval;
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
}

// region Constructors

impl Aabb {
    /// Creates a new [Aabb] from two points, which do *not* have to be sorted by min/max
    pub fn new(a: impl Into<Point3>, b: impl Into<Point3>) -> Self {
        let (a, b) = (a.into(), b.into());
        let min = Point3::min(a, b);
        let max = Point3::max(a, b);
        Self { min, max }
    }

    pub fn new_centred(centre: impl Into<Point3>, size: impl Into<Size3>) -> Self {
        let (centre, size) = (centre.into(), size.into());
        let min = centre - size.to_vector() / 2.;
        let max = centre + size.to_vector() / 2.;
        Self::new(min, max)
    }

    /// Returns an [Aabb] that surrounds the two given boxes
    pub fn encompass(a: impl Borrow<Self>, b: impl Borrow<Self>) -> Self {
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
    pub fn with_min_padding(&self, thresh: Number) -> Self {
        let mut dims = self.size();
        let centre = self.center();
        dims.as_array_mut().iter_mut().for_each(|d| *d = d.max(thresh));
        return Self::new_centred(centre, dims);
    }
}

// endregion Constructors

// region Helper
impl Aabb {
    /// A special [`Aabb`] value that indicates the lack of bounds, aka an infinite bounding box.
    ///
    /// This can be useful for shapes such as infinite planes, which cannot be bounded.
    ///
    /// This is currently implemented as an AABB with corners in both infinities,
    /// however this should not be relied upon. If an object is not bounded, it should
    /// *always* only return [INFINITE][`Aabb::INFINITE`], and no other possible values are allowed.
    pub const INFINITE: Self = Self {
        min: Point3::NEG_INFINITY,
        max: Point3::INFINITY,
    };

    pub const fn is_infinite(&self) -> bool { *self == Self::INFINITE }

    // Returns the corners of the AABB
    pub const fn corners(&self) -> [Point3; 8] {
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

    pub const fn size(&self) -> Size3 { Size3::from_vector(self.max() - self.min()) }
    pub const fn area(&self) -> Number {
        let size = self.size().to_vector();
        ((size.x * size.y) + (size.y * size.z) + (size.z * size.x)) * 2.
    }
    pub const fn volume(&self) -> Number { self.size().volume() }

    pub const fn center(&self) -> Point3 { self.max + (self.size().to_vector() / 2.0) }
}

// endregion Helper

// region Impl
impl Aabb {
    /// Checks whether the given ray intersects with the AABB at any point within the given distance interval
    pub fn hit(&self, ray: &Ray, interval: &Interval<Number>) -> bool {
        /*
        CREDITS:

        Author: Tavianator
        URL:
            - <https://tavianator.com/cgit/dimension.git/tree/libdimension/bvh/bvh.c#n196>
            - <https://tavianator.com/2011/ray_box.html>
        */

        // This is actually correct, even though it appears not to handle edge cases
        // (`ray.n.{x,y,z} == 0`). It works because the infinities that result from
        // dividing by zero will still behave correctly in the comparisons. Rays
        // which are parallel to an axis and outside the box will have `t_min == INF`
        // or `t_max == -INF`, while rays inside the box will have `t_min` and `t_max`
        // unchanged.

        // PERF: Can we elide this check; does the maths still work?
        if self.is_infinite() {
            return true;
        }

        let tx1 = (self.min.x - ray.pos().x) * ray.inv_dir().x;
        let tx2 = (self.max.x - ray.pos().x) * ray.inv_dir().x;

        let mut t_min = Number::min(tx1, tx2);
        let mut t_max = Number::max(tx1, tx2);

        let ty1 = (self.min.y - ray.pos().y) * ray.inv_dir().y;
        let ty2 = (self.max.y - ray.pos().y) * ray.inv_dir().y;

        t_min = Number::max(t_min, Number::min(ty1, ty2));
        t_max = Number::min(t_max, Number::max(ty1, ty2));

        let tz1 = (self.min.z - ray.pos().z) * ray.inv_dir().z;
        let tz2 = (self.max.z - ray.pos().z) * ray.inv_dir().z;

        t_min = Number::max(t_min, Number::min(tz1, tz2));
        t_max = Number::min(t_max, Number::max(tz1, tz2));

        return interval.range_overlaps(&t_min, &t_max);
    }
}

// endregion Impl

// region Bounded trait

#[enum_dispatch]
pub trait Bounded: ComponentRequirements {
    /// Gets the bounding box for this object.
    ///
    /// If the mesh can't be bounded (e.g. infinite plane), return [`Aabb::INFINITE`]
    fn aabb(&self) -> Aabb;
}

// endregion Bounded trait
