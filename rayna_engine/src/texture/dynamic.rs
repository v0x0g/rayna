use crate::shared::intersect::Intersection;
use crate::texture::Texture;
use rand_core::RngCore;
use rayna_shared::def::types::Pixel;

#[derive(Clone, Debug)]
pub struct DynamicTexture {
    pub inner: Box<dyn Texture>,
}

impl Texture for DynamicTexture {
    fn value(&self, intersection: &Intersection, rng: &mut dyn RngCore) -> Pixel {
        self.inner.value(intersection, rng)
    }
}
