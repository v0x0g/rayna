//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-mesh intersection tests by narrowing the search space,
//! by skipping meshes that obviously can't be intersected.

use crate::core::types::{Number, Point3, Vector3};
use getset::Getters;
use indextree::{Arena, NodeId};
use rand_core::RngCore;
use std::ops::{Add, Div};

use crate::mesh::{Mesh as MeshTrait, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::generic_bvh::{GenericBvh, GenericBvhNode};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use crate::shared::validate;

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
    /// The given slice of `meshes` should only contain *bounded* meshes (i.e. [`HasAabb::aabb()`] returns [`Some(_)`]).
    /// The exact behaviour is not specified, but will most likely result in a panic during building/accessing the tree
    pub fn new(meshes: Vec<Mesh>) -> Self {
        Self {
            // Pretty shit approximation, averages all the centres of sub-meshes
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
    /// Given a [NodeId] on the [Arena] tree, calculates the nearest intersection for the given `ray` and `interval`
    ///
    /// If the node is a [BvhNode::Mesh], it passes on the check to the mesh.
    /// Otherwise, if it's a [BvhNode::Aabb], it:
    ///     - Tries to bail early if the [Aabb] is missed
    ///     - Collects all the child nodes
    ///     - Intersects on all those children (by calling itself recursively)
    ///     - Returns the closest intersection of the above
    fn bvh_node_intersect(
        ray: &Ray,
        interval: &Interval<Number>,
        node: NodeId,
        arena: &Arena<GenericBvhNode<Mesh>>,
        rng: &mut dyn RngCore,
    ) -> Option<Intersection> {
        return match arena.get(node).expect("node should exist in arena").get() {
            // An aabb will need to delegate to child nodes if not missed
            GenericBvhNode::Nested(aabb) => {
                if !aabb.hit(ray, interval) {
                    return None;
                }

                // PERF: This shrinks the ray interval for each child, so that we only ever
                //  check for intersections closer than the current closest.
                let mut shrunk_interval = *interval;
                let mut closest_intersect = None;
                for child in node.children(arena) {
                    let Some(intersect) = Self::bvh_node_intersect(ray, &shrunk_interval, child, arena, rng) else {
                        continue;
                    };

                    validate::intersection(ray, &intersect, &shrunk_interval);
                    shrunk_interval = shrunk_interval.with_some_end(intersect.dist);
                    closest_intersect = Some(intersect)
                }

                closest_intersect
            }
            // meshes can be delegated directly
            GenericBvhNode::Object(mesh) => {
                if !mesh.expect_aabb().hit(ray, interval) {
                    None
                } else {
                    mesh.intersect(ray, interval, rng)
                }
            }
        };
    }
}

impl<Mesh: MeshTrait> MeshProperties for BvhMesh<Mesh> {
    fn centre(&self) -> Point3 { self.centre }
}

impl<Mesh: MeshTrait> MeshTrait for BvhMesh<Mesh> {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        // Pass everything on to our magical function
        Self::bvh_node_intersect(ray, interval, self.inner.root_id()?, &self.inner.arena(), rng)
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
