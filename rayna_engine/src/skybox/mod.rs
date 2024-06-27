pub mod dynamic;
pub mod hdri;
pub mod none;
pub mod simple;

use self::{
    dynamic::DynamicSkybox,
    hdri::HdrImageSkybox,
    none::NoSkybox,
    simple::{SimpleSkybox, WhiteSkybox},
};
use crate::core::types::Colour;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use enum_dispatch::enum_dispatch;

/// The main trait for implementing a skybox
///
/// This simply needs to return the sky colour for a given ray
#[enum_dispatch]
#[doc(notable_trait)]
pub trait Skybox: RtRequirement {
    fn sky_colour(&self, ray: &Ray) -> Colour;
}

#[enum_dispatch(Skybox)]
#[derive(Clone, Debug)]
pub enum SkyboxInstance {
    SimpleSkybox,
    WhiteSkybox,
    NoSkybox,
    DynamicSkybox,
    HdrImageSkybox,
}

impl Default for SkyboxInstance {
    fn default() -> Self { SimpleSkybox::default().into() }
}

/// This allows us to use [Option::None] as shorthand for no skybox
impl From<Option<SkyboxInstance>> for SkyboxInstance {
    fn from(value: Option<SkyboxInstance>) -> Self { value.unwrap_or(Self::NoSkybox(NoSkybox {})) }
}
