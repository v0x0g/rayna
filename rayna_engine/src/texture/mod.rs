pub mod dynamic;
mod solid;

use crate::shared::intersect::Intersection;
use crate::shared::RtRequirement;
use enum_dispatch::enum_dispatch;
use rand_core::RngCore;
use rayna_shared::def::types::Pixel;
//noinspection ALL
use self::dynamic::DynamicTexture;

/// The trait that defines what properties a texture has
#[enum_dispatch]
pub trait Texture: RtRequirement {
    fn value(&self, intersection: &Intersection, rng: &mut dyn RngCore) -> Pixel;
}

dyn_clone::clone_trait_object!(Texture);

/// An optimised implementation of [Texture], using static dispatch
#[enum_dispatch(Texture)]
#[derive(Clone, Debug)]
pub enum TextureType {
    DynamicTexture,
}
