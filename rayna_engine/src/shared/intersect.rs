use crate::shared::ray::Ray;
use rayna_shared::def::types::{Num, Vec3};

/// A struct representing a ray-object intersection
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Intersection {
    /// The position in world coordinates of the intersection
    pub pos: Vec3,
    /// Surface normal at intersection.
    ///
    /// # Invariants
    ///     Must be normalised
    pub normal: Vec3,
    /// Distance along the ray that the intersection occurred
    pub dist: Num,
    /// Original ray
    pub ray: Ray,
}
