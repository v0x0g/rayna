use crate::shared::math;
use crate::shared::ray::Ray;
use crate::skybox::Skybox;
use rayna_engine::core::types::Colour;

#[derive(Copy, Clone, Debug, Default)]
pub struct DefaultSkybox;

impl Skybox for DefaultSkybox {
    fn sky_colour(&self, ray: &Ray) -> Colour {
        let a = (0.5 * ray.dir().y) + 0.5;

        let white = Colour::from([1., 1., 1.]);
        let blue = Colour::from([0.5, 0.7, 1.]);

        math::lerp_px(white, blue, a)
    }
}
