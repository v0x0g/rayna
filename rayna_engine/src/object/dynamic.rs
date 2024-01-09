use crate::accel::aabb::Aabb;
use crate::object::Object;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use rayna_shared::def::types::Number;
use smallvec::SmallVec;
use std::sync::Arc;

/// Object wrapper around a `dyn` [Object]; Delegates everything to the inner object.
///
/// If possible use the enum variants on [super::ObjectInstance], so that static-dispatch is used instead of dynamic dispatch
#[derive(Clone, Debug)]
pub struct DynamicObject {
    pub inner: Arc<dyn Object>,
}

impl Object for DynamicObject {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        self.inner.intersect(ray, bounds)
    }

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>) {
        self.inner.intersect_all(ray, output)
    }

    fn aabb(&self) -> Option<&Aabb> {
        self.inner.aabb()
    }
}
