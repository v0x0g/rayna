use crate::shared::intersect::Intersection;
use crate::shared::RtRequirement;
use crate::texture::Texture;
use derivative::Derivative;
use image::Pixel as _;
use noise::utils::ColorGradient;
use rand_core::RngCore;
use rayna_shared::def::types::{Channel, Number, Pixel};
#[allow(unused) /* Inside macro */]
use std::fmt::Debug;

/// An extended trait what wraps a few other traits.
///
/// Essentially a noise function that's safe to use in the engine
pub trait RtNoiseFn<const D: usize>: noise::NoiseFn<Number, { D }> + RtRequirement {}
impl<const D: usize, N: noise::NoiseFn<Number, { D }> + RtRequirement + Clone> RtNoiseFn<D> for N {}
dyn_clone::clone_trait_object!(<const D: usize> RtNoiseFn<D>);

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum ColourSource<N: RtNoiseFn<D> + Clone, const D: usize> {
    Greyscale(N),
    Gradient(N, ColorGradient),
    Rgb([N; 3]),
}

impl<const D: usize, N: RtNoiseFn<D> + Clone> ColourSource<N, D> {
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
pub struct UvNoiseTexture<N: RtNoiseFn<2> + Clone> {
    pub func: ColourSource<N, 2>,
}

impl<N: RtNoiseFn<2> + Clone> Texture for UvNoiseTexture<N> {
    fn value(&self, intersection: &Intersection, _rng: &mut dyn RngCore) -> Pixel {
        self.func.get(intersection.uv.to_array())
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct WorldNoiseTexture<N: RtNoiseFn<3> + Clone> {
    pub func: ColourSource<N, 3>,
}

impl<N: RtNoiseFn<3> + Clone> Texture for WorldNoiseTexture<N> {
    fn value(&self, intersection: &Intersection, _rng: &mut dyn RngCore) -> Pixel {
        self.func.get(intersection.pos_w.to_array())
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct LocalNoiseTexture<N: RtNoiseFn<3> + Clone> {
    pub func: ColourSource<N, 3>,
}

impl<N: RtNoiseFn<3> + Clone> Texture for LocalNoiseTexture<N> {
    fn value(&self, intersection: &Intersection, _rng: &mut dyn RngCore) -> Pixel {
        self.func.get(intersection.pos_l.to_array())
    }
}
