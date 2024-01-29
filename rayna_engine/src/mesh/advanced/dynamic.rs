use crate::core::types::{Number, Point3};
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use rand_core::RngCore;

use std::sync::Arc;

/// Object wrapper around a `dyn` [Mesh]; Delegates everything to the inner mesh.
///
/// If possible use the enum variants on [super::MeshInstance], so that static-dispatch is used instead of dynamic dispatch
#[derive(Clone, Debug)]
pub struct DynamicMesh {
    pub inner: Arc<dyn Mesh>,
}

impl DynamicMesh {
    pub fn new(value: impl Mesh + 'static) -> Self { Self { inner: Arc::new(value) } }
}

// TODO: Create an `impl<T: Mesh> From<T> for DynamicMesh`
//  This will probably need specialisation or type inequality interval
//  else it will overlap with `impl<T> From<T> for T`, where `T == DynamicMesh`

impl super::super::MeshInstance {
    pub fn from_dyn(value: impl Mesh + 'static) -> Self { Self::from(DynamicMesh::new(value)) }
}

impl Mesh for DynamicMesh {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        self.inner.intersect(ray, interval, rng)
    }
}

impl HasAabb for DynamicMesh {
    fn aabb(&self) -> Option<&Aabb> { self.inner.aabb() }
}
impl MeshProperties for DynamicMesh {
    fn centre(&self) -> Point3 { self.inner.centre() }
}
