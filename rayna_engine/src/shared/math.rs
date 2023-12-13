use crate::def::types::Num;
use crate::def::types::Pix;
use puffin::profile_function;

pub fn lerp(a: Pix, b: Pix, t: Num) -> Pix {
    profile_function!();

    glam::DVec3::lerp(
        glam::DVec3::from(a.0.map(Num::from)),
        glam::DVec3::from(b.0.map(Num::from)),
        t,
    )
    .as_vec3()
    .to_array()
    .into()
}
