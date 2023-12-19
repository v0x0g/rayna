use crate::mat::Material;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use image::Pixel as _;
use rayna_shared::def::types::{Number, Pixel, Vector3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DielectricMaterial {
    pub albedo: Pixel,
    pub refractive_index: Number,
}

impl RtRequirement for DielectricMaterial {}

impl Material for DielectricMaterial {
    fn scatter(&self, intersection: &Intersection) -> Option<Vector3> {
        todo!()
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
