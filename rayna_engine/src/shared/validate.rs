use crate::core::types::{Channel, Colour, Number, Point2, Point3, Vector2, Vector3};
use crate::shared::intersect::MeshIntersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use approx::*;
use std::borrow::Borrow;

/// Macro that inserts a `return` statement if debug assertions are disabled
///
/// Required because we use some of the asserts from [`approx`],
/// which don't have a [`debug_assert!`] equivalent, so the only way to not
/// execute them in release is to return.
macro_rules! debug_assert_only {
    () => {
        if cfg!(not(debug_assertions)) {
            return;
        }
    };
}

pub const EPSILON: Number = 1e-6;
pub const ULPS: usize = 4;
pub const RELATIVE: Number = 1e-3;

/// Check is not [`Number::NAN`] and not [`Number::INFINITY`]
#[inline(always)]
#[track_caller]
pub fn number(val: impl Borrow<Number>) {
    debug_assert_only!();
    let val = val.borrow();

    assert!(!val.is_nan(), "should not be nan; val: {val}");
    assert!(!val.is_infinite(), "should not be inf; val: {val}");
}

/// Check is a valid vector, and normalised
#[inline(always)]
#[track_caller]
pub fn normal3(n: impl Borrow<Vector3>) {
    debug_assert_only!();
    let n = n.borrow();

    vector3(n);
    assert!(
        n.is_normalized(),
        "should be normalised; vec: {n:?}, len: {:?}",
        n.length()
    );
}

/// Check is a valid vector, and normalised
#[inline(always)]
#[track_caller]
pub fn normal2(n: impl Borrow<Vector2>) {
    debug_assert_only!();
    let n = n.borrow();

    vector2(n);
    assert!(
        n.is_normalized(),
        "should be normalised; vec: {n:?}, len: {:?}",
        n.length()
    );
}

/// Check all components are valid numbers
#[inline(always)]
#[track_caller]
pub fn point3(v: impl Borrow<Point3>) {
    debug_assert_only!();
    let p = v.borrow();

    for c in p.as_array() {
        number(c);
    }
}

/// Check all components are valid numbers
#[inline(always)]
#[track_caller]
pub fn vector2(v: impl Borrow<Vector2>) {
    debug_assert_only!();
    let v = v.borrow();

    for c in v.as_array() {
        number(c);
    }
}

/// Check all components are valid numbers
#[inline(always)]
#[track_caller]
pub fn vector3(v: impl Borrow<Vector3>) {
    debug_assert_only!();
    let v = v.borrow();

    for c in v.as_array() {
        number(c);
    }
}

/// Check position and direction are valid (ignore [`Ray::inv_dir`])
#[inline(always)]
#[track_caller]
pub fn ray(r: impl Borrow<Ray>) {
    debug_assert_only!();
    let r = r.borrow();

    normal3(r.dir());
    point3(r.pos());
}

/// Same checks as [`self::number`], as well as non-negative
#[inline(always)]
#[track_caller]
pub fn channel(c: impl Borrow<Channel>) {
    debug_assert_only!();
    let c = c.borrow();

    assert!(!c.is_nan(), "should not be nan; val: {c}");
    assert!(!c.is_infinite(), "should not be inf; val: {c}");
    assert!(*c >= 0.0, "channel should be >= 0; chan: {c:?}");
}

/// Check all channels are valid
#[inline(always)]
#[track_caller]
pub fn colour(col: impl Borrow<Colour>) {
    debug_assert_only!();
    let col = col.borrow();

    for c in col.into_iter() {
        channel(c);
    }
}

/// Check UV coordinates are in the range `[0..1]`, and not [`Number::NAN`]
#[inline(always)]
#[track_caller]
pub fn uv(uv: impl Borrow<Point2>) {
    debug_assert_only!();
    let uv = uv.borrow();

    assert!(!uv.is_nan(), "should not be nan; uvs: {uv:?}");

    assert!(
        (uv.cmpge(Point2::ZERO) & uv.cmple(Point2::ONE)).all(),
        "uv coordinates should be `0..=1`; uv: {uv:?}"
    )
}

/// Asserts that an intersection was valid
///
/// Validates all sub-fields (`ray`, `uv`, etc), as well as checking that
/// the intersection position matches the distance along the ray
#[inline(always)]
#[track_caller]
pub fn intersection(
    ray: impl Borrow<Ray>,
    intersect: impl Borrow<MeshIntersection>,
    interval: impl Borrow<Interval<Number>>,
) {
    debug_assert_only!();
    let intersect = intersect.borrow();
    let interval = interval.borrow();
    let ray = ray.borrow();

    uv(&intersect.uv);
    point3(intersect.pos_w);
    number(intersect.dist);
    normal3(intersect.ray_normal);
    normal3(intersect.normal);

    assert!(
        interval.contains(&intersect.dist),
        "intersect dist {} not in interval {}",
        intersect.dist,
        interval
    );

    // Dist between start and end should match `.dist` field
    let ray_len = (ray.pos() - intersect.pos_w).length();
    assert_relative_eq!(ray_len, intersect.dist, epsilon = EPSILON, max_relative = RELATIVE);

    assert!(
        Point3::relative_eq(
            &intersect.pos_w,
            &ray.at(intersect.dist),
            EPSILON,
            RELATIVE
        ),
        "intersect position doesn't match ray at intersection dist; intersect_pos: {i_pos:?}, dist: {dist}, ray: {ray:?}, ray_pos: {r_pos:?}",
        i_pos = intersect.pos_w,
        dist = intersect.dist,
        ray = ray,
        r_pos = ray.at(intersect.dist)
    );
}
