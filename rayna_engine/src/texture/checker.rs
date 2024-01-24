use num_traits::float::FloatCore;
use num_traits::Euclid;
use rand_core::RngCore;
use std::fmt::Debug;

use crate::core::types::{Colour, Number, Vector2, Vector3};

use crate::shared::intersect::Intersection;
use crate::texture::dynamic::DynamicTexture;
use crate::texture::Texture;

#[derive(Clone, Debug)]
pub struct WorldCheckerTexture<Odd: Texture = DynamicTexture, Even: Texture = DynamicTexture> {
    pub offset: Vector3,
    pub even: Even,
    pub odd: Odd,
    pub scale: Number,
}

impl<Odd: Texture, Even: Texture> Texture for WorldCheckerTexture<Odd, Even> {
    fn value(&self, intersection: &Intersection, rng: &mut dyn RngCore) -> Colour {
        let pos = (intersection.pos_w.to_vector() / self.scale) + self.offset;

        do_checker(pos.to_array(), &self.odd, &self.even, intersection, rng)
    }
}

#[derive(Clone, Debug)]
pub struct UvCheckerTexture<Odd: Texture = DynamicTexture, Even: Texture = DynamicTexture> {
    pub offset: Vector2,
    pub even: Even,
    pub odd: Odd,
    pub scale: Number,
}

impl<Odd: Texture, Even: Texture> Texture for UvCheckerTexture<Odd, Even> {
    fn value(&self, intersection: &Intersection, rng: &mut dyn RngCore) -> Colour {
        let pos = (intersection.uv.to_vector() / self.scale) + self.offset;

        do_checker(pos.to_array(), &self.odd, &self.even, intersection, rng)
    }
}

#[inline(always)]
pub fn do_checker<C: Euclid + FloatCore>(
    coords: impl IntoIterator<Item = C>,
    odd: &impl Texture,
    even: &impl Texture,
    intersection: &Intersection,
    rng: &mut dyn RngCore,
) -> Colour {
    let two: C = C::one() + C::one();

    let sum: C = coords.into_iter().map(C::floor).fold(C::zero(), |a: C, b: C| a + b);

    let is_even = C::rem_euclid(&sum, &two) < C::one();

    if is_even {
        even.value(intersection, rng)
    } else {
        odd.value(intersection, rng)
    }
}
