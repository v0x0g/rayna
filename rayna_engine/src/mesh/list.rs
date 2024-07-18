use getset::{CopyGetters, Getters};
use itertools::Itertools;
use rand_core::RngCore;

use crate::core::types::Number;
use crate::mesh::{Mesh, MeshInstance, MeshToken};
use crate::scene::Scene;
use crate::shared::aabb::{Aabb, Bounded};
use crate::shared::intersect::MeshIntersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;

/// A group of meshes that are rendered as one mesh
#[derive(Clone, Debug, Getters, CopyGetters)]
pub struct ListMesh {
    #[get_copy = "pub"]
    aabb: Aabb,
    #[get = "pub"]
    meshes: Vec<MeshToken>,
}

// region Constructors

impl ListMesh {
    /// Creates a list of meshes that have already been inserted into the scene
    pub fn new_from(scene: &Scene, tokens: impl IntoIterator<Item: Into<MeshToken>>) -> Self {
        let tokens = tokens.into_iter().map(Into::into).collect_vec();
        let aabb = Aabb::encompass_iter(tokens.iter().map(|t| scene.get_mesh(t).aabb()));
        Self { aabb, meshes: tokens }
    }

    /// Creates a list of meshes, adding them to the scene
    pub fn new_in(scene: &mut Scene, meshes: impl IntoIterator<Item: Into<MeshInstance>>) -> Self {
        let meshes = meshes.into_iter().map(Into::into);
        let aabb = Aabb::encompass_iter(meshes.iter().map(|m| m.aabb()));
        let tokens = meshes.into_iter().map(|m| scene.add_mesh(m));
        Self { aabb, meshes: tokens }
    }
}

/// Create `ListMesh` from iterator of [`MeshToken`]
impl<Iter: IntoIterator<Item: Into<MeshToken>>> From<Iter> for ListMesh {
    fn from(meshes: Iter) -> Self { Self::new(meshes) }
}

/// Create ([`ListMesh`] as [`MeshInstance`]) from iterator of [`MeshToken`]
impl<Iter: IntoIterator<Item: Into<MeshToken>>> From<Iter> for MeshInstance {
    fn from(meshes: Iter) -> Self { ListMesh::new(meshes.into_iter().map(Into::into)).into() }
}

// endregion Constructors

// region Mesh Impl
impl Bounded for ListMesh {
    fn aabb(&self) -> Aabb { self.aabb }
}

impl Mesh for ListMesh {
    fn intersect(
        &self,
        scene: &Scene,
        ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<MeshIntersection> {
        self.meshes
            .iter()
            .map(|t| scene.get_mesh(t))
            .filter_map(|m| m.intersect(scene, ray, interval, rng))
            .min()
    }
}

// endregion Mesh Impl
