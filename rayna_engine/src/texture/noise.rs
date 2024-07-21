use crate::core::gradient::Gradient;
use crate::core::types::{Colour, Number};
use crate::noise::NoiseToken;
use crate::scene::Scene;
use crate::shared::intersect::MeshIntersection;
use crate::texture::Texture;
use rand_core::RngCore;

#[derive(Debug, Clone)]
pub enum NoiseColour {
    Monochrome {
        noise: NoiseToken,
        colour: Colour,
    },
    Rgb {
        r: NoiseToken,
        g: NoiseToken,
        b: NoiseToken,
    },
    Gradient {
        noise: NoiseToken,
        gradient: Gradient<Colour>,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum NoiseSource {
    UV,
    WorldPos,
    LocalPos,
}

#[derive(Clone, Debug)]
pub struct NoiseTexture {
    pub source: NoiseSource,
    pub colour: NoiseColour,
}

impl Texture for NoiseTexture {
    fn value(&self, scene: &Scene, intersection: &MeshIntersection, _rng: &mut dyn RngCore) -> Colour {
        fn convert(val: f64) -> Number { (val as Number) / 2.0 + 0.5 }

        // FIXME: This is extremely bad code, and it's factorial complexity.
        //  This needs to be refactored ASAP. Maybe a macro?
        match self {
            Self {
                source: NoiseSource::UV,
                colour: NoiseColour::Monochrome { noise, colour },
            } => {
                let val = scene.get_noise2(noise).value(intersection.uv.as_array());
                colour * convert(val)
            }
            Self {
                source: NoiseSource::WorldPos,
                colour: NoiseColour::Monochrome { noise, colour },
            } => {
                let val = scene.get_noise3(noise).value(intersection.pos_w.as_array());
                colour * convert(val)
            }
            Self {
                source: NoiseSource::LocalPos,
                colour: NoiseColour::Monochrome { noise, colour },
            } => {
                let val = scene.get_noise3(noise).value(intersection.pos_l.as_array());
                colour * convert(val)
            }
            Self {
                source: NoiseSource::UV,
                colour: NoiseColour::Rgb { r, g, b },
            } => {
                let r = scene.get_noise2(r).value(intersection.uv.as_array());
                let g = scene.get_noise2(g).value(intersection.uv.as_array());
                let b = scene.get_noise2(b).value(intersection.uv.as_array());
                Colour::from([r, g, b].map(convert))
            }
            Self {
                source: NoiseSource::WorldPos,
                colour: NoiseColour::Rgb { r, g, b },
            } => {
                let r = scene.get_noise3(r).value(intersection.pos_w.as_array());
                let g = scene.get_noise3(g).value(intersection.pos_w.as_array());
                let b = scene.get_noise3(b).value(intersection.pos_w.as_array());
                Colour::from([r, g, b].map(convert))
            }
            Self {
                source: NoiseSource::LocalPos,
                colour: NoiseColour::Rgb { r, g, b },
            } => {
                let r = scene.get_noise2(r).value(intersection.pos_l.as_array());
                let g = scene.get_noise2(g).value(intersection.pos_l.as_array());
                let b = scene.get_noise2(b).value(intersection.pos_l.as_array());
                Colour::from([r, g, b].map(convert))
            }
            Self {
                source: NoiseSource::UV,
                colour: NoiseColour::Gradient { noise, gradient },
            } => {
                let val = scene.get_noise2(noise).value(intersection.uv.as_array());
                gradient.get(convert(val))
            }
            Self {
                source: NoiseSource::WorldPos,
                colour: NoiseColour::Gradient { noise, gradient },
            } => {
                let val = scene.get_noise3(noise).value(intersection.pos_w.as_array());
                gradient.get(convert(val))
            }
            Self {
                source: NoiseSource::LocalPos,
                colour: NoiseColour::Gradient { noise, gradient },
            } => {
                let val = scene.get_noise2(noise).value(intersection.pos_l.as_array());
                gradient.get(convert(val))
            }
        }
    }
}
