use getset::CopyGetters;
use rand_core::RngCore;
use smallvec::SmallVec;

use rayna_shared::def::types::{Number, Point3};

use crate::mesh::planar::{Planar, PlanarBuilder};
use crate::mesh::{Mesh, MeshInstance, MeshProperties};
use crate::shared::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;

#[derive(Copy, Clone, Debug)]
pub struct TriangleBuilder {
    pub plane: PlanarBuilder,
}

#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct TriangleMesh {
    /// The plane that this mesh sits upon
    plane: Planar,
    aabb: Aabb,
    centre: Point3,
}

impl From<TriangleBuilder> for TriangleMesh {
    fn from(builder: TriangleBuilder) -> Self {
        let plane = Planar::from(builder.plane);
        let (p, a, b) = (plane.p(), plane.p() + plane.u(), plane.p() + plane.v());
        let centre = p + (plane.u() / 2.) + (plane.v() / 2.);
        let aabb = Aabb::encompass_points([p, a, b]).min_padded(super::planar::AABB_PADDING);

        Self { plane, aabb, centre }
    }
}

impl From<TriangleBuilder> for MeshInstance {
    fn from(value: TriangleBuilder) -> Self { TriangleMesh::from(value).into() }
}

impl Mesh for TriangleMesh {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        let i = self.plane.intersect_bounded(ray, bounds)?;
        // Check in bounds for our segment of the plane: `u + v: [0..1]`
        // TODO: Bayesian coordinates for triangles??
        if (i.uv.x + i.uv.y) < 1. {
            Some(i)
        } else {
            None
        }
    }

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>, rng: &mut dyn RngCore) {
        // Planes won't intersect more than once, except in the parallel case
        // That's infinite intersections but we ignore that case
        self.intersect(ray, &Bounds::FULL, rng).map(|i| output.push(i));
    }
}

impl MeshProperties for TriangleMesh {
    fn aabb(&self) -> Option<&Aabb> { Some(&self.aabb) }
    fn centre(&self) -> Point3 { self.plane.p() }
}
