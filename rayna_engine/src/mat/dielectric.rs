use crate::mat::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::{math, RtRequirement};
use image::Pixel as _;
use rayna_shared::def::types::{Number, Pixel, Vector3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DielectricMaterial {
    pub albedo: Pixel,
    pub refractive_index: Number,
}

impl RtRequirement for DielectricMaterial {}

impl Material for DielectricMaterial {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<Vector3> {
        let ir_ratio = if intersection.front_face {
            1.0 / self.refractive_index
        } else {
            self.refractive_index
        };
        let cos_theta = Number::min(Vector3::dot(-ray.dir(), intersection.normal), 1.0);
        let sin_theta = Number::sqrt(1.0 - cos_theta * cos_theta);

        let dir = if ir_ratio * sin_theta > 1.0 {
            // Cannot refract, have to reflect
            math::reflect(ray.dir(), intersection.normal)
        } else {
            math::refract(ray.dir(), intersection.normal, ir_ratio)
        };

        return Some(dir);
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
