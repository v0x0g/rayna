use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use image::Pixel as _;
use rand::RngCore;
use rayna_shared::def::types::{Pixel, Vector3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LambertianMaterial {
    pub albedo: Pixel,
}

impl Material for LambertianMaterial {
    fn scatter(
        &self,
        _ray: &Ray,
        intersection: &Intersection,
        rng: &mut dyn RngCore,
    ) -> Option<Vector3> {
        // Completely random scatter direction, in same hemisphere as normal
        let rand = rng::vector_in_unit_sphere(rng);
        // Bias towards the normal so we get a `cos(theta)` distribution (Lambertian scatter)
        let vec = intersection.normal + rand;
        // Can't necessarily normalise, since maybe `rand + normal == 0`
        Some(vec.try_normalize().unwrap_or(intersection.normal))
    }

    //noinspection DuplicatedCode
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
