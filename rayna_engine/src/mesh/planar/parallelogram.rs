use getset::CopyGetters;
use rand_core::RngCore;

use crate::core::types::{Number, Point2, Point3};

use crate::mesh::planar::Planar;
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;

#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct ParallelogramMesh {
    /// The plane that this mesh sits upon
    plane: Planar,
    aabb: Aabb,
    centre: Point3,
}

// region Constructors

impl ParallelogramMesh {
    pub fn new(plane: impl Into<Planar>) -> Self {
        let plane = plane.into();
        let (p, a, b, ab) = (
            plane.p(),
            plane.p() + plane.u(),
            plane.p() + plane.v(),
            plane.p() + plane.u() + plane.v(),
        );
        let centre = p + (plane.u() / 2.) + (plane.v() / 2.);
        let aabb = Aabb::encompass_points([p, a, b, ab]).min_padded(super::AABB_PADDING);

        Self { plane, aabb, centre }
    }
}

impl<T: Into<Planar>> From<T> for ParallelogramMesh {
    fn from(plane: T) -> Self { Self::new(plane) }
}

// endregion Constructors

// region Mesh Impl

impl Mesh for ParallelogramMesh {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        let i = self.plane.intersect_bounded(ray, interval)?;
        // Check in interval for our segment of the plane: `uv in [0, 1]`
        if (i.uv.cmple(Point2::ONE) & i.uv.cmpge(Point2::ZERO)).all() {
            Some(i)
        } else {
            None
        }
    }
}

impl HasAabb for ParallelogramMesh {
    fn aabb(&self) -> Option<&Aabb> { Some(&self.aabb) }
}
impl MeshProperties for ParallelogramMesh {
    fn centre(&self) -> Point3 { self.plane.p() }
}

// endregion Mesh Impl
