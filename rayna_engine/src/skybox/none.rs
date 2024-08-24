use crate::core::ray::Ray;
use crate::core::types::Colour;
use crate::skybox::Skybox;

#[derive(Copy, Clone, Debug, Default)]
pub struct NoSkybox;

impl Skybox for NoSkybox {
    fn sky_colour(&self, _ray: &Ray) -> Colour {
        const BLACK: Colour = Colour { 0: [0.; 3] };
        BLACK
    }
}
