use getset::Getters;
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
    /// BVH-optimised tree of objects
    bvh: Bvh,
    /// [Aabb] for the bounded collection of objects
    aabb: Aabb,
    /// All the unbounded objects in the list (objects where [Object::aabb()] returned [None]
    unbounded: Vec<ObjectType>,
}

// Iter<Into<ObjType>> => ObjectList
impl<Obj: Into<ObjectType>, Iter: IntoIterator<Item = Obj>> From<Iter> for ObjectList {
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
        let aabb = Aabb::encompass_iter(
            bounded
                .iter()
                .map(|o| o.aabb().expect("already filtered out unbounded objects")),
        );
        Self {
            bvh,
            aabb,
            unbounded,
        }
    }
}

impl Object for ObjectList {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        self.bvh.intersect(ray, bounds)
    }

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>) {
        self.bvh.intersect_all(ray, output);
    }

    fn aabb(&self) -> Option<&Aabb> {
        // List may have unbounded objects, so we can't return Some()
        None
    }
}
