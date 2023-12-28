use crate::accel::aabb::Aabb;
use crate::object::Object;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use rayna_shared::def::types::Number;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DynamicObject {
    pub inner: Arc<dyn Object>,
}

impl Object for DynamicObject {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        self.inner.intersect(ray, bounds)
    }

    fn intersect_all(&self, ray: &Ray) -> Option<Box<dyn Iterator<Item = Intersection> + '_>> {
        self.inner.intersect_all(ray)
    }

    fn bounding_box(&self) -> &Aabb {
        self.inner.bounding_box()
    }
}
