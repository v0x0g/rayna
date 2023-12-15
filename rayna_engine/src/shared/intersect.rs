use crate::mat::MaterialType;
use crate::shared::ray::Ray;
use derivative::Derivative;
use rayna_shared::def::types::{Number, Vector};

/// A struct representing a ray-object intersection
#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq)]
pub struct Intersection {
    /// The position in world coordinates of the intersection
    pub pos: Vector,
    /// Surface normal at intersection.
    /// This should point in the *outwards* direction, irrespective of the
    /// incident ray
    ///
    /// # Invariants
    ///     Must be normalised
    pub normal: Vector,
    /// Surface normal at intersection.
    /// This should point in the *opposite* direction to the incident ray
    ///
    /// # Invariants
    ///     Must be normalised
    pub ray_normal: Vector,
    /// Distance along the ray that the intersection occurred
    pub dist: Number,
    /// Original ray
    pub ray: Ray,
    #[derivative(PartialEq = "ignore")]
    pub material: MaterialType,
}
