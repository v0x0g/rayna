use crate::core::types::{Number, Point2, Point3, Vector3};
use crate::material::Material;
use derivative::Derivative;
use std::cmp::Ordering;

/// A struct representing a ray-mesh intersection
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Intersection {
    /// The position in world coordinates of the intersection
    pub pos_w: Point3,
    /// The position in mesh-local coordinates of the intersection
    pub pos_l: Point3,
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
    pub dist: Number,
    /// The UV coordinates for the point on the mesh's surface. Normally used for texture mapping.
    ///
    /// # Convention
    /// As a general rule, for any *bounded* face (one that doesn't extend to infinity along any direction),
    /// this should range from `0.0..=1.0` for both dimensions. If the surface is infinite (e.g. infinite ground plane),
    /// then it is acceptable to use unbounded UV coordinates, if not wrapping/mirroring them
    pub uv: Point2,
    /// Numeric ID for which "face" was hit
    ///
    /// For objects with a single 'surface' (like a [sphere](crate::mesh::sphere::SphereMesh), this would be always zero.
    /// For an mesh that may have multiple faces (like a [box](`crate::mesh::primitive::axis_box::AxisBoxMesh`), this would unique per-side.
    pub side: usize,
}

impl Eq for Intersection {}

impl PartialOrd<Self> for Intersection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Number::partial_cmp(&self.dist, &other.dist) }
}

impl Ord for Intersection {
    fn cmp(&self, other: &Self) -> Ordering {
        Number::partial_cmp(&self.dist, &other.dist)
            .expect("couldn't compare intersections distances: invariant `.dist != NaN` failed")
    }
}

/// A small wrapper class that includes a reference to a material as well as
/// the actual intersection with the model.
///
/// Only really used to provide a nicer return value for [`crate::object::Object::full_intersect()`].
#[derive(Clone, Debug, Derivative)]
#[derivative(Ord, PartialOrd, Eq, PartialEq)]
pub struct FullIntersection<'mat, Mat: Material> {
    pub intersection: Intersection,
    /// NOTE: For all comparisons, this field is ignored ([PartialEq], [Ord], [PartialOrd])
    #[derivative(PartialOrd = "ignore", Ord = "ignore", PartialEq = "ignore")]
    pub material: &'mat Mat,
}

impl<'mat, Mat: Material + 'mat> From<(&'mat Mat, Intersection)> for FullIntersection<'mat, Mat> {
    fn from((material, intersection): (&'mat Mat, Intersection)) -> Self { Self { intersection, material } }
}

impl Intersection {
    /// Converts a partial [`Intersection`] into a [`FullIntersection<Mat>`]
    pub fn make_full<'mat, Mat: Material>(self, material: &'mat Mat) -> FullIntersection<'mat, Mat> {
        FullIntersection {
            intersection: self,
            material,
        }
    }
}
