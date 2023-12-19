use rayna_shared::def::types::Pixel;
use rayna_shared::def::types::{Number, Vector3};

/// Your standard linear interpolation function
pub fn lerp(a: Pixel, b: Pixel, t: Number) -> Pixel {
    glam::DVec3::lerp(
        glam::DVec3::from(a.0.map(Number::from)),
        glam::DVec3::from(b.0.map(Number::from)),
        t,
    )
    .as_vec3()
    .to_array()
    .into()
}

/// Calculates the vector reflection of vector `d` across the surface normal `n`
pub fn reflect(d: Vector3, n: Vector3) -> Vector3 {
    d - n * (2. * d.dot(n))
}
