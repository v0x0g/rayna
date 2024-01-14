use getset::Getters;
use smallvec::SmallVec;

use crate::accel::aabb::Aabb;
use crate::accel::bvh::Bvh;
use crate::scene::{FullObject, SceneObject};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;
use rayna_shared::def::types::Number;

#[derive(Clone, Debug, Getters)]
#[get = "pub"]
pub struct SceneObjectList {
    /// BVH-optimised tree of objects
    bvh: Bvh<SceneObject>,
    /// All the unbounded objects in the list (objects where [Object::aabb()] returned [None]
    unbounded: Vec<SceneObject>,
}

// Iter<Into<ObjType>> => ObjectList
impl<Iter: IntoIterator<Item = SceneObject>> From<Iter> for SceneObjectList {
    fn from(value: Iter) -> Self {
        let mut bounded = vec![];
        let mut unbounded = vec![];
        for obj in value.into_iter() {
            if let Some(_) = obj.aabb() {
                bounded.push(obj);
            } else {
                unbounded.push(obj);
            }
        }
        let bvh = Bvh::new(bounded);
        Self { bvh, unbounded }
    }
}

impl FullObject for SceneObjectList {
    fn full_intersect<'o>(
        &'o self,
        ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o>> {
        let bvh_int = self.bvh.full_intersect(ray, bounds).into_iter();
        let unbound_int = self.unbounded.iter().filter_map(|o| o.full_intersect(ray, bounds));
        Iterator::chain(bvh_int, unbound_int).min()
    }

    fn full_intersect_all<'o>(&'o self, ray: &Ray, output: &mut SmallVec<[FullIntersection<'o>; 32]>) {
        self.bvh.full_intersect_all(ray, output);
        self.unbounded.iter().for_each(|o| o.full_intersect_all(ray, output));
    }

    fn aabb(&self) -> Option<&Aabb> {
        // List may have unbounded objects, so we can't return Some()
        None
    }
}
