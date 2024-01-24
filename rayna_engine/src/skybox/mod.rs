pub mod default;
pub mod dynamic;
pub mod none;

use self::{default::DefaultSkybox, dynamic::DynamicSkybox, none::NoSkybox};
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use enum_dispatch::enum_dispatch;
use rayna_engine::core::types::Colour;

/// The main trait for implementing a skybox
///
/// This simply needs to return the sky colour for a given ray
#[enum_dispatch]
pub trait Skybox: RtRequirement {
    fn sky_colour(&self, ray: &Ray) -> Colour;
}

#[enum_dispatch(Skybox)]
#[derive(Clone, Debug)]
pub enum SkyboxInstance {
    DefaultSkybox,
    NoSkybox,
    DynamicSkybox,
}

impl Default for SkyboxInstance {
    fn default() -> Self { DefaultSkybox::default().into() }
}

/// This allows us to use [Option::None] as shorthand for no skybox
impl From<Option<SkyboxInstance>> for SkyboxInstance {
    fn from(value: Option<SkyboxInstance>) -> Self { value.unwrap_or(Self::NoSkybox(NoSkybox {})) }
}
