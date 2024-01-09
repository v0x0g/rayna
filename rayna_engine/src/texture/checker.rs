use std::sync::Arc;
use crate::shared::intersect::Intersection;
use crate::texture::{Texture, TextureInstance};
use num_integer::Integer;
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Pixel, Vector3};

#[derive(Clone, Debug)]
pub struct WorldCheckerTexture {
    pub offset: Vector3,
    // TODO: Find if there's a way to remove the Arc<> wrapper without having cycles in the type hierarchy
    pub even: Arc<TextureInstance>,
    pub odd: Arc<TextureInstance>,
    pub scale: Number,
}

impl Texture for WorldCheckerTexture {
    fn value(&self, intersection: &Intersection, rng: &mut dyn RngCore) -> Pixel {
        let pos = intersection.pos_w.to_vector();
        let Some(coords) = (pos / self.scale).floor().try_cast::<u64>() else {
            return super::texture_error_value();
        };
        let sum = coords.into_iter().fold(0, u64::wrapping_add);

        if sum.is_even() {
            self.even.value(intersection, rng)
        } else {
            self.odd.value(intersection, rng)
        }
    }
}
