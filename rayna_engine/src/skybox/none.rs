use crate::shared::ray::Ray;
use crate::skybox::Skybox;
use rayna_shared::def::types::Pixel;

#[derive(Copy, Clone, Debug, Default)]
pub struct NoSkybox;

impl Skybox for NoSkybox {
    fn sky_colour(&self, _ray: &Ray) -> Pixel {
        const BLACK: Pixel = Pixel { 0: [0.; 3] };
        BLACK
    }
}
