use crate::mesh::{Object, ObjectProperties};
use crate::shared::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Point3};
use smallvec::SmallVec;
use std::sync::Arc;

/// Object wrapper around a `dyn` [Object]; Delegates everything to the inner mesh.
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
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        self.inner.intersect(ray, bounds, rng)
    }

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>, rng: &mut dyn RngCore) {
        self.inner.intersect_all(ray, output, rng)
    }
}

impl ObjectProperties for DynamicObject {
    fn aabb(&self) -> Option<&Aabb> { self.inner.aabb() }
    fn centre(&self) -> Point3 { self.inner.centre() }
}
