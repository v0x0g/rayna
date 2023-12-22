//! Helper module for RNG-related functions

use rand::Rng;
use rayna_shared::def::types::{Vector2, Vector3};

// region RNG Pool

// TODO: Thread-safe RNG pool

// endregion

// region 3D

/// Returns a random vector in a unit cube (-1..=1)
pub fn vector_in_unit_cube<R: Rng>(rng: &mut R) -> Vector3 {
    let mut arr = [0.; 3];
    arr.fill_with(|| rng.gen_range(-1.0..=1.0));
    arr.into()
}

/// Returns a random vector in a positive-only unit cube (`0..=1`)
pub fn vector_in_unit_cube_01<R: Rng>(rng: &mut R) -> Vector3 {
    let mut arr = [0.; 3];
    arr.fill_with(|| rng.gen_range(0.0..=1.0));
    arr.into()
}

/// Returns a random vector in a unit sphere (`-1..=1`, `length <= 1`)
pub fn vector_in_unit_sphere<R: Rng>(rng: &mut R) -> Vector3 {
    loop {
        let v = vector_in_unit_cube(rng);
        if v.length_squared() <= 1. {
            break v;
        }
    }
}
/// Returns a random vector on a unit sphere (`-1..=1`, `length = 1`)
pub fn vector_on_unit_sphere<R: Rng>(rng: &mut R) -> Vector3 {
    loop {
        let Some(vec) = vector_in_unit_sphere(rng).try_normalize() else {
            continue;
        };
        return vec;
    }
}

/// Returns a random vector in a unit hemisphere (`-1..=1`, `length = 1`)
/// The output vector is guaranteed to point in the same hemisphere as the normal,
/// i.e. `dot(vec, normal) >= 0.0`
pub fn vector_in_unit_hemisphere<R: Rng>(rng: &mut R, normal: Vector3) -> Vector3 {
    let vec = vector_in_unit_sphere(rng);
    // pointing same side as normal
    if Vector3::dot(vec, normal) >= 0. {
        vec
    } else {
        -vec
    }
}

/// Returns a random vector on a unit hemisphere (`-1..=1`, `length = 1`)
/// The output vector is guaranteed to point in the same hemisphere as the normal,
/// i.e. `dot(vec, normal) >= 0.0`, and have a unit length (
pub fn vector_on_unit_hemisphere<R: Rng>(rng: &mut R, normal: Vector3) -> Vector3 {
    loop {
        let Some(vec) = vector_in_unit_sphere(rng).try_normalize() else {
            continue;
        };
        // pointing same side as normal
        if Vector3::dot(vec, normal) >= 0. {
            break vec;
        } else {
            break -vec;
        }
    }
}

// endregion 3D

// region 2D

/// Returns a random vector in a unit square (-1..=1)
pub fn vector_in_unit_square<R: Rng>(rng: &mut R) -> Vector2 {
    let mut arr = [0.; 2];
    arr.fill_with(|| rng.gen_range(-1.0..=1.0));
    arr.into()
}

/// Returns a random vector in a positive-only unit square (`0..=1`)
pub fn vector_in_unit_square_01<R: Rng>(rng: &mut R) -> Vector2 {
    let mut arr = [0.; 2];
    arr.fill_with(|| rng.gen_range(0.0..=1.0));
    arr.into()
}

/// Returns a random vector in a unit circle (`-1..=1`, `length <= 1`)
pub fn vector_in_unit_circle<R: Rng>(rng: &mut R) -> Vector2 {
    loop {
        let v = vector_in_unit_square(rng);
        if v.length_squared() <= 1. {
            break v;
        }
    }
}
/// Returns a random vector on a unit circle (`-1..=1`, `length = 1`)
pub fn vector_on_unit_circle<R: Rng>(rng: &mut R) -> Vector2 {
    loop {
        let Some(vec) = vector_in_unit_circle(rng).try_normalize() else {
            continue;
        };
        return vec;
    }
}

/// Returns a random vector in a unit semicircle (`-1..=1`, `length = 1`)
/// The output vector is guaranteed to point in the same semicircle as the normal,
/// i.e. `dot(vec, normal) >= 0.0`
pub fn vector_in_unit_semicircle<R: Rng>(rng: &mut R, normal: Vector2) -> Vector2 {
    let vec = vector_in_unit_circle(rng);
    // pointing same side as normal
    if Vector2::dot(vec, normal) >= 0. {
        vec
    } else {
        -vec
    }
}

/// Returns a random vector on a unit semicircle (`-1..=1`, `length = 1`)
/// The output vector is guaranteed to point in the same semicircle as the normal,
/// i.e. `dot(vec, normal) >= 0.0`, and have a unit length (
pub fn vector_on_unit_semicircle<R: Rng>(rng: &mut R, normal: Vector2) -> Vector2 {
    loop {
        let Some(vec) = vector_in_unit_circle(rng).try_normalize() else {
            continue;
        };
        // pointing same side as normal
        if Vector2::dot(vec, normal) >= 0. {
            break vec;
        } else {
            break -vec;
        }
    }
}

//endregion 2D
