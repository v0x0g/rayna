use rayna_shared::def::types::Number;
use rayna_shared::def::types::Pix;

pub fn lerp(a: Pix, b: Pix, t: Number) -> Pix {
    glam::DVec3::lerp(
        glam::DVec3::from(a.0.map(Number::from)),
        glam::DVec3::from(b.0.map(Number::from)),
        t,
    )
    .as_vec3()
    .to_array()
    .into()
}
