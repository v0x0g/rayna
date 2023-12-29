//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-object intersection tests by narrowing the search space,
//! by skipping objects that obviously can't be intersected.

use std::cmp::Ordering;

use indextree::{Arena, NodeId};
use rand::prelude::SliceRandom;
use rand::thread_rng;

use crate::accel::aabb::Aabb;
use crate::object::{Object, ObjectType};

#[derive(Clone, Debug)]
pub struct Bvh {
    tree: Arena<BvhNode>,
}

#[derive(Copy, Clone, Debug)]
enum SplitAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Debug)]
enum BvhNode {
    Aabb(Aabb),
    Object(ObjectType),
    /// Marker for a temporary node, should not be present in the final tree
    TempNode,
}

impl Bvh {
    pub fn new(objects: &[ObjectType]) -> Self {
        let mut tree = Arena::with_capacity(objects.len());
        let root_id = tree.new_node(BvhNode::TempNode);
        Self::new_node_recursive(objects, SplitAxis::X, &mut tree, root_id);

        eprintln!("\n\n{tree:?}\n\n");

        Self { tree }
    }

    /// Recursively processes the slice of `objects`, adding them to the `node` recursively until
    /// the tree is exhausted
    ///
    /// # Note:
    /// Since the tree structure doesn't have a concept of leaves/nodes being different, we use
    /// [None] to be nodes/branches, and [Some] to be leaves
    fn new_node_recursive(
        objects: &[ObjectType],
        axis: SplitAxis,
        tree: &mut Arena<BvhNode>,
        node_id: NodeId,
    ) {
        let comparator: fn(&ObjectType, &ObjectType) -> Ordering = match axis {
            SplitAxis::X => |a, b| {
                PartialOrd::partial_cmp(&a.bounding_box().min().x, &b.bounding_box().min().x)
                    .expect("should be able to cmp AABB bounds: should not be nan")
            },
            SplitAxis::Y => |a, b| {
                PartialOrd::partial_cmp(&a.bounding_box().min().y, &b.bounding_box().min().y)
                    .expect("should be able to cmp AABB bounds: should not be nan")
            },
            SplitAxis::Z => |a, b| {
                PartialOrd::partial_cmp(&a.bounding_box().min().z, &b.bounding_box().min().z)
                    .expect("should be able to cmp AABB bounds: should not be nan")
            },
        };

        let bvh_data = match objects {
            [obj] => BvhNode::Object(obj.clone()),
            [a, b] => {
                // "Smaller" node goes on the left (first)
                // TODO: Is sorting for a bi-node necessary?
                if comparator(&a, &b) == Ordering::Less {
                    node_id.append(tree.new_node(BvhNode::Object(a.clone())), tree);
                    node_id.append(tree.new_node(BvhNode::Object(b.clone())), tree);
                } else {
                    node_id.append(tree.new_node(BvhNode::Object(b.clone())), tree);
                    node_id.append(tree.new_node(BvhNode::Object(a.clone())), tree);
                };
                BvhNode::Aabb(Aabb::encompass(a.bounding_box(), b.bounding_box()))
            }
            objects => {
                let mut objects = Vec::from(objects);
                objects.sort_by(comparator);

                let rng = &mut thread_rng();
                let split_axis = [SplitAxis::X, SplitAxis::Y, SplitAxis::Z]
                    .choose(rng)
                    .unwrap();

                // split in half and repeat tree
                let (left, right) = objects.split_at(objects.len() / 2);

                let left_aabb = Aabb::encompass_iter(left.iter().map(Object::bounding_box));
                let left_node = tree.new_node(BvhNode::Aabb(left_aabb));
                node_id.append(left_node.clone(), tree);
                Self::new_node_recursive(left, *split_axis, tree, left_node);

                let right_aabb = Aabb::encompass_iter(right.iter().map(Object::bounding_box));
                let right_node = tree.new_node(BvhNode::Aabb(right_aabb));
                node_id.append(right_node.clone(), tree);
                Self::new_node_recursive(right, *split_axis, tree, right_node);

                BvhNode::Aabb(Aabb::encompass(left_aabb, right_aabb))
            }
        };

        // Update the current node
        *tree[node_id].get_mut() = bvh_data;
    }

    // pub fn new_sah(objects: &[ObjectType]) -> Self {
    //     // let mut tree = TreeBuilder::new().with_capacity(objects.len()).build();
    //     // tree.root_id();
    //     todo!()
    // }
}

// TODO: impl<O: Object> Object for NodeRef<O>
