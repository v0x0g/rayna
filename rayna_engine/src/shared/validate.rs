use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use approx::*;
use rayna_shared::def::types::{Number, Pixel, Vector};
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

/// Asserts that an intersection was valid
#[inline(always)]
#[track_caller]
pub fn intersection(intersect: impl Borrow<Intersection>, bounds: impl Borrow<Bounds<Number>>) {
    debug_assert_only!();

    let intersect = intersect.borrow();
    let bounds = bounds.borrow();

    vector(intersect.pos);
    number(intersect.dist);
    ray(&intersect.ray);

    assert!(
        bounds.contains(&intersect.dist),
        "intersect dist {} not in bounds {}",
        intersect.dist,
        bounds
    );

    // Dist between start and end should match `.dist` field
    let ray_len = (intersect.ray.pos() - intersect.pos).length();
    assert_relative_eq!(
        ray_len,
        intersect.dist,
        epsilon = EPSILON,
        max_relative = RELATIVE
    );

    normal(intersect.ray_normal);
    normal(intersect.normal);
}

#[inline(always)]
#[track_caller]
pub fn number(x: impl Borrow<Number>) {
    debug_assert_only!();

    let x = x.borrow();
    assert!(!x.is_nan(), "x = {x}");
}

#[inline(always)]
#[track_caller]
pub fn normal(n: impl Borrow<Vector>) {
    debug_assert_only!();
    let n = n.borrow();
    vector(n);
    assert!(n.is_normalized(), "{n:?} ({:?})", n.length());
}

#[inline(always)]
#[track_caller]
pub fn vector(v: impl Borrow<Vector>) {
    debug_assert_only!();
    let v = v.borrow();
    assert!(!v.is_nan(), "{v:?}");
}

#[inline(always)]
#[track_caller]
pub fn ray(r: impl Borrow<Ray>) {
    debug_assert_only!();
    let r = r.borrow();
    normal(r.dir());
}

#[inline(always)]
#[track_caller]
pub fn colour(c: impl Borrow<Pixel>) {
    debug_assert_only!();
    let c = c.borrow();
    assert!(
        c.0.iter().all(|&chan| chan >= 0.0),
        "channels >= 0 for {c:?}"
    )
}
