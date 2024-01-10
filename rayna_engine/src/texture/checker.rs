use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use num_traits::float::FloatCore;
use num_traits::Euclid;
use rand_core::RngCore;

use rayna_shared::def::types::{Number, Pixel, Vector2, Vector3};

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
        let pos = (intersection.pos_w.to_vector() / self.scale) + self.offset;

        do_checker(
            pos.to_array(),
            self.odd.deref(),
            self.even.deref(),
            intersection,
            rng,
        )
    }
}
#[derive(Clone, Debug)]
pub struct UvCheckerTexture {
    pub offset: Vector2,
    pub even: Arc<TextureInstance>,
    pub odd: Arc<TextureInstance>,
    pub scale: Number,
}

impl Texture for UvCheckerTexture {
    fn value(&self, intersection: &Intersection, rng: &mut dyn RngCore) -> Pixel {
        let pos = (intersection.uv.to_vector() / self.scale) + self.offset;

        do_checker(
            pos.to_array(),
            self.odd.deref(),
            self.even.deref(),
            intersection,
            rng,
        )
    }
}

#[inline(always)]
pub fn do_checker<C: Euclid + FloatCore>(
    coords: impl IntoIterator<Item = C>,
    odd: &impl Texture,
    even: &impl Texture,
    intersection: &Intersection,
    rng: &mut dyn RngCore,
) -> Pixel {
    let two: C = C::one() + C::one();

    let sum: C = coords
        .into_iter()
        .map(C::floor)
        .fold(C::zero(), |a: C, b: C| a + b);

    let is_even = C::rem_euclid(&sum, &two) < C::one();

    if is_even {
        even.value(intersection, rng)
    } else {
        odd.value(intersection, rng)
    }
}
