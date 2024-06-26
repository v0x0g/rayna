//! Helper module for RNG-related functions

use crate::core::types::{Channel, Colour, Number, Vector2, Vector3};
use glamour::AngleConsts;

use crate::shared::validate;
use rand::distributions::uniform::SampleRange;
use rand::Rng;
use rand_core::SeedableRng;

const PI: Number = <Number as AngleConsts>::PI;

// TODO: Rework this random a little
//  - Make it less repetitive (extract to trait?)
//  - Use inverse CDF's to get rid of the `loop`s

/// A struct that can be used in [opool] to allocate RNGs
/// using the [SeedableRng::from_entropy] method
#[derive(Copy, Clone, Debug, Default)]
pub struct RngPoolAllocator;
impl<R: SeedableRng> opool::PoolAllocator<R> for RngPoolAllocator {
    fn allocate(&self) -> R { R::from_entropy() }
}

// region 1D

/// Returns a number in the range `-1.0..1.0`
pub fn number_in_unit_line<R: Rng + ?Sized>(rng: &mut R) -> Number {
    let n = rng.gen_range(-1.0..=1.0);
    validate::number(&n);
    n
}

/// Returns a number in the range `0.0..1.0`
pub fn number_in_unit_line_01<R: Rng + ?Sized>(rng: &mut R) -> Number {
    let n = rng.gen_range(0.0..=1.0);
    validate::number(&n);
    n
}

// endregion 1D

// region 3D

/// Returns a random vector in a unit cube (-1..=1)
pub fn vector_in_unit_cube<R: Rng + ?Sized>(rng: &mut R) -> Vector3 {
    let mut arr = [0.; 3];
    arr.fill_with(|| rng.gen_range(-1.0..=1.0));
    let v = arr.into();
    validate::vector3(&v);
    v
}

/// Returns a random vector in a positive-only unit cube (`0..=1`)
pub fn vector_in_unit_cube_01<R: Rng + ?Sized>(rng: &mut R) -> Vector3 {
    let mut arr = [0.; 3];
    arr.fill_with(|| rng.gen_range(0.0..=1.0));
    let v = arr.into();
    validate::vector3(&v);
    v
}

/// Returns a random vector in a unit sphere (`-1..=1`, `length <= 1`)
pub fn vector_in_unit_sphere<R: Rng + ?Sized>(rng: &mut R) -> Vector3 {
    loop {
        let v = vector_in_unit_cube(rng);
        validate::vector3(&v);
        if v.length_squared() <= 1. {
            break v;
        }
    }
}

/// Returns a random normalised vector on a unit sphere (`-1..=1`, `length = 1`)
pub fn normal_on_unit_sphere<R: Rng + ?Sized>(rng: &mut R) -> Vector3 {
    // Adapted from good 'ol Pete
    let r1 = number_in_unit_line_01(rng);
    let r2 = number_in_unit_line_01(rng);

    let x = Number::cos(2. * PI * r1) * 2. * Number::sqrt(r2 * (1. - r2));
    let y = Number::sin(2. * PI * r1) * 2. * Number::sqrt(r2 * (1. - r2));
    let z = 1. - 2. * r2;

    let v = (x, y, z).into();
    validate::normal3(&v);
    v
}

/// Returns a random vector in a unit hemisphere (`-1..=1`)
/// The output vector is guaranteed to point in the same hemisphere as the normal,
/// i.e. `dot(vec, normal) >= 0.0`
pub fn vector_in_unit_hemisphere<R: Rng + ?Sized>(rng: &mut R, normal: Vector3) -> Vector3 {
    let vec = vector_in_unit_sphere(rng);
    validate::vector3(&vec);
    vec * Vector3::dot(vec, normal).signum()
}

/// Returns a random vector on a unit hemisphere (`-1..=1`, `length = 1`)
/// The output vector is guaranteed to point in the same hemisphere as the normal,
/// i.e. `dot(vec, normal) >= 0.0`, and have a unit length (
pub fn normal_on_unit_hemisphere<R: Rng + ?Sized>(rng: &mut R, normal: Vector3) -> Vector3 {
    loop {
        let Some(vec) = vector_in_unit_sphere(rng).try_normalize() else {
            continue;
        };
        validate::normal3(&vec);
        break vec * Vector3::dot(vec, normal).signum();
    }
}

// endregion 3D

// region 2D

/// Returns a random vector in a unit square (-1..=1)
pub fn vector_in_unit_square<R: Rng + ?Sized>(rng: &mut R) -> Vector2 {
    let mut arr = [0.; 2];
    arr.fill_with(|| rng.gen_range(-1.0..=1.0));
    let v = arr.into();
    validate::vector2(&v);
    v
}

/// Returns a random vector in a positive-only unit square (`0..=1`)
pub fn vector_in_unit_square_01<R: Rng + ?Sized>(rng: &mut R) -> Vector2 {
    let mut arr = [0.; 2];
    arr.fill_with(|| rng.gen_range(0.0..=1.0));
    let v = arr.into();
    validate::vector2(&v);
    v
}

/// Returns a random vector in a unit circle (`-1..=1`, `length <= 1`)
pub fn vector_in_unit_circle<R: Rng + ?Sized>(rng: &mut R) -> Vector2 {
    loop {
        let v = vector_in_unit_square(rng);
        validate::vector2(&v);
        if v.length_squared() <= 1. {
            return v;
        }
    }
}
/// Returns a random vector on a unit circle (`-1..=1`, `length = 1`)
pub fn normal_on_unit_circle<R: Rng + ?Sized>(rng: &mut R) -> Vector2 {
    loop {
        let Some(vec) = vector_in_unit_circle(rng).try_normalize() else {
            continue;
        };
        validate::normal2(&vec);
        return vec;
    }
}

/// Returns a random vector in a unit semicircle (`-1..=1`, `length = 1`)
/// The output vector is guaranteed to point in the same semicircle as the normal,
/// i.e. `dot(vec, normal) >= 0.0`
pub fn vector_in_unit_semicircle<R: Rng + ?Sized>(rng: &mut R, normal: Vector2) -> Vector2 {
    let vec = vector_in_unit_circle(rng);
    validate::vector2(&vec);
    // pointing same side as normal
    vec * Vector2::dot(vec, normal).signum()
}

/// Returns a random vector on a unit semicircle (`-1..=1`, `length = 1`)
/// The output vector is guaranteed to point in the same semicircle as the normal,
/// i.e. `dot(vec, normal) >= 0.0`, and have a unit length (
pub fn normal_on_unit_semicircle<R: Rng + ?Sized>(rng: &mut R, normal: Vector2) -> Vector2 {
    loop {
        let Some(vec) = vector_in_unit_circle(rng).try_normalize() else {
            continue;
        };
        // pointing same side as normal
        return vec * Vector2::dot(vec, normal).signum();
    }
}

//endregion 2D

// region Colours

/// Returns a random RGB colour
pub fn colour_rgb<R: Rng + ?Sized>(rng: &mut R) -> Colour {
    let mut arr: [Channel; Colour::CHANNEL_COUNT] = Default::default();
    arr.fill_with(|| rng.gen_range(0.0..=1.0));
    let c = arr.into();
    validate::colour(&c);
    c
}
/// Returns a random RGB colour with a given range for the channels
pub fn colour_rgb_range<R: Rng + ?Sized, Ra: SampleRange<Channel> + Clone>(rng: &mut R, range: Ra) -> Colour {
    let mut arr: [Channel; Colour::CHANNEL_COUNT] = Default::default();
    arr.fill_with(|| rng.gen_range(range.clone()));

    let c = arr.into();
    validate::colour(&c);
    c
}

/// Returns a random black and white colour
pub fn colour_bw<R: Rng + ?Sized>(rng: &mut R) -> Colour {
    let val = rng.gen_range(0.0..=1.0);
    let c = [val; Colour::CHANNEL_COUNT].into();
    validate::colour(&c);
    c
}
/// Returns a random black and white colour with a given range for the channels
pub fn colour_bw_range<R: Rng + ?Sized, Ra: SampleRange<Channel>>(rng: &mut R, range: Ra) -> Colour {
    let val = rng.gen_range(range);
    let c = [val; Colour::CHANNEL_COUNT].into();
    validate::colour(&c);
    c
}

// endregion
