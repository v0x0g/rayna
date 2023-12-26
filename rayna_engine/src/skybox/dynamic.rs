use crate::shared::ray::Ray;
use crate::skybox::Skybox;
use rayna_shared::def::types::Pixel;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DynamicSkybox {
    pub inner: Arc<dyn Skybox>,
}

impl Skybox for DynamicSkybox {
    fn sky_colour(&self, ray: &Ray) -> Pixel {
        self.inner.sky_colour(ray)
    }
}
