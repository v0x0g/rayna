//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-mesh intersection tests by narrowing the search space,
//! by skipping objects that obviously can't be intersected.

use derivative::Derivative;
use getset::Getters;
use indextree::{Arena, NodeId};
use std::marker::PhantomData;

use rand_core::RngCore;
use smallvec::SmallVec;

use crate::material::Material;
use rayna_shared::def::types::Number;

use crate::object::Object;
use crate::shared::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::generic_bvh::{GenericBvh, GenericBvhNode};
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;

#[derive(Getters, Derivative)]
#[get = "pub"]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct BvhObject<Mesh, Mat, Obj>
where
    Mesh: crate::mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh = Mesh, Mat = Mat> + Clone,
{
    inner: GenericBvh<Obj>,
    phantom_mesh: PhantomData<Mesh>,
    phantom_mat: PhantomData<Mat>,
}

/// Helper function to unwrap an AABB with a panic message
fn expect_aabb<Obj: Object + Clone>(o: &Obj) -> &Aabb {
    o.aabb().as_ref().expect("aabb required as invariant of `Bvh`")
}

impl<Mesh, Mat, Obj> BvhObject<Mesh, Mat, Obj>
where
    Mesh: crate::mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh = Mesh, Mat = Mat> + Clone,
{
    /// Creates a new [BvhObject] tree from the given slice of objects
    ///
    /// # Note
    /// The given slice of `objects` should only contain *bounded* objects (i.e. [Object::aabb()] returns [`Some(_)`]).
    /// The exact behaviour is not specified, but will most likely result in a panic during building/accessing the tree
    pub fn new(objects: Vec<Obj>) -> Self {
        Self {
            inner: GenericBvh::new(objects),
            phantom_mat: PhantomData,
            phantom_mesh: PhantomData,
        }
    }
}

impl<Mesh, Mat, Obj> BvhObject<Mesh, Mat, Obj>
where
    Mesh: crate::mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh = Mesh, Mat = Mat> + Clone,
{
    /// Given a [NodeId] on the [Arena] tree, calculates the nearest intersection for the given `ray` and `bounds`
    ///
    /// If the node is a [BvhNode::Object], it passes on the check to the mesh.
    /// Otherwise, if it's a [BvhNode::Aabb], it:
    ///     - Tries to bail early if the [Aabb] is missed
    ///     - Collects all the child nodes
    ///     - Intersects on all those children (by calling itself recursively)
    ///     - Returns the closest intersection of the above
    fn bvh_node_intersect<'o>(
        ray: &Ray,
        bounds: &Bounds<Number>,
        node: NodeId,
        arena: &'o Arena<GenericBvhNode<Obj>>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Mat>> {
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
            // Objects can be delegated directly
            GenericBvhNode::Object(obj) => {
                if !obj.aabb().expect("aabb missing").hit(ray, bounds) {
                    None
                } else {
                    obj.full_intersect(ray, bounds, rng)
                }
            }
        };
    }

    /// Given a [NodeId] on the [Arena] tree, calculates the ALL intersection for the given `ray`
    ///
    /// If the node is a [BvhNode::Object], it passes on the check to the mesh.
    /// Otherwise, if it's a [BvhNode::Aabb], it:
    ///  - Tries to bail early if the [Aabb] is missed
    ///  - Collects all the child nodes
    ///  - Intersects on all those children (by calling itself recursively)
    fn bvh_node_intersect_all<'o>(
        ray: &Ray,
        node: NodeId,
        arena: &'o Arena<GenericBvhNode<Obj>>,
        output: &mut SmallVec<[FullIntersection<'o, Mat>; 32]>,
        rng: &mut dyn RngCore,
    ) {
        match arena.get(node).expect("node should exist in arena").get() {
            // An aabb will need to delegate to child nodes if not missed
            GenericBvhNode::Nested(aabb) => {
                if !aabb.hit(ray, &Bounds::FULL) {
                    return;
                }

                node.children(arena)
                    .for_each(|child| Self::bvh_node_intersect_all(ray, child, arena, output, rng));
            }
            // Objects can be delegated directly
            GenericBvhNode::Object(obj) => {
                if !expect_aabb(obj).hit(ray, &Bounds::FULL) {
                    return;
                }
                obj.full_intersect_all(ray, output, rng)
            }
        }

        // // Possibly faster method, doesn't do any tree traversal at all, should be linear
        // arena
        //     .iter()
        //     .filter(|node| !node.is_removed())
        //     .filter_map(|node| match node.get() {BvhNode::Object(o) => Some(o), _ => None})
        //     .for_each(|obj| obj.intersect_all(ray, output))
    }
}

impl<Mesh, Mat, Obj> Object for BvhObject<Mesh, Mat, Obj>
where
    Mesh: crate::mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh = Mesh, Mat = Mat> + Clone,
{
    type Mesh = Mesh;
    type Mat = Mat;

    fn full_intersect<'o>(
        &'o self,
        ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Mat>> {
        // Pass everything on to our magical function
        Self::bvh_node_intersect(ray, bounds, self.inner.root_id()?, &self.inner.arena(), rng)
    }

    fn full_intersect_all<'o>(
        &'o self,
        ray: &Ray,
        output: &mut SmallVec<[FullIntersection<'o, Mat>; 32]>,
        rng: &mut dyn RngCore,
    ) {
        if let Some(root) = self.inner.root_id() {
            Self::bvh_node_intersect_all(ray, root, self.inner.arena(), output, rng);
        }
    }

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
            GenericBvhNode::Object(o) => Some(expect_aabb(o)),
        }
    }
}
