use crate::shared::intersect::Intersection;
use crate::shared::RtRequirement;
use crate::texture::Texture;
use derivative::Derivative;
use image::Pixel as _;
use noise::utils::ColorGradient;
use noise::NoiseFn;
use rand_core::RngCore;
use rayna_shared::def::types::{Channel, Number, Pixel};
#[allow(unused) /* Inside macro */]
use std::fmt::Debug;

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum ColourSource<N: NoiseFn<Number, D> + RtRequirement + Clone, const D: usize> {
    Greyscale(N),
    Gradient(N, ColorGradient),
    Rgb([N; 3]),
}

impl<const D: usize, N: NoiseFn<Number, D> + RtRequirement + Clone> ColourSource<N, D> {
    pub fn get(&self, point: [Number; D]) -> Pixel {
        match self {
            Self::Greyscale(n) => Pixel::from([n.get(point) as Channel; 3]),
            Self::Gradient(n, g) => *Pixel::from_slice(&g.get_color(n.get(point)).map(Into::into)),
            Self::Rgb(n) => Pixel::from(n.each_ref().map(|n| n.get(point) as Channel)),
        }
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct UvNoiseTexture<N: NoiseFn<Number, 2> + RtRequirement + Clone> {
    pub func: ColourSource<N, 2>,
}

impl<N: NoiseFn<Number, 2> + RtRequirement + Clone> Texture for UvNoiseTexture<N> {
    fn value(&self, intersection: &Intersection, _rng: &mut dyn RngCore) -> Pixel {
        self.func.get(intersection.uv.to_array())
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct WorldNoiseTexture<N: NoiseFn<Number, 3> + RtRequirement + Clone> {
    pub func: ColourSource<N, 3>,
}

impl<N: NoiseFn<Number, 3> + RtRequirement + Clone> Texture for WorldNoiseTexture<N> {
    fn value(&self, intersection: &Intersection, _rng: &mut dyn RngCore) -> Pixel {
        self.func.get(intersection.pos_w.to_array())
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct LocalNoiseTexture<N: NoiseFn<Number, 3> + RtRequirement + Clone> {
    pub func: ColourSource<N, 3>,
}

impl<N: NoiseFn<Number, 3> + RtRequirement + Clone> Texture for LocalNoiseTexture<N> {
    fn value(&self, intersection: &Intersection, _rng: &mut dyn RngCore) -> Pixel {
        self.func.get(intersection.pos_l.to_array())
    }
}
