use rayna_shared::def::types::Num;
use rayna_shared::def::types::Pix;

pub fn lerp(a: Pix, b: Pix, t: Num) -> Pix {
    glam::DVec3::lerp(
        glam::DVec3::from(a.0.map(Num::from)),
        glam::DVec3::from(b.0.map(Num::from)),
        t,
    )
    .as_vec3()
    .to_array()
    .into()
}
