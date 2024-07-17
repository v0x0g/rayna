pub mod hdri;
pub mod none;
pub mod simple;

use crate::core::types::Colour;
use crate::shared::ray::Ray;
use crate::shared::ComponentRequirements;
use enum_dispatch::enum_dispatch;

/// The main trait for implementing a skybox
///
/// This simply needs to return the sky colour for a given ray
#[enum_dispatch]
#[doc(notable_trait)]
pub trait Skybox: ComponentRequirements {
    fn sky_colour(&self, ray: &Ray) -> Colour;
}

#[enum_dispatch(Skybox)]
#[derive(Clone, Debug, Default)]
pub enum SkyboxInstance {
    #[default]
    SimpleSkybox(self::simple::SimpleSkybox),
    WhiteSkybox(self::simple::WhiteSkybox),
    NoSkybox(self::none::NoSkybox),
    HdrImageSkybox(self::hdri::HdrImageSkybox),
}

/// This allows us to use [Option::None] as shorthand for no skybox
impl From<Option<SkyboxInstance>> for SkyboxInstance {
    fn from(value: Option<SkyboxInstance>) -> Self { value.unwrap_or(Self::NoSkybox(Default::default())) }
}
