use smallvec::SmallVec;

use rayna_shared::def::types::{Number, Point2, Point3};

use crate::accel::aabb::Aabb;
use crate::material::MaterialInstance;
use crate::object::planar::Planar;
use crate::object::{Object, ObjectInstance};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;

#[derive(Clone, Debug)]
pub struct ParallelogramBuilder {
    pub q: Point3,
    pub b: Point3,
    pub a: Point3,
    pub material: MaterialInstance,
}

#[derive(Clone, Debug)]
pub struct ParallelogramObject {
    plane: Planar,
    aabb: Aabb,
    material: MaterialInstance,
}

impl From<ParallelogramBuilder> for ParallelogramObject {
    fn from(p: ParallelogramBuilder) -> Self {
        let aabb = Aabb::encompass_points([p.q, p.a, p.b]).min_padded(super::planar::AABB_PADDING);
        let plane = Planar::new_points(p.q, p.a, p.b);
        Self {
            plane,
            aabb,
            material: p.material,
        }
    }
}

impl From<ParallelogramBuilder> for ObjectInstance {
    fn from(value: ParallelogramBuilder) -> Self {
        ParallelogramObject::from(value).into()
    }
}

impl Object for ParallelogramObject {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        let i = self.plane.intersect_bounded(ray, bounds, &self.material);
        // Check in bounds for our segment of the plane: `uv in [0, 1]`
        return match i {
            Some(i) if (i.uv.cmple(Point2::ONE) & i.uv.cmpge(Point2::ZERO)).all() => Some(i),
            _ => None,
        };
    }

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>) {
        // Planes won't intersect more than once, except in the parallel case
        // That's infinite intersections but we ignore that case
        self.intersect(ray, &Bounds::FULL).map(|i| output.push(i));
    }

    fn aabb(&self) -> Option<&Aabb> {
        Some(&self.aabb);
        None
    }
}
