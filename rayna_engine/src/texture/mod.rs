pub mod checker;
pub mod image;
pub mod solid;

use crate::core::types::Colour;
use crate::scene::Scene;
use crate::shared::intersect::MeshIntersection;
use crate::shared::token::generate_component_token;
use crate::shared::ComponentRequirements;
use enum_dispatch::enum_dispatch;
use rand::thread_rng;
use rand_core::RngCore;

/// The trait that defines what properties a texture has
#[enum_dispatch]
#[doc(notable_trait)]
pub trait Texture: ComponentRequirements {
    fn value(&self, scene: &Scene, intersection: &MeshIntersection, rng: &mut dyn RngCore) -> Colour;
}

/// An optimised implementation of [Texture], using static dispatch
#[enum_dispatch(Texture)]
#[derive(Clone, Debug, Default)]
pub enum TextureInstance {
    #[default]
    SolidTexture(self::solid::SolidTexture),
    WorldCheckerTexture(self::checker::WorldCheckerTexture),
    UvCheckerTexture(self::checker::UvCheckerTexture),
    ImageTexture(self::image::ImageTexture),
}

generate_component_token!(TextureToken for TextureInstance);

/// Special function to be called when an error occurs during texture value calculations,
/// and a value cannot be generated. Calling this has an advantage over panicking since it won't crash anything,
/// and it'll also allow breakpoints to be set to debug the problem.
#[cold]
pub fn texture_error_value() -> Colour { crate::shared::rng::colour_rgb(&mut thread_rng()) }
