use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use crate::texture::Texture;
use crate::texture::TextureType;
use rand::RngCore;
use rayna_shared::def::types::{Channel, Pixel, Vector3};

#[derive(Clone, Debug)]
pub struct LambertianMaterial {
    pub albedo: TextureType,
    pub emissive: TextureType,
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
        intersect: &Intersection,
        _future_ray: &Ray,
        future_col: &Pixel,
        rng: &mut dyn RngCore,
    ) -> Pixel {
        let f = future_col.0;
        let a = self.albedo.value(intersect, rng).0;
        let e = self.emissive.value(intersect, rng).0;

        unsafe {
            use core::arch::x86_64::*;
            use image::Pixel as _;
            // SAFETY: Loading len-3 slices into len-4 vector but should be safe since it's all on stack
            // 4th value is discarded
            let f = _mm_loadu_ps(f.as_ptr());
            let a = _mm_loadu_ps(a.as_ptr());
            let e = _mm_loadu_ps(e.as_ptr());
            let o = _mm_fmadd_ps(f, a, e);
            let mut p = [0.; 4];
            _mm_storeu_ps(p.as_mut_ptr(), o);
            *Pixel::from_slice(&p[0..3])
        }

        Pixel::from([
            Channel::mul_add(f[0], a[0], e[0]),
            Channel::mul_add(f[1], a[1], e[1]),
            Channel::mul_add(f[2], a[2], e[2]),
        ])
    }
}
