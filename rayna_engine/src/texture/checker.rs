use std::fmt::Debug;
use std::ops::Deref;
use num_traits::float::FloatCore;
use num_traits::Euclid;
use rand_core::RngCore;

use rayna_shared::def::types::{Number, Pixel, Vector2, Vector3};

use crate::shared::intersect::Intersection;
use crate::texture::Texture;
use crate::texture::dynamic::DynamicTexture;

#[derive(Clone, Debug)]
pub struct WorldCheckerTexture<Odd: Texture + Clone = DynamicTexture, Even: Texture + Clone = DynamicTexture> {
    pub offset: Vector3,
    pub even: Even,
    pub odd: Odd,
    pub scale: Number,
}

impl<Odd: Texture + Clone, Even: Texture + Clone> Texture for WorldCheckerTexture<Odd, Even> {
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
pub struct UvCheckerTexture<Odd: Texture + Clone = DynamicTexture, Even: Texture + Clone = DynamicTexture> {
    pub offset: Vector2,
    pub even: Even,
    pub odd: Odd,
    pub scale: Number,
}

impl<Odd: Texture + Clone, Even: Texture + Clone> Texture for UvCheckerTexture<Odd, Even> {
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
    rng: &mut impl RngCore,
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
