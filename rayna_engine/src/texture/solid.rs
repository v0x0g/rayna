use crate::shared::intersect::Intersection;
use crate::texture::Texture;
use rand_core::RngCore;
use rayna_shared::def::types::Pixel;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SolidTexture {
    pub albedo: Pixel,
}

impl Texture for SolidTexture {
    fn value(&self, _intersection: &Intersection, _rng: &mut dyn RngCore) -> Pixel {
        self.albedo
    }
}
