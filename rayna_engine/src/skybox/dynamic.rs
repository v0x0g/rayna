use crate::core::types::Colour;
use crate::shared::ray::Ray;
use crate::skybox::Skybox;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DynamicSkybox {
    pub inner: Arc<dyn Skybox>,
}

impl Skybox for DynamicSkybox {
    fn sky_colour(&self, ray: &Ray) -> Colour { self.inner.sky_colour(ray) }
}
