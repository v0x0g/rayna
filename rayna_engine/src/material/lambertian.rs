use crate::core::types::{Colour, Vector3};
use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use crate::texture::Texture;
use crate::texture::TextureInstance;

use rand::RngCore;

#[derive(Copy, Clone, Debug)]
pub struct LambertianMaterial<Tex: Texture> {
    pub albedo: Tex,
}

impl Default for LambertianMaterial<TextureInstance> {
    fn default() -> Self {
        Self {
            albedo: [0.5; 3].into(),
        }
    }
}

impl<Tex: Texture> Material for LambertianMaterial<Tex> {
    fn scatter(&self, _ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Option<Vector3> {
        // Completely random scatter direction, in same hemisphere as normal
        let rand = rng::vector_in_unit_sphere(rng);
        // Bias towards the normal so we get a `cos(theta)` distribution (Lambertian scatter)
        let vec = intersection.ray_normal + rand;
        // Can't necessarily normalise, since maybe `rand + normal == 0`
        Some(vec.try_normalize().unwrap_or(intersection.ray_normal))
    }

    //noinspection DuplicatedCode
    fn reflected_light(
        &self,
        _ray: &Ray,
        intersect: &Intersection,
        _future_ray: &Ray,
        future_col: &Colour,
        rng: &mut dyn RngCore,
    ) -> Colour {
        future_col * self.albedo.value(intersect, rng)
    }
}
