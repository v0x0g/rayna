use crate::mat::Material;
use crate::shared::intersect::Intersection;
use crate::shared::rng;
use rand::thread_rng;
use rayna_shared::def::types::Vector;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Diffuse {}

impl Material for Diffuse {
    fn scatter(&self, intersection: Intersection) -> Vector {
        // Completely random scatter direction
        // In same hemisphere as normal
        rng::vector_in_unit_hemisphere(&mut thread_rng(), intersection.normal)
    }
}
