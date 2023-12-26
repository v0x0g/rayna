use crate::material::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use rand_core::RngCore;
use rayna_shared::def::types::{Pixel, Vector3};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DynamicMaterial {
    pub inner: Arc<dyn Material>,
}

impl Material for DynamicMaterial {
    fn scatter(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        rng: &mut dyn RngCore,
    ) -> Option<Vector3> {
        self.inner.scatter(ray, intersection, rng)
    }

    fn calculate_colour(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        future_ray: &Ray,
        future_col: &Pixel,
    ) -> Pixel {
        self.inner
            .calculate_colour(ray, intersection, future_ray, future_col)
    }
}
