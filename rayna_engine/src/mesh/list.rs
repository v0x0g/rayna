use getset::{CopyGetters, Getters};
use itertools::Itertools;
use rand_core::RngCore;

use crate::core::types::Number;
use crate::mesh::{Mesh, MeshInstance};
use crate::shared::aabb::{Aabb, Bounded};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;

/// A group of meshes that are rendered as one mesh
///
/// # Notes
/// Since this only implements [`MeshTrait`], and not [`crate::object::Object`], all the sub-objects
/// will share the same material (once placed inside an actual object instance).
#[derive(Clone, Debug, Getters, CopyGetters)]
pub struct ListMesh {
    #[get_copy = "pub"]
    aabb: Aabb,
    // TODO: Store MeshInstance or MeshToken?
    #[get = "pub"]
    items: Vec<MeshInstance>,
}

// region Constructors

impl ListMesh {
    pub fn new(meshes: impl IntoIterator<Item: Into<MeshInstance>>) -> Self {
        let items = meshes.into_iter().map(Into::into).collect_vec();
        let aabb = Aabb::encompass_iter(items.iter().map(|m| m.aabb()));
        Self { aabb, items }
    }
}

/// Create `MeshList` from iterator of Meshes
impl<Iter: IntoIterator<Item: Into<MeshInstance>>> From<Iter> for ListMesh {
    fn from(meshes: Iter) -> Self { Self::new(meshes) }
}

/// Create (`MeshList as MeshInstance`) from iterator of `MeshInstance`
impl<Iter: IntoIterator<Item: Into<MeshInstance>>> From<Iter> for MeshInstance {
    fn from(meshes: Iter) -> Self { ListMesh::new(meshes.into_iter().map(Into::into)).into() }
}

// endregion Constructors

// region Mesh Impl
impl Bounded for ListMesh {
    fn aabb(&self) -> Aabb { self.aabb }
}

impl Mesh for ListMesh {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        self.items.iter().filter_map(|o| o.intersect(ray, interval, rng)).min()
    }
}

// endregion Mesh Impl
