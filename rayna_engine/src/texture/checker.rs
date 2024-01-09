use std::sync::Arc;

use num_integer::Integer;
use num_traits::ToPrimitive;
use rand_core::RngCore;

use rayna_shared::def::types::{Number, Pixel, Vector3};

use crate::shared::intersect::Intersection;
use crate::texture::{Texture, TextureInstance};

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
        let floor = (pos / self.scale).floor();
        // Use i128 for greatest range and lowest change of cast failing
        let Some(coords) = floor.as_array().try_map(|n| n.to_i128()) else {
            return super::texture_error_value();
        };
        let sum = coords.into_iter().fold(0, i128::wrapping_add);

        if sum.is_even() {
            self.even.value(intersection, rng)
        } else {
            self.odd.value(intersection, rng)
        }
    }
}
