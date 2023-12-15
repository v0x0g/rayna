pub mod default_skybox;

use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use rayna_shared::def::types::Pixel;

dyn_clone::clone_trait_object!(Skybox);
pub trait Skybox: RtRequirement {
    fn sky_colour(&self, ray: Ray) -> Pixel;
}
