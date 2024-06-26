use crate::core::types::{Channel, Colour, Number, Point3, Vector3};
use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::math;
use crate::shared::ray::Ray;
use crate::texture::Texture;

use num_traits::Pow;
use rand::{Rng, RngCore};

#[derive(Copy, Clone, Debug)]
pub struct DielectricMaterial<Tex: Texture> {
    pub albedo: Tex,
    pub refractive_index: Number,
    pub density: Number,
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

    //noinspection DuplicatedCode
    fn reflected_light(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        _future_ray: &Ray,
        future_col: &Colour,
        rng: &mut dyn RngCore,
    ) -> Colour {
        // We only get information for the previous ray and current intersection here (not future intersect)
        // Therefore we cannot know how far we have travelled inside the material on the 'entering' intersection.
        // So on the entering intersection, do nothing, and on exiting intersection, calculate distance travelled inside
        // the object, so we can use [Beer's Law] (https://en.wikipedia.org/wiki/Beer%E2%80%93Lambert_law)
        // Possibly sub-optimal, but not much we can do

        let exiting_intersection = !intersection.front_face;
        if !exiting_intersection {
            return *future_col;
        }

        let dist_inside = Point3::distance(intersection.pos_w, ray.pos());
        let transmission = (-self.density * dist_inside) as Channel;
        // NOTE: This is the colour at the exiting intersection, which might not be accurate if the texture
        //  is non-homogenous
        // TODO: Fix this texture issue somehow, maybe sample along the line and integrate that?
        let attenuation_col = self.albedo.value(intersection, rng);

        // future_col * (attenuation_col.exp(transmission))
        future_col * attenuation_col * transmission.exp()
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
