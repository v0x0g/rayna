use getset::Getters;
use smallvec::SmallVec;

use rayna_shared::def::types::Number;

use crate::accel::aabb::Aabb;
use crate::accel::bvh::Bvh;
use crate::object::{Object, ObjectInstance};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;

#[derive(Clone, Debug, Getters)]
#[get = "pub"]
pub struct ObjectList {
    /// BVH-optimised tree of objects
    bvh: Bvh,
    /// All the unbounded objects in the list (objects where [Object::aabb()] returned [None]
    unbounded: Vec<ObjectInstance>,
}

// Iter<Into<ObjType>> => ObjectList
impl<Obj: Into<ObjectInstance>, Iter: IntoIterator<Item = Obj>> From<Iter> for ObjectList {
    fn from(value: Iter) -> Self {
        let mut bounded = vec![];
        let mut unbounded = vec![];
        for obj in value.into_iter().map(Obj::into) {
            if let Some(_) = obj.aabb() {
                bounded.push(obj);
            } else {
                unbounded.push(obj);
            }
        }
        let bvh = Bvh::new(&bounded);
        Self { bvh, unbounded }
    }
}

impl Object for ObjectList {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        let bvh_int = self.bvh.intersect(ray, bounds).into_iter();
        let unbound_int = self
            .unbounded
            .iter()
            .filter_map(|o| o.intersect(ray, bounds));
        Iterator::chain(bvh_int, unbound_int).min()
    }

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>) {
        self.bvh.intersect_all(ray, output);
        self.unbounded
            .iter()
            .for_each(|o| o.intersect_all(ray, output));
    }

    fn aabb(&self) -> Option<&Aabb> {
        // List may have unbounded objects, so we can't return Some()
        None
    }
}
