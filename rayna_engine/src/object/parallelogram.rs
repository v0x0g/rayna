use getset::CopyGetters;
use rand_core::RngCore;
use smallvec::SmallVec;

use rayna_shared::def::types::{Number, Point2, Point3};

use crate::accel::aabb::Aabb;
use crate::object::planar::{Planar, PlanarBuilder};
use crate::object::{Object, ObjectInstance, ObjectProperties};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;

#[derive(Copy, Clone, Debug)]
pub struct ParallelogramBuilder {
    pub plane: PlanarBuilder,
}

#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct ParallelogramObject {
    /// The plane that this object sits upon
    plane: Planar,
    aabb: Aabb,
    centre: Point3,
}

impl From<ParallelogramBuilder> for ParallelogramObject {
    fn from(builder: ParallelogramBuilder) -> Self {
        let plane = Planar::from(builder.plane);
        let (p, a, b) = (plane.p(), plane.p() + plane.u(), plane.p() + plane.v());
        let centre = p + (plane.u() / 2.) + (plane.v() / 2.);
        let aabb = Aabb::encompass_points([p, a, b]).min_padded(super::planar::AABB_PADDING);

        Self { plane, aabb, centre }
    }
}

impl From<ParallelogramBuilder> for ObjectInstance {
    fn from(value: ParallelogramBuilder) -> Self { ParallelogramObject::from(value).into() }
}

impl Object for ParallelogramObject {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        let i = self.plane.intersect_bounded(ray, bounds)?;
        // Check in bounds for our segment of the plane: `uv in [0, 1]`
        if (i.uv.cmple(Point2::ONE) & i.uv.cmpge(Point2::ZERO)).all() {
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

impl ObjectProperties for ParallelogramObject {
    fn aabb(&self) -> Option<&Aabb> { Some(&self.aabb) }
    fn centre(&self) -> Point3 { self.plane.p() }
}
