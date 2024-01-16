use crate::mesh::{Mesh as MeshTrait, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::bounds::Bounds;
use crate::shared::generic_bvh::GenericBvh;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use getset::Getters;
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Point3};
use smallvec::SmallVec;

/// A group of meshes that are rendered as one mesh
///
/// # Notes
/// Since this only implements [Object], and not [crate::scene::FullObject], all the sub-objects
/// will share the same material (once placed inside a [crate::scene::SceneObject]
#[derive(Clone, Debug, Getters)]
#[get = "pub"]
pub struct GroupMesh<Mesh: MeshTrait> {
    unbounded: Vec<Mesh>,
    bounded: GenericBvh<Mesh>,
    /// The averaged centre of all the sub-objects
    centre: Point3,
    aabb: Option<Aabb>,
}

impl<Mesh: MeshTrait> MeshProperties for GroupMesh<Mesh> {
    fn centre(&self) -> Point3 { self.centre }
}

impl<Mesh: MeshTrait> HasAabb for GroupMesh<Mesh> {
    fn aabb(&self) -> Option<&Aabb> { self.aabb.as_ref() }
}

impl<Mesh: MeshTrait> MeshTrait for GroupMesh<Mesh> {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {}
}
