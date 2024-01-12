use crate::accel::aabb::Aabb;
use crate::object::{Object, ObjectProperties};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;
use rayna_shared::def::types::{Number, Point3};
use smallvec::SmallVec;
use std::sync::Arc;

/// Object wrapper around a `dyn` [Object]; Delegates everything to the inner object.
///
/// If possible use the enum variants on [super::ObjectInstance], so that static-dispatch is used instead of dynamic dispatch
#[derive(Clone, Debug)]
pub struct DynamicObject {
    pub inner: Arc<dyn Object>,
}

impl DynamicObject {
    pub fn from(value: impl Object + 'static) -> Self { Self { inner: Arc::new(value) } }
}

impl super::ObjectInstance {
    pub fn from_dyn(value: impl Object + 'static) -> Self { Self::from(DynamicObject::from(value)) }
}

impl Object for DynamicObject {
    fn intersect<'o>(&'o self, ray: &Ray, bounds: &Bounds<Number>) -> Option<FullIntersection<'o>> {
        self.inner.intersect(ray, bounds)
    }

    fn intersect_all<'o>(&'o self, ray: &Ray, output: &mut SmallVec<[FullIntersection<'o>; 32]>) {
        self.inner.intersect_all(ray, output)
    }
}

impl ObjectProperties for DynamicObject {
    fn aabb(&self) -> Option<&Aabb> { self.inner.aabb() }
    fn centre(&self) -> Point3 { self.inner.centre() }
}
