use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::math;
use crate::shared::ray::Ray;
use crate::texture::Texture;
use image::Pixel as _;
use num_traits::Pow;
use rand::{Rng, RngCore};
use rayna_shared::def::types::{Channel, Number, Pixel, Vector3};
use std::ops::Mul;

#[derive(Copy, Clone, Debug)]
pub struct DielectricMaterial<Tex: Texture> {
    pub albedo: Tex,
    pub refractive_index: Number,
}

impl<Tex: Texture> Material for DielectricMaterial<Tex> {
    fn scatter(&self, ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Option<Vector3> {
        let index_ratio = if intersection.front_face {
            1.0 / self.refractive_index
        } else {
            self.refractive_index
        };
        let cos_theta = Number::min(Vector3::dot(-ray.dir(), intersection.ray_normal), 1.0);
        let sin_theta = Number::sqrt(1.0 - cos_theta * cos_theta);

        let total_internal_reflection = index_ratio * sin_theta > 1.0;
        let schlick_approx_reflect = Self::reflectance(cos_theta, index_ratio) > rng.gen::<Number>();

        let dir = if total_internal_reflection || schlick_approx_reflect {
            // Cannot refract, have to reflect
            math::reflect(ray.dir(), intersection.ray_normal)
        } else {
            math::refract(ray.dir(), intersection.ray_normal, index_ratio)
        };

        return Some(dir);
    }

    fn scatter_probability(&self, _ray_in: &Ray, _scattered: &Ray, _intersection: &Intersection) -> Number { todo!() }

    // TODO: Beer's law?
    //noinspection DuplicatedCode
    fn reflected_light(
        &self,
        _ray: &Ray,
        intersection: &Intersection,
        _future_ray: &Ray,
        future_col: &Pixel,
        rng: &mut dyn RngCore,
    ) -> Pixel {
        Pixel::map2(&future_col, &self.albedo.value(intersection, rng), Channel::mul)
    }
}

impl<Tex: Texture> DielectricMaterial<Tex> {
    fn reflectance(cosine: Number, ref_idx: Number) -> Number {
        // Use Schlick's approximation for reflectance.
        let r0 = (1. - ref_idx) / (1. + ref_idx);
        let r0_sqr = r0 * r0;
        return r0_sqr + (1. - r0_sqr) * Number::pow(1. - cosine, 5);
    }
}
