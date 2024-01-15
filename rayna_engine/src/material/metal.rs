use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::{math, rng};
use crate::texture::{Texture, TextureInstance};
use image::Pixel as _;
use rand::RngCore;
use rayna_shared::def::types::{Channel, Number, Pixel, Vector3};
use std::ops::Mul;

#[derive(Clone, Debug)]
pub struct MetalMaterial {
    pub albedo: TextureInstance,
    pub fuzz: Number,
}

impl Material for MetalMaterial {
    fn scatter(&self, ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Option<Vector3> {
        let reflected = math::reflect(ray.dir(), intersection.ray_normal);
        let rand = rng::vector_on_unit_sphere(rng);

        // Generate some fuzzy reflections by adding a "cloud" of random points
        // around the reflection (a sphere with `radius=fuzz` centred at `reflected)
        let vec = reflected + (rand * self.fuzz);
        // This might end up scattering beneath the surface of the mesh, so check here
        return if Vector3::dot(vec, intersection.ray_normal) > 0. {
            // Scatter ok
            Some(vec.normalize())
        } else {
            // Scattered under surface
            None
        };
    }

    //noinspection DuplicatedCode
    fn reflected_light(
        &self,
        _ray: &Ray,
        intersect: &Intersection,
        _future_ray: &Ray,
        future_col: &Pixel,
        rng: &mut dyn RngCore,
    ) -> Pixel {
        Pixel::map2(future_col, &self.albedo.value(intersect, rng), Channel::mul)
    }
}
