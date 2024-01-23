use std::ops::{Add, Mul, Sub};

use rayna_shared::def::types::{Channel, Colour, Number, Vector3};

/// Your standard linear interpolation function
pub fn lerp<T>(a: T, b: T, t: Number) -> T
where
    T: Add<Output = T> + Mul<Number, Output = T> + Sub<Output = T> + Clone,
{
    a.clone() + ((b.clone() - a.clone()) * t)
}

pub fn lerp_px(a: Colour, b: Colour, t: Number) -> Colour {
    let [ra, ga, ba] = a.0;
    let [rb, gb, bb] = b.0;
    Colour::from([
        lerp(ra as Number, rb as Number, t) as Channel,
        lerp(ga as Number, gb as Number, t) as Channel,
        lerp(ba as Number, bb as Number, t) as Channel,
    ])
}

/// Calculates the vector reflection of vector `d` across the surface normal `n`
pub fn reflect(d: Vector3, n: Vector3) -> Vector3 { d - n * (2. * d.dot(n)) }

pub fn refract(vec: Vector3, n: Vector3, ir_ratio: Number) -> Vector3 {
    let cos_theta = Vector3::dot(-vec, n).min(1.);
    let r_out_perp = (vec + n * cos_theta) * ir_ratio;
    let r_out_parallel = n * -Number::sqrt(Number::abs(1.0 - r_out_perp.length_squared()));
    return r_out_perp + r_out_parallel;
}
