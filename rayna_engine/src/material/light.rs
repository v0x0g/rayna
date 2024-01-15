use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::texture::{Texture, TextureInstance};
use rand_core::RngCore;
use rayna_shared::def::types::{Pixel, Vector3};

/// A simple emissive material for turning an object into a light.
///
/// Does not scatter.
#[derive(Clone, Debug)]
pub struct LightMaterial {
    pub emissive: TextureInstance,
}

impl Material for LightMaterial {
    fn scatter(&self, _ray: &Ray, _intersection: &Intersection, _rng: &mut dyn RngCore) -> Option<Vector3> { None }

    fn calculate_colour(
        &self,
        _ray: &Ray,
        intersection: &Intersection,
        _future_ray: &Ray,
        future_col: &Pixel,
        rng: &mut dyn RngCore,
    ) -> Pixel {
        super::calculate_colour_simple(future_col, Pixel::from([0.; 3]), self.emissive.value(intersection, rng))
    }
}
