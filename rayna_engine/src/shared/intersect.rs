use crate::material::MaterialType;
use derivative::Derivative;
use rayna_shared::def::types::{Number, Point3, Vector3};
use std::cmp::Ordering;

/// A struct representing a ray-object intersection
#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq)]
pub struct Intersection {
    /// The position in world coordinates of the intersection
    pub pos: Point3,
    /// Surface normal at intersection.
    /// This should point in the *outwards* direction, irrespective of the
    /// incident ray
    ///
    /// # Invariants
    ///  - Must be normalised
    ///  - Cannot be zero/nan
    pub normal: Vector3,
    /// Surface normal at intersection.
    /// This should point in the *opposite* direction to the incident ray
    ///
    /// # Invariants
    /// - Must be normalised
    /// - Cannot be Zero/Nan
    pub ray_normal: Vector3,
    pub front_face: bool,
    /// Distance along the ray that the intersection occurred
    ///
    ///
    pub dist: Number,
    #[derivative(PartialEq = "ignore", PartialOrd = "ignore")]
    pub material: MaterialType,
}

impl Eq for Intersection {}

impl PartialOrd<Self> for Intersection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Number::partial_cmp(&self.dist, &other.dist)
    }
}

impl Ord for Intersection {
    fn cmp(&self, other: &Self) -> Ordering {
        Number::partial_cmp(&self.dist, &other.dist)
            .expect("couldn't compare intersections distances: invariant `.dist != NaN` failed")
    }
}
