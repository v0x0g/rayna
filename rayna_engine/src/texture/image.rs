use crate::core::types::{Colour, Image, Number, Size2, Vector2};
use crate::shared::intersect::Intersection;
use crate::texture::Texture;
use num_traits::ToPrimitive;
use rand_core::RngCore;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ImageTexture {
    pub image: Arc<Image>,
    pub scale: Size2,
    pub offset: Vector2,
}

impl From<Image> for ImageTexture {
    fn from(value: Image) -> Self { Self::from(Arc::new(value)) }
}

impl From<Arc<Image>> for ImageTexture {
    fn from(value: Arc<Image>) -> Self {
        Self {
            offset: Vector2::ZERO,
            scale: Size2::splat(1.),
            image: value,
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, intersection: &Intersection, _rng: &mut dyn RngCore) -> Colour {
        // Calculate pixel positions after scale and offset
        let translated = self.offset + (intersection.uv.to_vector() * self.scale.to_vector());
        // Flip y-axis to image coords
        let (u, v) = (translated.x, 1. - translated.y);

        // Don't need bounds check on uv coords, should be valid already
        // Don't need to
        let Some(i) = (u * self.image.width() as Number).to_usize() else {
            return super::texture_error_value();
        };
        let Some(j) = (v * self.image.height() as Number).to_usize() else {
            return super::texture_error_value();
        };

        // Should never be out of bounds
        self.image[(i, j)]
    }
}
