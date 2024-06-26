pub mod checker;
pub mod dynamic;
pub mod image;
pub mod noise;
pub mod solid;

use crate::core::types::Colour;
use crate::shared::intersect::Intersection;
use crate::shared::RtRequirement;
use enum_dispatch::enum_dispatch;
use rand::thread_rng;
use rand_core::RngCore;
//noinspection ALL
use self::{
    checker::{UvCheckerTexture, WorldCheckerTexture},
    dynamic::DynamicTexture,
    image::ImageTexture,
    noise::{LocalNoiseTexture, UvNoiseTexture, WorldNoiseTexture},
    solid::SolidTexture,
};

/// The trait that defines what properties a texture has
#[enum_dispatch]
#[doc(notable_trait)]
pub trait Texture: RtRequirement {
    fn value(&self, intersection: &Intersection, rng: &mut dyn RngCore) -> Colour;
}

/// An optimised implementation of [Texture], using static dispatch
#[enum_dispatch(Texture)]
#[derive(Clone, Debug)]
pub enum TextureInstance {
    SolidTexture,
    WorldCheckerTexture(WorldCheckerTexture<DynamicTexture, DynamicTexture>),
    UvCheckerTexture(UvCheckerTexture<DynamicTexture, DynamicTexture>),
    ImageTexture,
    UvNoiseTexture(UvNoiseTexture<Box<dyn noise::RtNoiseFn<2>>>),
    LocalNoiseTexture(LocalNoiseTexture<Box<dyn noise::RtNoiseFn<3>>>),
    WorldNoiseTexture(WorldNoiseTexture<Box<dyn noise::RtNoiseFn<3>>>),
    DynamicTexture,
}

impl Default for TextureInstance {
    fn default() -> Self { SolidTexture::default().into() }
}

/// Special function to be called when an error occurs during texture value calculations,
/// and a value cannot be generated. Calling this has an advantage over panicking since it won't crash anything,
/// and it'll also allow breakpoints to be set to debug the problem.
#[cold]
pub fn texture_error_value() -> Colour { crate::shared::rng::colour_rgb(&mut thread_rng()) }
