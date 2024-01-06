use getset::Getters;
use itertools::Itertools;
use smallvec::SmallVec;

use rayna_shared::def::types::Number;

use crate::accel::aabb::Aabb;
use crate::accel::bvh::Bvh;
use crate::object::{Object, ObjectType};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;

#[derive(Clone, Debug, Getters)]
#[get = "pub"]
pub struct ObjectList {
    raw: Vec<ObjectType>,
    bvh: Bvh,
    aabb: Aabb,
}

// Iter<Into<ObjType>> => ObjectList
impl<Obj: Into<ObjectType>, Iter: IntoIterator<Item = Obj>> From<Iter> for ObjectList {
    fn from(value: Iter) -> Self {
        let raw = value.into_iter().map(Into::into).collect_vec();
        let bvh = Bvh::new(&raw);
        let aabb = Aabb::encompass_iter(raw.iter().map(Object::aabb));
        Self { raw, bvh, aabb }
    }
}

impl Object for ObjectList {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        self.bvh.intersect(ray, bounds)
    }

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>) {
        self.bvh.intersect_all(ray, output);
    }

    fn aabb(&self) -> &Aabb {
        &self.aabb
    }
}
