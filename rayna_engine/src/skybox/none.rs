use crate::shared::ray::Ray;
use crate::skybox::Skybox;
use rayna_shared::def::types::Colour;

#[derive(Copy, Clone, Debug, Default)]
pub struct NoSkybox;

impl Skybox for NoSkybox {
    fn sky_colour(&self, _ray: &Ray) -> Colour {
        const BLACK: Colour = Colour { 0: [0.; 3] };
        BLACK
    }
}
