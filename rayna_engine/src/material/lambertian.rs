use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use crate::texture::Texture;
use crate::texture::TextureInstance;
use glamour::AngleConsts;
use image::Pixel as _;
use rand::RngCore;
use rayna_shared::def::types::{Channel, Number, Pixel, Vector3};
use std::ops::Mul;

#[derive(Copy, Clone, Debug)]
pub struct LambertianMaterial<TexAlbedo: Texture, TexEmissive: Texture> {
    pub albedo: TexAlbedo,
    pub emissive: TexEmissive,
}

impl Default for LambertianMaterial<TextureInstance, TextureInstance> {
    fn default() -> Self {
        Self {
            albedo: [0.5; 3].into(),
            emissive: [0.; 3].into(),
        }
    }
}

impl<TexAlbedo: Texture, TexEmissive: Texture> Material for LambertianMaterial<TexAlbedo, TexEmissive> {
    fn scatter(&self, _ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Option<Vector3> {
        // Completely random scatter direction, in same hemisphere as normal
        let rand = rng::vector_in_unit_sphere(rng);
        // Bias towards the normal so we get a `cos(theta)` distribution (Lambertian scatter)
        let vec = intersection.ray_normal + rand;
        // Can't necessarily normalise, since maybe `rand + normal == 0`
        Some(vec.try_normalize().unwrap_or(intersection.ray_normal))
    }

    fn scatter_pdf(&self, _ray_in: &Ray, scattered: &Ray, intersection: &Intersection) -> Number {
        // We have a `cos(theta)` lambertian distribution,
        // Where `P(ray_out) = cos(angle_between(ray_in, ray_out))`
        // We can factor this using the dot product
        let cos_theta = intersection.ray_normal.dot(scattered.dir());
        return (cos_theta / Number::PI).max(0.);
    }

    fn emitted_light(&self, _ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Pixel {
        self.emissive.value(intersection, rng)
    }

    //noinspection DuplicatedCode
    fn reflected_light(
        &self,
        _ray: &Ray,
        intersect: &Intersection,
        _future_ray: &Ray,
        future_col: &Pixel,
        rng: &mut dyn RngCore,
    ) -> Pixel {
        Pixel::map2(future_col, &self.albedo.value(intersect, rng), Channel::mul)
    }
}
