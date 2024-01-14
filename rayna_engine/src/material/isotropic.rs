use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use crate::texture::{Texture, TextureInstance};
use rand_core::RngCore;
use rayna_shared::def::types::{Pixel, Vector3};

/// A material that uniformly scatters rays in all directions
///
/// Normally this is paired with a [crate::object::homogenous_volume::HomogeneousVolumeObject]
#[derive(Clone, Debug)]
pub struct IsotropicMaterial {
    pub albedo: TextureInstance,
}

impl Material for IsotropicMaterial {
    fn scatter(&self, _ray: &Ray, _intersection: &Intersection, rng: &mut dyn RngCore) -> Option<Vector3> {
        Some(rng::vector_on_unit_sphere(rng))
    }

    fn calculate_colour(
        &self,
        _ray: &Ray,
        intersection: &Intersection,
        _future_ray: &Ray,
        future_col: &Pixel,
        rng: &mut dyn RngCore,
    ) -> Pixel {
        super::calculate_colour_simple(future_col, self.albedo.value(intersection, rng), Pixel::from([0.; 3]))
    }
}
