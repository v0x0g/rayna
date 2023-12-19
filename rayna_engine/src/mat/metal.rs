use crate::mat::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::{rng, RtRequirement};
use image::Pixel as _;
use rand::thread_rng;
use rayna_shared::def::types::{Number, Pixel, Vector3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MetalMaterial {
    pub albedo: Pixel,
    pub fuzz: Number,
}

impl RtRequirement for MetalMaterial {}

impl Material for MetalMaterial {
    fn scatter(&self, intersection: &Intersection) -> Option<Vector3> {
        let d = intersection.ray.dir();
        let n = intersection.ray_normal;
        let reflected = d - n * (2. * d.dot(n));
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
        _intersection: &Intersection,
        _future_ray: Ray,
        future_col: Pixel,
    ) -> Pixel {
        Pixel::map2(&future_col, &self.albedo, |a, b| a * b)
    }
}
