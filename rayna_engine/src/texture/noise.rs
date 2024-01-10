use crate::shared::intersect::Intersection;
use crate::shared::RtRequirement;
use crate::texture::{Texture, TextureInstance};
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

/// Enum that describes how a noise source is used to generate a colour for a pixel
///
/// The values can be output in the range `-1.0..=1.0`
#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum ColourSource<N: RtNoiseFn<D> + Clone, const D: usize> {
    /// Treat the noise generator's values as greyscale values
    Greyscale(N),
    /// Use the given gradient to convert noise values to colours
    ///
    /// Note this is a 24-bit RGB gradient, not the 96-bit RGB gradient used in the rest of the engine
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
        // Normalise `-1..1` to `0..1`
        .map_without_alpha(|c| c / 2. + 0.5)
    }
}

impl<'n, const D: usize, N: RtNoiseFn<D> + Clone + 'n> ColourSource<N, D> {
    /// Converts the explicit instance of a colour source into a dynamic boxed colour source
    pub fn as_dyn_box(&self) -> ColourSource<Box<dyn RtNoiseFn<D> + 'n>, D> {
        match self {
            Self::Greyscale(n) => ColourSource::Greyscale(Box::new(n.clone())),
            Self::Gradient(n, g) => ColourSource::Gradient(Box::new(n.clone()), g.clone()),
            Self::Rgb(n) => ColourSource::Rgb(
                n.each_ref()
                    .map(|n| Box::new(n.clone()) as Box<dyn RtNoiseFn<D>>),
            ),
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

// Unfortunately due to some problems with overlapping impls (which `feature = min_specialization` can't solve)
// We need to have the Box<N> here, meaning the user has to box their noise function
impl<N: RtNoiseFn<2> + Clone + 'static> From<UvNoiseTexture<Box<N>>> for TextureInstance {
    fn from(value: UvNoiseTexture<Box<N>>) -> Self {
        TextureInstance::UvNoiseTexture(UvNoiseTexture {
            func: value.func.as_dyn_box(),
        })
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

// See above for explanation of this
impl<N: RtNoiseFn<3> + Clone + 'static> From<WorldNoiseTexture<Box<N>>> for TextureInstance {
    fn from(value: WorldNoiseTexture<Box<N>>) -> Self {
        TextureInstance::WorldNoiseTexture(WorldNoiseTexture {
            func: value.func.as_dyn_box(),
        })
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

// See above for explanation of this
impl<N: RtNoiseFn<3> + Clone + 'static> From<LocalNoiseTexture<Box<N>>> for TextureInstance {
    fn from(value: LocalNoiseTexture<Box<N>>) -> Self {
        TextureInstance::LocalNoiseTexture(LocalNoiseTexture {
            func: value.func.as_dyn_box(),
        })
    }
}

/// Struct to allow using functions to act as noise sources
#[derive(Derivative, Copy)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
pub struct FnNoiseWrapper<F: RtRequirement + Clone>(F);
impl<const D: usize, F: Fn([Number; D]) -> f64 + RtRequirement + Clone>
    noise::NoiseFn<Number, { D }> for FnNoiseWrapper<F>
{
    fn get(&self, point: [Number; D]) -> f64 {
        self.0(point)
    }
}
