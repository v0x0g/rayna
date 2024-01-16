use crate::mesh::bvh::BvhMesh;
use crate::mesh::{Mesh as MeshTrait, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use getset::Getters;
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Point3};

/// A group of meshes that are rendered as one mesh
///
/// # Notes
/// Since this only implements [Object], and not [crate::scene::FullObject], all the sub-objects
/// will share the same material (once placed inside a [crate::scene::SceneObject]
#[derive(Clone, Debug, Getters)]
#[get = "pub"]
pub struct MeshList<Mesh: MeshTrait> {
    unbounded: Vec<Mesh>,
    bounded: BvhMesh<Mesh>,
    /// The averaged centre of all the sub-objects
    centre: Point3,
    aabb: Option<Aabb>,
}

impl<Mesh: MeshTrait> MeshProperties for MeshList<Mesh> {
    fn centre(&self) -> Point3 { self.centre }
}

impl<Mesh: MeshTrait> HasAabb for MeshList<Mesh> {
    fn aabb(&self) -> Option<&Aabb> { self.aabb.as_ref() }
}

impl<Mesh: MeshTrait> MeshTrait for MeshList<Mesh> {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        let bvh_int = self.bounded.intersect(ray, bounds, rng).into_iter();
        let unbound_int = self.unbounded.iter().filter_map(|o| o.intersect(ray, bounds, rng));
        Iterator::chain(bvh_int, unbound_int).min()
    }
}
