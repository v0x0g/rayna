pub mod checker;
pub mod image;
mod noise;
pub mod solid;

use crate::core::component::Component;
use crate::core::intersect::MeshIntersection;
use crate::core::token::generate_component_token;
use crate::core::types::Colour;
use crate::scene::Scene;
use enum_dispatch::enum_dispatch;
use rand::thread_rng;
use rand_core::RngCore;

/// The trait that defines what properties a texture has
#[enum_dispatch]
#[doc(notable_trait)]
pub trait Texture: Component {
    fn value(&self, scene: &Scene, intersection: &MeshIntersection, rng: &mut dyn RngCore) -> Colour;
}

/// An optimised implementation of [Texture], using static dispatch
#[enum_dispatch(Texture)]
#[derive(Clone, Debug)]
pub enum TextureInstance {
    SolidTexture(self::solid::SolidTexture),
    WorldCheckerTexture(self::checker::WorldCheckerTexture),
    UvCheckerTexture(self::checker::UvCheckerTexture),
    ImageTexture(self::image::ImageTexture),
}

impl Default for TextureInstance {
    fn default() -> Self { Self::SolidTexture(Default::default()) }
}

generate_component_token!(TextureToken for TextureInstance);

/// Special function to be called when an error occurs during texture value calculations,
/// and a value cannot be generated. Calling this has an advantage over panicking since it won't crash anything,
/// and it'll also allow breakpoints to be set to debug the problem.
pub fn texture_error_value() -> Colour { crate::core::rng::colour_rgb(&mut thread_rng()) }
