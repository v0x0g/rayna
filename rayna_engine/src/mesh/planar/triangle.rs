use getset::CopyGetters;
use rand_core::RngCore;

use crate::core::types::{Number, Point3};

use crate::mesh::planar::Planar;
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;

#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct TriangleMesh {
    /// The plane that this mesh sits upon
    plane: Planar,
    aabb: Aabb,
    centre: Point3,
}

// region Constructors

impl TriangleMesh {
    pub fn new(plane: impl Into<Planar>) -> Self {
        let plane = plane.into();
        let (p, a, b) = (plane.p(), plane.p() + plane.u(), plane.p() + plane.v());
        let centre = p + (plane.u() / 2.) + (plane.v() / 2.);
        let aabb = Aabb::encompass_points([p, a, b]).min_padded(super::AABB_PADDING);

        Self { plane, aabb, centre }
    }
}

// endregion Constructors

impl<P: Into<Planar>> From<P> for TriangleMesh {
    fn from(plane: P) -> Self { Self::new(plane) }
}

// region Mesh Impl

impl Mesh for TriangleMesh {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        let i = self.plane.intersect_bounded(ray, interval)?;
        // Check in interval for our segment of the plane: `u + v: [0..1]`
        // TODO: Bayesian coordinates for triangles??
        if (i.uv.x + i.uv.y) < 1. {
            Some(i)
        } else {
            None
        }
    }
}

impl HasAabb for TriangleMesh {
    fn aabb(&self) -> Option<&Aabb> { Some(&self.aabb) }
}
impl MeshProperties for TriangleMesh {
    fn centre(&self) -> Point3 { self.plane.p() }
}

// endregion Mesh Impl
