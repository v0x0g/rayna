//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-mesh intersection tests by narrowing the search space,
//! by skipping objects that obviously can't be intersected.

use getset::Getters;
use indextree::{Arena, NodeId};
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Point3};

use crate::object::transform::ObjectTransform;
use crate::object::Object;
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::bounds::Bounds;
use crate::shared::generic_bvh::{GenericBvh, GenericBvhNode};
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;

#[derive(Getters, Clone, Debug)]
#[get = "pub"]
pub struct BvhObject<Obj: Object> {
    inner: GenericBvh<Obj>,
    transform: ObjectTransform,
    #[get(skip)]
    aabb: Option<Aabb>,
}

impl<Obj: Object> BvhObject<Obj> {
    /// See [super::simple::SimpleObject::new()]
    ///
    /// # Panics
    /// See [Self::new_uncorrected()]
    pub fn new(
        objects: impl IntoIterator<Item = Obj>,
        transform: impl Into<ObjectTransform>,
        correct_centre: impl Into<Point3>,
    ) -> Self {
        let transform = transform.into().with_correction(correct_centre);

        Self::new_uncorrected(objects, transform)
    }

    /// See [super::simple::SimpleObject::new_uncorrected()]
    ///
    /// # Panics
    /// The given iterator of `objects` should only contain *bounded* objects (i.e. [Object::aabb()] returns [`Some(_)`]).
    /// The exact behaviour is not specified, but will most likely result in a panic during building/accessing the tree
    pub fn new_uncorrected(objects: impl IntoIterator<Item = Obj>, transform: impl Into<ObjectTransform>) -> Self {
        let transform = transform.into();
        let inner = GenericBvh::new(objects);
        let aabb = inner.root_id().map(|root| match inner.arena()[root].get() {
            GenericBvhNode::Nested(aabb) => *aabb,
            GenericBvhNode::Object(o) => *o.expect_aabb(),
        });

        Self { inner, transform, aabb }
    }
}

impl<Obj: Object> BvhObject<Obj> {
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
    ) -> Option<FullIntersection<'o, Obj::Mat>> {
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
                if !obj.expect_aabb().hit(ray, bounds) {
                    None
                } else {
                    obj.full_intersect(ray, bounds, rng)
                }
            }
        };
    }
}

impl<Obj: Object> Object for BvhObject<Obj> {
    type Mesh = <Obj as Object>::Mesh;
    type Mat = <Obj as Object>::Mat;

    fn full_intersect<'o>(
        &'o self,
        orig_ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Obj::Mat>> {
        let trans_ray = self.transform.incoming_ray(orig_ray);
        // Pass everything on to our magical function
        let mut inner = Self::bvh_node_intersect(&trans_ray, bounds, self.inner.root_id()?, &self.inner.arena(), rng)?;
        inner.intersection = self.transform.outgoing_intersection(orig_ray, inner.intersection);
        Some(inner)
    }
}

impl<Obj: Object> HasAabb for BvhObject<Obj> {
    fn aabb(&self) -> Option<&Aabb> { self.aabb.as_ref() }
}
