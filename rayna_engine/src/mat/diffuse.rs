use crate::mat::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use crate::shared::RtRequirement;
use image::Pixel as _;
use rand::thread_rng;
use rayna_shared::def::types::{Number, Pixel, Vector3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DiffuseMaterial {
    pub diffusion: Number,
    pub albedo: Pixel,
}

impl RtRequirement for DiffuseMaterial {}

impl Material for DiffuseMaterial {
    fn scatter(&self, intersection: &Intersection) -> Option<Vector3> {
        // Completely random scatter direction, in same hemisphere as normal
        let rand = rng::vector_on_unit_hemisphere(&mut thread_rng(), intersection.normal);

        // Bias towards the normal so we get a `cos(theta)` distribution (Lambertian scatter)
        let vec = intersection.normal + (rand * self.diffusion);
        // Can normalise safely since we know can never be zero
        Some(vec.normalize())
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
