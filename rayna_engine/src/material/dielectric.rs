use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::math;
use crate::shared::ray::Ray;
use image::Pixel as _;
use num_traits::Pow;
use rand::{Rng, RngCore};
use rayna_shared::def::types::{Number, Pixel, Vector3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DielectricMaterial {
    pub albedo: Pixel,
    pub refractive_index: Number,
}

impl Material for DielectricMaterial {
    fn scatter(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        rng: &mut dyn RngCore,
    ) -> Option<Vector3> {
        let index_ratio = if intersection.front_face {
            1.0 / self.refractive_index
        } else {
            self.refractive_index
        };
        let cos_theta = Number::min(Vector3::dot(-ray.dir(), intersection.ray_normal), 1.0);
        let sin_theta = Number::sqrt(1.0 - cos_theta * cos_theta);

        let total_internal_reflection = index_ratio * sin_theta > 1.0;
        let schlick_approx_reflect =
            Self::reflectance(cos_theta, index_ratio) > rng.gen::<Number>();

        let dir = if total_internal_reflection || schlick_approx_reflect {
            // Cannot refract, have to reflect
            math::reflect(ray.dir(), intersection.ray_normal)
        } else {
            math::refract(ray.dir(), intersection.ray_normal, index_ratio)
        };

        return Some(dir);
    }

    //noinspection DuplicatedCode
    fn calculate_colour(
        &self,
        _ray: &Ray,
        _intersection: &Intersection,
        _future_ray: &Ray,
        future_col: &Pixel,
        rng: &mut dyn RngCore,
    ) -> Pixel {
        Pixel::map2(&future_col, &self.albedo, |a, b| a * b)
    }
}

impl DielectricMaterial {
    fn reflectance(cosine: Number, ref_idx: Number) -> Number {
        // Use Schlick's approximation for reflectance.
        let r0 = (1. - ref_idx) / (1. + ref_idx);
        let r0_sqr = r0 * r0;
        return r0_sqr + (1. - r0_sqr) * Number::pow(1. - cosine, 5);
    }
}
