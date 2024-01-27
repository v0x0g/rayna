use std::ops::{Add, Mul, Sub};

use crate::core::types::{Channel, Colour, Number, Vector3};

pub trait Lerp<Frac>: num_traits::NumOps<Frac> + Sized {
    fn lerp(a: Self, b: Self, t: Frac) -> Self { a + (b - a) * t }
}

impl<Frac, T: num_traits::NumOps<Frac> + Sized> Lerp<Frac> for T {}

/// Calculates the vector reflection of vector `d` across the surface normal `n`
pub fn reflect(d: Vector3, n: Vector3) -> Vector3 { d - n * (2. * d.dot(n)) }

pub fn refract(vec: Vector3, n: Vector3, ir_ratio: Number) -> Vector3 {
    let cos_theta = Vector3::dot(-vec, n).min(1.);
    let r_out_perp = (vec + n * cos_theta) * ir_ratio;
    let r_out_parallel = n * -Number::sqrt(Number::abs(1.0 - r_out_perp.length_squared()));
    return r_out_perp + r_out_parallel;
}
