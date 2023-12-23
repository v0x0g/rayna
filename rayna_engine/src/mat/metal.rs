use crate::mat::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::{math, rng, RtRequirement};
use image::Pixel as _;
use rand::{thread_rng, Rng};
use rayna_shared::def::types::{Number, Pixel, Vector3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MetalMaterial {
    pub albedo: Pixel,
    pub fuzz: Number,
}

impl RtRequirement for MetalMaterial {}

impl Material for MetalMaterial {
    fn scatter(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        rng: &mut dyn Rng,
    ) -> Option<Vector3> {
        let reflected = math::reflect(ray.dir(), intersection.ray_normal);
        let rand = rng::vector_on_unit_sphere(&mut thread_rng());

        // Generate some fuzzy reflections by adding a "cloud" of random points
        // around the reflection (a sphere with `radius=fuzz` centred at `reflected)
        let vec = reflected + (rand * self.fuzz);
        // This might end up scattering beneath the surface of the object, so check here
        return if Vector3::dot(vec, intersection.ray_normal) > 0. {
            // Scatter ok
            Some(vec.normalize())
        } else {
            // Scattered under surface
            None
        };
    }

    fn calculate_colour(
        &self,
        _ray: &Ray,
        _intersection: &Intersection,
        _future_ray: &Ray,
        future_col: &Pixel,
    ) -> Pixel {
        Pixel::map2(&future_col, &self.albedo, |a, b| a * b)
    }
}
