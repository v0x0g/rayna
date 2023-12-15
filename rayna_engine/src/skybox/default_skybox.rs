use crate::shared::ray::Ray;
use crate::shared::{math, RtRequirement};
use crate::skybox::Skybox;
use rayna_shared::def::types::Pixel;

#[derive(Copy, Clone, Debug, Default)]
pub struct DefaultSkybox;

impl RtRequirement for DefaultSkybox {}

impl Skybox for DefaultSkybox {
    fn sky_colour(&self, ray: Ray) -> Pixel {
        let a = (0.5 * ray.dir().y) + 0.5;

        let white = Pixel::from([1., 1., 1.]);
        let blue = Pixel::from([0.5, 0.7, 1.]);

        math::lerp(white, blue, a)
    }
}
