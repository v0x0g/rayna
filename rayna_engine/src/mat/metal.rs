use crate::mat::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use image::Pixel as _;
use rayna_shared::def::types::{Pixel, Vector3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MetalMaterial {
    pub albedo: Pixel,
}

impl RtRequirement for MetalMaterial {}

impl Material for MetalMaterial {
    fn scatter(&self, intersection: &Intersection) -> Option<Vector3> {
        let d = intersection.ray.dir();
        let n = intersection.ray_normal;
        let vec = d - n * (2. * d.dot(n));
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
