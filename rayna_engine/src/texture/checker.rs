use std::fmt::Debug;

use glamour::TransformMap;
use num_traits::float::FloatCore;
use num_traits::Euclid;
use rand_core::RngCore;

use crate::core::types::{Colour, Transform2, Transform3};
use crate::scene::Scene;
use crate::shared::intersect::MeshIntersection;
use crate::texture::{Texture, TextureToken};

#[derive(Clone, Debug)]
pub struct WorldCheckerTexture {
    pub even: TextureToken,
    pub odd: TextureToken,
    pub transform: Transform3,
}

impl Texture for WorldCheckerTexture {
    fn value(&self, scene: &Scene, intersection: &MeshIntersection, rng: &mut dyn RngCore) -> Colour {
        let pos = self.transform.map_point(intersection.pos_w);

        let tok = choose_checker(pos.to_array(), self.odd, self.even);
        scene.get_tex(tok).value(scene, intersection, rng)
    }
}

#[derive(Clone, Debug)]
pub struct UvCheckerTexture {
    pub even: TextureToken,
    pub odd: TextureToken,
    pub transform: Transform2,
}

impl Texture for UvCheckerTexture {
    fn value(&self, scene: &Scene, intersection: &MeshIntersection, rng: &mut dyn RngCore) -> Colour {
        let pos = self.transform.map(intersection.uv);

        let tok = choose_checker(pos.to_array(), self.odd, self.even);
        scene.get_tex(tok).value(scene, intersection, rng)
    }
}

#[inline(always)]
pub fn choose_checker<C: Euclid + FloatCore>(
    coords: impl IntoIterator<Item = C>,
    odd: TextureToken,
    even: TextureToken,
) -> TextureToken {
    let two: C = C::one() + C::one();
    let sum: C = coords.into_iter().map(C::floor).fold(C::zero(), |a: C, b: C| a + b);
    let is_even = C::rem_euclid(&sum, &two) < C::one();
    if is_even {
        even
    } else {
        odd
    }
}
