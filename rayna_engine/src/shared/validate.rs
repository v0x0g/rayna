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

/// Asserts that an intersection was valid
#[inline]
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
    assert_relative_eq!(ray_len, intersect.dist);

    normal(intersect.ray_normal);
    normal(intersect.normal);
}

#[inline]
pub fn number(x: impl Borrow<Number>) {
    debug_assert_only!();

    assert!(!x.borrow().is_nan());
}

#[inline]
pub fn normal(n: impl Borrow<Vector>) {
    debug_assert_only!();

    vector(n.borrow());
    assert!(n.borrow().is_normalized());
}

#[inline]
pub fn vector(v: impl Borrow<Vector>) {
    debug_assert_only!();

    assert!(!v.borrow().is_nan());
}

#[inline]
pub fn ray(r: impl Borrow<Ray>) {
    debug_assert_only!();

    normal(r.borrow().dir());
    vector(r.borrow().dir());
}

#[inline]
pub fn colour(c: impl Borrow<Pixel>) {
    assert!(c.borrow().0.iter().all(|&chan| chan >= 0.0))
}
