use getset::{CopyGetters, Getters};
use itertools::Itertools;
use rand_core::RngCore;

use crate::core::aabb::{Aabb, Bounded};
use crate::core::intersect::MeshIntersection;
use crate::core::interval::Interval;
use crate::core::ray::Ray;
use crate::core::types::Number;
use crate::mesh::{Mesh, MeshInstance, MeshToken};
use crate::scene::Scene;

/// A group of meshes that are rendered as one mesh
#[derive(Clone, Debug, Getters, CopyGetters)]
pub struct ListMesh {
    #[get_copy = "pub"]
    aabb: Aabb,
    #[get = "pub"]
    tokens: Vec<MeshToken>,
}

// region Constructors

impl ListMesh {
    /// Creates a list of meshes that have already been inserted into the scene
    pub fn new_from(scene: &Scene, tokens: impl IntoIterator<Item: Into<MeshToken>>) -> Self {
        let tokens = tokens.into_iter().map(Into::into).collect_vec();
        let aabb = Aabb::encompass_iter(tokens.iter().map(|t| scene.get_mesh(t).aabb()));
        Self { aabb, tokens }
    }

    /// Creates a list of meshes, adding them to the scene
    pub fn new_in(scene: &mut Scene, meshes: impl IntoIterator<Item: Into<MeshInstance>>) -> Self {
        let meshes = meshes.into_iter().map(Into::into).collect_vec();
        let aabb = Aabb::encompass_iter(meshes.iter().map(Bounded::aabb));
        let tokens = Vec::from_iter(meshes.into_iter().map(|m| scene.add_mesh(m)));
        Self { aabb, tokens }
    }
}

// endregion Constructors

// region Mesh Impl
impl Bounded for ListMesh {
    fn aabb(&self) -> Aabb {
        self.aabb
    }
}

impl Mesh for ListMesh {
    fn intersect(
        &self,
        scene: &Scene,
        ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<MeshIntersection> {
        self.tokens
            .iter()
            .map(|t| scene.get_mesh(t))
            .filter_map(|m| m.intersect(scene, ray, interval, rng))
            .min()
    }
}

// endregion Mesh Impl
