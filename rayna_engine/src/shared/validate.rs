use crate::core::types::{Colour, Number, Point2, Point3, Vector3};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use approx::*;
use std::borrow::Borrow;

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

#[inline(always)]
#[track_caller]
pub fn number(val: impl Borrow<Number>) {
    debug_assert_only!();

    let val = val.borrow();
    assert!(!val.is_nan(), "should not be nan; val: {val}");
}

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

#[inline(always)]
#[track_caller]
pub fn point3(v: impl Borrow<Point3>) {
    debug_assert_only!();
    let v = v.borrow();
    assert!(!v.is_nan(), "should not be nan; vec: {v:?}");
}

#[inline(always)]
#[track_caller]
pub fn vector3(v: impl Borrow<Vector3>) {
    debug_assert_only!();
    let v = v.borrow();
    assert!(!v.is_nan(), "should not be nan; vec: {v:?}");
}

#[inline(always)]
#[track_caller]
pub fn ray(r: impl Borrow<Ray>) {
    debug_assert_only!();
    let r = r.borrow();
    normal3(r.dir());
}

#[inline(always)]
#[track_caller]
pub fn colour(c: impl Borrow<Colour>) {
    debug_assert_only!();
    let c = c.borrow();
    assert!(
        c.0.iter().all(|&chan| chan >= 0.0),
        "channels should be >= 0; col: {c:?}"
    )
}

#[inline(always)]
#[track_caller]
pub fn uv(uv: impl Borrow<Point2>) {
    debug_assert_only!();
    let uv = uv.borrow();
    assert!(!uv.is_nan(), "should not be nan; uvs: {uv:?}");
    // This check does not apply since the change allowing UV coordinates to be unbounded
    // assert!(
    //     (uv.cmpge(Point2::ZERO) & uv.cmple(Point2::ONE)).all(),
    //     "uv coordinates should be `0..=1`; uv: {uv:?}"
    // )
}

/// Asserts that an intersection was valid
#[inline(always)]
#[track_caller]
pub fn intersection(ray: impl Borrow<Ray>, intersect: impl Borrow<Intersection>, bounds: impl Borrow<Bounds<Number>>) {
    debug_assert_only!();

    let intersect = intersect.borrow();
    let bounds = bounds.borrow();
    let ray = ray.borrow();

    uv(&intersect.uv);

    point3(intersect.pos_w);
    number(intersect.dist);

    assert!(
        bounds.contains(&intersect.dist),
        "intersect dist {} not in bounds {}",
        intersect.dist,
        bounds
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

    normal3(intersect.ray_normal);
    normal3(intersect.normal);
}
