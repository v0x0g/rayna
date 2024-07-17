use rand_core::RngCore;

use crate::core::types::Colour;

use crate::shared::intersect::MeshIntersection;
use crate::texture::{Texture, TextureInstance};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SolidTexture {
    pub albedo: Colour,
}

impl<T: Into<Colour>> From<T> for SolidTexture {
    fn from(value: T) -> Self { Self { albedo: value.into() } }
}

impl<T: Into<Colour>> From<T> for TextureInstance {
    fn from(value: T) -> Self { SolidTexture::from(value).into() }
}

impl Default for SolidTexture {
    fn default() -> Self { Colour::BLACK.into() }
}

impl Texture for SolidTexture {
    fn value(&self, _intersection: &MeshIntersection, _rng: &mut dyn RngCore) -> Colour { self.albedo }
}
