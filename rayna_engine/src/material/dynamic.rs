use crate::core::types::{Colour, Vector3};
use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use rand_core::RngCore;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DynamicMaterial {
    pub inner: Arc<dyn Material>,
}

impl Material for DynamicMaterial {
    fn scatter(&self, ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Option<Vector3> {
        self.inner.scatter(ray, intersection, rng)
    }

    fn emitted_light(&self, ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Colour {
        self.inner.emitted_light(ray, intersection, rng)
    }

    fn reflected_light(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        future_ray: &Ray,
        future_col: &Colour,
        rng: &mut dyn RngCore,
    ) -> Colour {
        self.inner
            .reflected_light(ray, intersection, future_ray, future_col, rng)
    }
}
