//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-mesh intersection tests by narrowing the search space,
//! by skipping meshes that obviously can't be intersected.

use getset::Getters;
use indextree::{Arena, NodeId};
use rand_core::RngCore;
use rayna_engine::core::types::{Number, Point3, Vector3};
use std::ops::{Add, Div};

use crate::mesh::{Mesh as MeshTrait, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::bounds::Bounds;
use crate::shared::generic_bvh::{GenericBvh, GenericBvhNode};
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;

#[derive(Getters, Clone, Debug)]
#[get = "pub"]
pub struct BvhMesh<Mesh: MeshTrait> {
    inner: GenericBvh<Mesh>,
    centre: Point3,
}

// region Constructors

impl<Mesh: MeshTrait> BvhMesh<Mesh> {
    /// Creates a new [BvhMesh] tree from the given slice of meshes
    ///
    /// # Note
    /// The given slice of `meshes` should only contain *bounded* meshes (i.e. [Mesh::aabb()] returns [`Some(_)`]).
    /// The exact behaviour is not specified, but will most likely result in a panic during building/accessing the tree
    pub fn new(meshes: Vec<Mesh>) -> Self {
        Self {
            centre: meshes
                .iter()
                .map(MeshProperties::centre)
                .map(Point3::to_vector)
                .fold(Vector3::ZERO, Vector3::add)
                .div(meshes.len() as Number)
                .to_point(),
            inner: GenericBvh::new(meshes),
        }
    }
}

// endregion Constructors

// region Mesh Impl

impl<Mesh: MeshTrait> BvhMesh<Mesh> {
    /// Given a [NodeId] on the [Arena] tree, calculates the nearest intersection for the given `ray` and `bounds`
    ///
    /// If the node is a [BvhNode::Mesh], it passes on the check to the mesh.
    /// Otherwise, if it's a [BvhNode::Aabb], it:
    ///     - Tries to bail early if the [Aabb] is missed
    ///     - Collects all the child nodes
    ///     - Intersects on all those children (by calling itself recursively)
    ///     - Returns the closest intersection of the above
    fn bvh_node_intersect(
        ray: &Ray,
        bounds: &Bounds<Number>,
        node: NodeId,
        arena: &Arena<GenericBvhNode<Mesh>>,
        rng: &mut dyn RngCore,
    ) -> Option<Intersection> {
        return match arena.get(node).expect("node should exist in arena").get() {
            // An aabb will need to delegate to child nodes if not missed
            GenericBvhNode::Nested(aabb) => {
                if !aabb.hit(ray, bounds) {
                    return None;
                }

                // TODO: Rework this to use the new Bounds::bitor API to shrink the next child's search range
                //  So keep track of the bounds, and each iteration shrink with `bounds = bounds | ..intersection.dist`
                //  And if an intersect was found in that shrunk range then we know that

                node.children(arena)
                    .filter_map(|child| Self::bvh_node_intersect(ray, bounds, child, arena, rng))
                    .min()
            }
            // meshes can be delegated directly
            GenericBvhNode::Object(mesh) => {
                if !mesh.aabb().expect("aabb missing").hit(ray, bounds) {
                    None
                } else {
                    mesh.intersect(ray, bounds, rng)
                }
            }
        };
    }
}

impl<Mesh: MeshTrait> MeshProperties for BvhMesh<Mesh> {
    fn centre(&self) -> Point3 { self.centre }
}

impl<Mesh: MeshTrait> MeshTrait for BvhMesh<Mesh> {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        // Pass everything on to our magical function
        Self::bvh_node_intersect(ray, bounds, self.inner.root_id()?, &self.inner.arena(), rng)
    }
}

impl<Obj: MeshTrait> HasAabb for BvhMesh<Obj> {
    fn aabb(&self) -> Option<&Aabb> {
        let root = self.inner.root_id()?;
        match self
            .inner
            .arena()
            .get(root)
            .expect(&format!("arena should contain root node {root}"))
            .get()
        {
            GenericBvhNode::Nested(aabb) => Some(aabb),
            GenericBvhNode::Object(o) => Some(o.expect_aabb()),
        }
    }
}

// endregion Mesh Impl
