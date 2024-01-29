use crate::mesh::advanced::bvh::BvhMesh;
use crate::mesh::{Mesh as MeshTrait, MeshInstance, MeshProperties};

use crate::core::types::{Number, Point3};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use getset::Getters;
use rand_core::RngCore;

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

// region Constructors

impl<Mesh: MeshTrait> MeshList<Mesh> {
    pub fn new<IntoMesh: Into<Mesh>>(meshes: impl IntoIterator<Item = IntoMesh>) -> Self {
        let mut bounded = vec![];
        let mut unbounded = vec![];
        let mut centre = Point3::ZERO;

        for mesh in meshes.into_iter().map(IntoMesh::into) {
            centre += mesh.centre().to_vector();
            if let Some(_) = mesh.aabb() {
                bounded.push(mesh);
            } else {
                unbounded.push(mesh);
            }
        }

        let aabb = if unbounded.is_empty() && !bounded.is_empty() {
            // All objects were checked for AABB so can unwrap
            Some(Aabb::encompass_iter(bounded.iter().map(Mesh::aabb).map(Option::unwrap)))
        } else {
            None
        };

        let bvh = BvhMesh::new(bounded);

        Self {
            unbounded,
            bounded: bvh,
            centre,
            aabb,
        }
    }
}

/// Create MeshList<M> from iterator
impl<Mesh: MeshTrait, IntoMesh: Into<Mesh>, Iter: IntoIterator<Item = IntoMesh>> From<Iter> for MeshList<Mesh> {
    fn from(meshes: Iter) -> Self { Self::new(meshes) }
}

/// Create (MeshList<M> as MeshInstance) from iterator of MeshInstance
impl<IntoMesh: Into<MeshInstance>, Iter: IntoIterator<Item = IntoMesh>> From<Iter> for MeshInstance {
    fn from(meshes: Iter) -> Self { MeshList::<MeshInstance>::new(meshes.into_iter().map(IntoMesh::into)).into() }
}

// endregion Constructors

// region Mesh Impl

impl<Mesh: MeshTrait> MeshProperties for MeshList<Mesh> {
    fn centre(&self) -> Point3 { self.centre }
}

impl<Mesh: MeshTrait> HasAabb for MeshList<Mesh> {
    fn aabb(&self) -> Option<&Aabb> { self.aabb.as_ref() }
}

impl<Mesh: MeshTrait> MeshTrait for MeshList<Mesh> {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        let bvh_int = self.bounded.intersect(ray, interval, rng).into_iter();
        let unbound_int = self.unbounded.iter().filter_map(|o| o.intersect(ray, interval, rng));
        Iterator::chain(bvh_int, unbound_int).min()
    }
}

// endregion Mesh Impl
