use crate::mat::Material;
use crate::shared::intersect::Intersection;
use crate::shared::{rng, RtRequirement};
use rand::thread_rng;
use rayna_shared::def::types::Vector;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DiffuseMaterial {}

impl RtRequirement for DiffuseMaterial {}

impl Material for DiffuseMaterial {
    fn scatter(&self, intersection: &Intersection) -> Vector {
        // Completely random scatter direction
        // In same hemisphere as normal
        rng::vector_in_unit_hemisphere(&mut thread_rng(), intersection.normal)
    }
}
