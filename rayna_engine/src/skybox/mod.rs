pub mod hdri;
pub mod none;
pub mod simple;

use crate::core::component::Component;
use crate::core::ray::Ray;
use crate::core::types::Colour;
use derive_where::derive_where;
use enum_dispatch::enum_dispatch;

/// The main trait for implementing a skybox
///
/// This simply needs to return the sky colour for a given ray
#[enum_dispatch]
#[doc(notable_trait)]
pub trait Skybox: Component {
    fn sky_colour(&self, ray: &Ray) -> Colour;
}

#[enum_dispatch(Skybox)]
#[derive(Clone, Debug)]
#[derive_where(Default)]
pub enum SkyboxInstance {
    #[derive_where(default)]
    SimpleSkybox(self::simple::SimpleSkybox),
    WhiteSkybox(self::simple::WhiteSkybox),
    NoSkybox(self::none::NoSkybox),
    HdrImageSkybox(self::hdri::HdrImageSkybox),
}

/// This allows us to use [Option::None] as shorthand for no skybox
impl From<Option<SkyboxInstance>> for SkyboxInstance {
    fn from(value: Option<SkyboxInstance>) -> Self { value.unwrap_or(Self::NoSkybox(Default::default())) }
}
