//! Helper module for RNG-related functions

use rand::Rng;
use rayna_shared::def::types::Vector;

/// Returns a random vector in a unit cube (-1..=1)
pub fn vector_in_unit_cube<R: Rng>(rng: &mut R) -> Vector {
    let mut arr = [0.; 3];
    arr.fill_with(|| rng.gen_range(-1.0..=1.0));
    arr.into()
}

/// Returns a random vector in a positive-only unit cube (`0..=1`)
pub fn vector_in_unit_cube_01<R: Rng>(rng: &mut R) -> Vector {
    let mut arr = [0.; 3];
    arr.fill_with(|| rng.gen_range(0.0..=1.0));
    arr.into()
}

/// Returns a random vector in a unit sphere (`-1..=1`, `length = 1`)
pub fn vector_in_unit_sphere<R: Rng>(rng: &mut R) -> Vector {
    loop {
        let v = vector_in_unit_cube(rng);
        if v.length_squared() <= 1. {
            break v;
        }
    }
}

/// Returns a random vector in a unit hemisphere (`-1..=1`, `length = 1`)
/// The output vector is guaranteed to point in the same hemisphere as the normal,
/// i.e. `dot(vec, normal) >= 0.0`
pub fn vector_in_unit_hemisphere<R: Rng>(rng: &mut R, normal: Vector) -> Vector {
    let vec = vector_in_unit_sphere(rng);
    // pointing same side as normal
    if Vector::dot(vec, normal) >= 0. {
        vec
    } else {
        -vec
    }
}

/// Returns a random vector on a unit hemisphere (`-1..=1`, `length = 1`)
/// The output vector is guaranteed to point in the same hemisphere as the normal,
/// i.e. `dot(vec, normal) >= 0.0`, and have a unit length (
pub fn vector_on_unit_hemisphere<R: Rng>(rng: &mut R, normal: Vector) -> Vector {
    loop {
        let Some(vec) = vector_in_unit_sphere(rng).try_normalize() else {
            continue;
        };
        // pointing same side as normal
        if Vector::dot(vec, normal) >= 0. {
            break vec;
        } else {
            break -vec;
        }
    }
}
