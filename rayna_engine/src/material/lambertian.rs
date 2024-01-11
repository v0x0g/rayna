use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use crate::texture::Texture;
use crate::texture::TextureInstance;
use rand::RngCore;
use rayna_shared::def::types::{Channel, Pixel, Vector3};

#[derive(Clone, Debug)]
pub struct LambertianMaterial {
    pub albedo: TextureInstance,
    pub emissive: TextureInstance,
}

impl Material for LambertianMaterial {
    fn scatter(&self, _ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Option<Vector3> {
        // Completely random scatter direction, in same hemisphere as normal
        let rand = rng::vector_in_unit_sphere(rng);
        // Bias towards the normal so we get a `cos(theta)` distribution (Lambertian scatter)
        let vec = intersection.ray_normal + rand;
        // Can't necessarily normalise, since maybe `rand + normal == 0`
        Some(vec.try_normalize().unwrap_or(intersection.ray_normal))
    }

    //noinspection DuplicatedCode
    fn calculate_colour(
        &self,
        _ray: &Ray,
        intersect: &Intersection,
        _future_ray: &Ray,
        future_col: &Pixel,
        rng: &mut dyn RngCore,
    ) -> Pixel {
        let f = future_col.0;
        let a = self.albedo.value(intersect, rng).0;
        let e = self.emissive.value(intersect, rng).0;

        Pixel::from([
            Channel::mul_add(f[0], a[0], e[0]),
            Channel::mul_add(f[1], a[1], e[1]),
            Channel::mul_add(f[2], a[2], e[2]),
        ])
    }
}
