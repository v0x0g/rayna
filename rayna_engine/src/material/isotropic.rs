use crate::core::types::{Colour, Number, Vector3};
use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use crate::texture::{Texture, TextureInstance};

use rand_core::RngCore;

/// A material that uniformly scatters rays in all directions
///
/// Normally this is paired with a [crate::mesh::homogenous_volume::HomogeneousVolumeMesh]
#[derive(Copy, Clone, Debug)]
pub struct IsotropicMaterial<Tex: Texture> {
    pub albedo: Tex,
}

impl Default for IsotropicMaterial<TextureInstance> {
    fn default() -> Self {
        Self {
            albedo: [0.5; 3].into(),
        }
    }
}

impl<Tex: Texture> Material for IsotropicMaterial<Tex> {
    fn scatter(&self, _ray: &Ray, _intersection: &Intersection, rng: &mut dyn RngCore) -> Option<Vector3> {
        Some(rng::vector_on_unit_sphere(rng))
    }
    // TODO: Should be equal all directions
    fn scatter_probability(&self, _ray_in: &Ray, _scattered: &Ray, _intersection: &Intersection) -> Number { todo!() }

    //TODO: Take into account distance along travelled ray (beer's law?)
    fn reflected_light(
        &self,
        _ray: &Ray,
        intersection: &Intersection,
        _future_ray: &Ray,
        future_col: &Colour,
        rng: &mut dyn RngCore,
    ) -> Colour {
        let albedo = self.albedo.value(intersection, rng);
        future_col * albedo
    }
}
