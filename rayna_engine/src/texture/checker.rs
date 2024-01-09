use crate::shared::intersect::Intersection;
use crate::texture::{Texture, TextureInstance};
use derivative::Derivative;
use num_integer::Integer;
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Pixel, Vector3};

#[derive(Derivative)]
#[derivative(Copy, Clone, Debug(bound = ""))]
pub struct WorldCheckerTexture<
    A: Texture + Clone = TextureInstance,
    B: Texture + Clone = TextureInstance,
> {
    pub offset: Vector3,
    pub even: A,
    pub odd: B,
    pub scale: Number,
}

impl<A: Texture + Clone, B: Texture + Clone> Texture for WorldCheckerTexture<A, B> {
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
