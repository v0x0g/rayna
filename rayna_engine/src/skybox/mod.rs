pub mod default;
pub mod dynamic;

use self::{default::DefaultSkybox, dynamic::DynamicSkybox};
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use enum_dispatch::enum_dispatch;
use rayna_shared::def::types::Pixel;

dyn_clone::clone_trait_object!(Skybox);
#[enum_dispatch(SkyboxType)]
pub trait Skybox: RtRequirement {
    fn sky_colour(&self, ray: &Ray) -> Pixel;
}

#[enum_dispatch]
#[derive(Clone, Debug)]
pub enum SkyboxType {
    DefaultSkybox,
    DynamicSkybox,
}

impl Default for SkyboxType {
    fn default() -> Self {
        DefaultSkybox::default().into()
    }
}
