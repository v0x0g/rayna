use crate::core::types::{Channel, Colour};
use crate::shared::math::Lerp;
use crate::shared::ray::Ray;
use crate::skybox::Skybox;

/// A skybox that mixes between blue and white, depending on pitch
///
/// Fades to blue at the top, white at the bottom
#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleSkybox;

impl Skybox for SimpleSkybox {
    fn sky_colour(&self, ray: &Ray) -> Colour {
        let a = (0.5 * ray.dir().y) + 0.5;

        let white = Colour::from([1., 1., 1.]);
        let blue = Colour::from([0.5, 0.7, 1.]);

        // TODO: Come back once `Colour: Lerp<Number>`
        Colour::lerp(white, blue, a as Channel)
    }
}

/// An all-white skybox, uniform everywhere
#[derive(Copy, Clone, Debug, Default)]
pub struct WhiteSkybox;

impl Skybox for WhiteSkybox {
    fn sky_colour(&self, _ray: &Ray) -> Colour { Colour::WHITE }
}
