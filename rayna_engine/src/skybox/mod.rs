pub mod default;
pub mod dynamic;

use self::{default::DefaultSkybox, dynamic::DynamicSkybox};
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use enum_dispatch::enum_dispatch;
use rayna_shared::def::types::Pixel;

dyn_clone::clone_trait_object!(Skybox);
#[enum_dispatch]
pub trait Skybox: RtRequirement {
    fn sky_colour(&self, ray: &Ray) -> Pixel;
}

#[enum_dispatch(Skybox)]
#[derive(Clone, Debug)]
pub enum SkyboxInstance {
    DefaultSkybox,
    DynamicSkybox,
}

impl Default for SkyboxInstance {
    fn default() -> Self {
        DefaultSkybox::default().into()
    }
}
