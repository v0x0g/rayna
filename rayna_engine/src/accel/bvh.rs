//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-object intersection tests by narrowing the search space,
//! by skipping objects that obviously can't be intersected.

use std::cmp::Ordering;

use rand::prelude::SliceRandom;
use rand::thread_rng;
use slab_tree::{NodeMut, TreeBuilder};

use crate::object::{Object, ObjectType};

#[derive(Clone, Debug)]
pub struct Bvh {}

#[derive(Copy, Clone, Debug)]
enum SplitAxis {
    X,
    Y,
    Z,
}

impl Bvh {
    pub fn new(objects: &[ObjectType]) -> Self {
        let mut tree = TreeBuilder::new().with_capacity(objects.len()).build();
        tree.set_root(None);
        Self::new_node(
            objects,
            SplitAxis::X,
            tree.root_mut().expect("we just set the root"),
        );
        todo!()
    }

    ///
    /// # Note:
    /// Since the tree structure doesn't have a concept of leaves/nodes being different, we use
    /// [Option::None] to be nodes/branches, and [Option::Some] to be leaves
    fn new_node(objects: &[ObjectType], axis: SplitAxis, mut node: NodeMut<Option<ObjectType>>) {
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

        match objects {
            [obj] => {
                *node.data() = Some(obj.clone());
            }
            [a, b] => {
                // "Smaller" node goes on the left (first)
                if comparator(&a, &b) == Ordering::Less {
                    node.append(Some(a.clone()));
                    node.append(Some(b.clone()));
                } else {
                    node.append(Some(b.clone()));
                    node.append(Some(a.clone()));
                };
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

                let left_node = node.append(None);
                Self::new_node(left, *split_axis, left_node);

                let right_node = node.append(None);
                Self::new_node(right, *split_axis, right_node);
            }
        };
    }

    pub fn new_sah(objects: &[ObjectType]) -> Self {
        // let mut tree = TreeBuilder::new().with_capacity(objects.len()).build();
        // tree.root_id();
        todo!()
    }
}

// TODO: impl<O: Object> Object for NodeRef<O>
