use rayna_shared::def::types::Number;
use rayna_shared::def::types::Pixel;

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
