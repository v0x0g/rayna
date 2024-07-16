use crate::core::types::{Colour, Image, Number, Transform2};
use crate::shared::intersect::Intersection;
use crate::texture::Texture;
use glamour::TransformMap;
use rand_core::RngCore;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ImageTexture {
    pub image: Image,
    pub transform: Transform2,
}

impl From<Image> for ImageTexture {
    fn from(value: Image) -> Self { Self::from(Arc::new(value)) }
}

impl From<Image> for ImageTexture {
    fn from(value: Image) -> Self {
        Self {
            image: value,
            transform: Transform2::IDENTITY,
        }
    }
}

// TODO: Implement some sort of texture filtering and stuff
impl Texture for ImageTexture {
    fn value(&self, intersection: &Intersection, _rng: &mut dyn RngCore) -> Colour {
        // Calculate pixel positions after scale and offset
        let translated = self.transform.map(intersection.uv);
        // Flip y-axis to image coords
        let (u, v) = (translated.x, 1. - translated.y);

        let i = u * self.image.width() as Number;
        let j = v * self.image.height() as Number;
        self.image.get_bilinear(i, j)
    }
}
