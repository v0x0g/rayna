use smallvec::SmallVec;

use rayna_shared::def::types::{Number, Point2, Point3};

use crate::accel::aabb::Aabb;
use crate::material::MaterialType;
use crate::object::planar::Planar;
use crate::object::{Object, ObjectType};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;

#[derive(Clone, Debug)]
pub struct ParallelogramBuilder {
    pub corner_origin: Point3,
    pub corner_upper: Point3,
    pub corner_right: Point3,
    pub material: MaterialType,
}

#[derive(Clone, Debug)]
pub struct ParallelogramObject {
    plane: Planar,
    aabb: Aabb,
    material: MaterialType,
}

impl From<ParallelogramBuilder> for ParallelogramObject {
    fn from(p: ParallelogramBuilder) -> Self {
        let aabb = Aabb::encompass_points([p.corner_origin, p.corner_right, p.corner_upper])
            .min_padded(super::planar::AABB_PADDING);
        let plane = Planar::new_points(p.corner_origin, p.corner_right, p.corner_upper);
        Self {
            plane,
            aabb,
            material: p.material,
        }
    }
}

impl From<ParallelogramBuilder> for ObjectType {
    fn from(value: ParallelogramBuilder) -> Self {
        ParallelogramObject::from(value).into()
    }
}

impl Object for ParallelogramObject {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        let i = self.plane.intersect_bounded(ray, bounds, &self.material);
        // Check in bounds for our segment of the plane: `uv in [0, 1]`
        if i.is_some_and(|i| (i.uv.cmple(Point2::ONE) & i.uv.cmpge(Point2::ZERO)).all()) {
            i
        } else {
            None
        }
    }

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>) {
        // Planes won't intersect more than once, except in the parallel case
        // That's infinite intersections but we ignore that case
        self.intersect(ray, &Bounds::FULL).map(|i| output.push(i));
    }

    fn aabb(&self) -> Option<&Aabb> {
        Some(&self.aabb)
    }
}
