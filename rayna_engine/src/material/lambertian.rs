use crate::core::types::{Colour, Vector3};
use crate::material::Material;
use crate::shared::intersect::MeshIntersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use crate::texture::TextureInstance;
use crate::texture::{Texture, TextureToken};

use crate::scene::Scene;
use rand::RngCore;

#[derive(Copy, Clone, Debug)]
pub struct LambertianMaterial {
    pub albedo: TextureToken,
}

impl Default for LambertianMaterial {
    fn default() -> Self { Colour::HALF_GREY.into() }
}

impl From<TextureToken> for LambertianMaterial {
    fn from(value: TextureToken) -> Self { Self { albedo: value } }
}

impl Material for LambertianMaterial {
    fn scatter(
        &self,
        _ray: &Ray,
        _scene: &Scene,
        intersection: &MeshIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<Vector3> {
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
        scene: &Scene,
        intersect: &MeshIntersection,
        _future_ray: &Ray,
        future_col: &Colour,
        rng: &mut dyn RngCore,
    ) -> Colour {
        future_col * scene.get_tex(self.albedo).value(scene, intersect, rng)
    }
    fn emitted_light(
        &self,
        _ray: &Ray,
        _scene: &Scene,
        _intersection: &MeshIntersection,
        _rng: &mut dyn RngCore,
    ) -> Colour {
        Colour::BLACK
    }
}
