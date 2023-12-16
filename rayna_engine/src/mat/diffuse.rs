use crate::mat::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use crate::shared::RtRequirement;
use image::Pixel as _;
use rand::thread_rng;
use rayna_shared::def::types::{Pixel, Vector};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DiffuseMaterial {}

impl RtRequirement for DiffuseMaterial {}

impl Material for DiffuseMaterial {
    fn scatter(&self, intersection: &Intersection) -> Option<Vector> {
        // Completely random scatter direction, in same hemisphere as normal
        let rand = rng::vector_on_unit_hemisphere(&mut thread_rng(), intersection.normal);

        // Bias towards the normal so we get a `cos(theta)` distribution (Lambertian scatter)
        let vec = intersection.normal + rand;
        // Can normalise safely since we know can never be zero
        Some(vec.normalize())
    }

    fn calculate_colour(
        &self,
        _intersection: &Intersection,
        _future_ray: Ray,
        future_col: Pixel,
    ) -> Pixel {
        // Half grey
        future_col.map(|c| c / 2.)
    }
}
