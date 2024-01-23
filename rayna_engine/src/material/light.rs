use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::texture::Texture;
use rand_core::RngCore;
use rayna_shared::def::types::{Colour, Number, Vector3};

/// A simple emissive material for turning an mesh into a light.
///
/// Does not scatter.
#[derive(Copy, Clone, Debug)]
pub struct LightMaterial<Tex: Texture> {
    pub emissive: Tex,
}

impl<Tex: Texture> Material for LightMaterial<Tex> {
    fn scatter(&self, _ray: &Ray, _intersection: &Intersection, _rng: &mut dyn RngCore) -> Option<Vector3> { None }

    fn scatter_probability(&self, _ray_in: &Ray, _scattered: &Ray, _intersection: &Intersection) -> Number {
        // Light never scatters
        0.0
    }

    fn emitted_light(&self, _ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Colour {
        self.emissive.value(intersection, rng)
    }

    fn reflected_light(
        &self,
        _ray: &Ray,
        _intersection: &Intersection,
        _future_ray: &Ray,
        _future_col: &Colour,
        _rng: &mut dyn RngCore,
    ) -> Colour {
        [0.; 3].into()
    }
}
