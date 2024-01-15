use crate::shared::intersect::Intersection;
use crate::texture::Texture;
use rand_core::RngCore;
use rayna_shared::def::types::Pixel;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DynamicTexture {
    pub inner: Arc<dyn Texture>,
}

impl DynamicTexture {
    pub fn new(inner: impl Texture) -> Self { Self { inner: Arc::new(inner) } }
}

impl Texture for DynamicTexture {
    fn value(&self, intersection: &Intersection, rng: &mut dyn RngCore) -> Pixel { self.inner.value(intersection, rng) }
}
