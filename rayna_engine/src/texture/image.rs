use crate::shared::intersect::Intersection;
use crate::texture::Texture;
use derivative::Derivative;
use num_traits::ToPrimitive;
use rand_core::RngCore;
use rayna_shared::def::types::{ImgBuf, Number, Pixel, Size2, Vector2};
use std::sync::Arc;

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct ImageTexture {
    #[derivative(Debug = "ignore")]
    pub image: Arc<ImgBuf>,
    pub scale: Size2,
    pub offset: Vector2,
}

impl Texture for ImageTexture {
    fn value(&self, intersection: &Intersection, _rng: &mut dyn RngCore) -> Pixel {
        // Calculate pixel positions after scale and offset
        let translated = self.offset + (intersection.uv.to_vector() * self.scale.to_vector());
        // Flip y-axis to image coords
        let (u, v) = (translated.x, 1. - translated.y);

        // Don't need bounds check on uv coords, should be valid already
        // Don't need to
        let Some(i) = (u * self.image.width() as Number).to_u32() else {
            return super::texture_error_value();
        };
        let Some(j) = (v * self.image.height() as Number).to_u32() else {
            return super::texture_error_value();
        };

        // Should never be out of bounds
        self.image[(i, j)]
    }
}