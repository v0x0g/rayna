use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use crate::texture::{Texture, TextureInstance};
use image::Pixel as _;
use rand_core::RngCore;
use rayna_shared::def::types::{Channel, Pixel, Vector3};
use std::ops::Mul;

/// A material that uniformly scatters rays in all directions
///
/// Normally this is paired with a [crate::object::homogenous_volume::HomogeneousVolumeObject]
#[derive(Clone, Debug)]
pub struct IsotropicMaterial {
    pub albedo: TextureInstance,
}

impl Default for IsotropicMaterial {
    fn default() -> Self {
        Self {
            albedo: [0.5; 3].into(),
        }
    }
}

impl Material for IsotropicMaterial {
    fn scatter(&self, _ray: &Ray, _intersection: &Intersection, rng: &mut dyn RngCore) -> Option<Vector3> {
        Some(rng::vector_on_unit_sphere(rng))
    }

    fn reflected_light(
        &self,
        _ray: &Ray,
        intersection: &Intersection,
        _future_ray: &Ray,
        future_col: &Pixel,
        rng: &mut dyn RngCore,
    ) -> Pixel {
        let albedo = self.albedo.value(intersection, rng);
        Pixel::map2(future_col, &albedo, Channel::mul)
    }
}
