use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::{math, rng};
use crate::texture::Texture;
use image::Pixel as _;
use rand::RngCore;
use rayna_shared::def::types::{Channel, Colour, Number, Vector3};
use std::ops::Mul;

#[derive(Copy, Clone, Debug)]
pub struct MetalMaterial<Tex: Texture> {
    pub albedo: Tex,
    pub fuzz: Number,
}

impl<Tex: Texture> Material for MetalMaterial<Tex> {
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

    fn scatter_probability(&self, _ray_in: &Ray, _scattered: &Ray, _intersection: &Intersection) -> Number { todo!() }

    //noinspection DuplicatedCode
    fn reflected_light(
        &self,
        _ray: &Ray,
        intersect: &Intersection,
        _future_ray: &Ray,
        future_col: &Colour,
        rng: &mut dyn RngCore,
    ) -> Colour {
        Colour::map2(future_col, &self.albedo.value(intersect, rng), Channel::mul)
    }
}
