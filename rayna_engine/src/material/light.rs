use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::texture::Texture;
use rand_core::RngCore;
use rayna_shared::def::types::{Pixel, Vector3};

/// A simple emissive material for turning an mesh into a light.
///
/// Does not scatter.
#[derive(Copy, Clone, Debug)]
pub struct LightMaterial<Tex: Texture> {
    pub emissive: Tex,
}

impl<Tex: Texture> Material for LightMaterial<Tex> {
    fn scatter(&self, _ray: &Ray, _intersection: &Intersection, _rng: &mut dyn RngCore) -> Option<Vector3> { None }

    fn emitted_light(&self, _ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Pixel {
        self.emissive.value(intersection, rng)
    }

    fn reflected_light(
        &self,
        _ray: &Ray,
        _intersection: &Intersection,
        _future_ray: &Ray,
        _future_col: &Pixel,
        _rng: &mut dyn RngCore,
    ) -> Pixel {
        [0.; 3].into()
    }
}
