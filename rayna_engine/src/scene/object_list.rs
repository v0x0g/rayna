use getset::Getters;
use itertools::Itertools;

use rayna_shared::def::types::Number;

use crate::accel::aabb::Aabb;
use crate::accel::bvh::Bvh;
use crate::object::{Object, ObjectType};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::validate;

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
        let aabb = Aabb::encompass_iter(raw.iter().map(Object::bounding_box));
        Self { raw, bvh, aabb }
    }
}

impl Object for ObjectList {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        self.raw
            .iter()
            // Intersect all and only include hits not misses
            .filter_map(|obj| obj.intersect(ray, bounds))
            .inspect(|i| validate::intersection(ray, i, bounds))
            // Choose closest intersect
            .min_by(|a, b| Number::total_cmp(&a.dist, &b.dist))
    }

    fn intersect_all<'a>(
        &'a self,
        ray: &'a Ray,
    ) -> Option<Box<dyn Iterator<Item = Intersection> + 'a>> {
        let mut iter = self
            .raw
            .iter()
            .filter_map(|obj| obj.intersect_all(ray)) // Iterator<Box<dyn Iterator>>
            .flatten()
            .peekable();
        // Ensure we have at least one item in the iter
        iter.peek()?;
        Some(Box::new(iter))
    }

    fn bounding_box(&self) -> &Aabb {
        &self.aabb
    }
}
