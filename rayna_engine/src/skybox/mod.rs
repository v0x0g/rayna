pub mod default_skybox;

use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use crate::skybox::default_skybox::DefaultSkybox;
use rayna_shared::def::types::Pixel;
use std::sync::Arc;

dyn_clone::clone_trait_object!(Skybox);
pub trait Skybox: RtRequirement {
    fn sky_colour(&self, ray: &Ray) -> Pixel;
}

#[derive(Clone, Debug)]
pub enum SkyboxType {
    Default(DefaultSkybox),
    Other(Arc<dyn Skybox>),
}

impl Skybox for SkyboxType {
    fn sky_colour(&self, ray: &Ray) -> Pixel {
        match self {
            Self::Default(sky) => sky.sky_colour(ray),
            Self::Other(sky) => sky.sky_colour(ray),
        }
    }
}

impl Default for SkyboxType {
    fn default() -> Self {
        Self::Default(DefaultSkybox::default())
    }
}
