use crate::core::gradient::Gradient;
use crate::core::intersect::MeshIntersection;
use crate::core::types::{Channel, Colour, Number};
use crate::noise::{Noise, NoiseToken};
use crate::scene::Scene;
use crate::texture::Texture;
use rand_core::RngCore;

#[derive(Copy, Clone, Debug)]
pub enum NoiseSource {
    UV(NoiseToken<2>),
    WorldPos(NoiseToken<3>),
    LocalPos(NoiseToken<3>),
}

#[derive(Clone, Debug)]
pub enum NoiseTexture {
    Monochrome {
        noise: NoiseSource,
        colour: Colour,
    },
    Rgb {
        r: NoiseSource,
        g: NoiseSource,
        b: NoiseSource,
    },
    Gradient {
        noise: NoiseSource,
        gradient: Gradient<Colour>,
    },
}

impl Texture for NoiseTexture {
    fn value(&self, scene: &Scene, intersection: &MeshIntersection, _rng: &mut dyn RngCore) -> Colour {
        let lookup = |source: &NoiseSource| -> Number {
            let val = match source {
                NoiseSource::UV(tok) => scene.get_noise2(&tok).value(intersection.uv.as_array()),
                NoiseSource::WorldPos(tok) => scene.get_noise3(&tok).value(intersection.pos_w.as_array()),
                NoiseSource::LocalPos(tok) => scene.get_noise3(&tok).value(intersection.pos_l.as_array()),
            };
            // Converts the float value output by the `noise` crate into a value we can use
            // This allows is to skip that step in the generator
            (val as Number) / 2.0 + 0.5
        };

        match self {
            Self::Monochrome { noise, colour } => colour * lookup(noise),
            Self::Rgb { r, g, b } => [r, g, b].map(lookup).map(|c| c as Channel).into(),
            Self::Gradient { noise, gradient } => gradient.get(lookup(noise)),
        }
    }
}
