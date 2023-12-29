//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-object intersection tests by narrowing the search space,
//! by skipping objects that obviously can't be intersected.

use std::cmp::Ordering;

use indextree::{Arena, NodeId};
use itertools::{zip_eq, Itertools};
use rand::prelude::SliceRandom;
use rand::thread_rng;

use rayna_shared::def::types::Number;

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

        Self::generate_nodes_sah(objects, &mut tree, root_id);

        // Ensure there are no temp nodes left in the tree
        assert!(
            tree.iter()
                .filter(|n| !n.is_removed())
                .all(|n| !matches!(n.get(), BvhNode::TempNode)),
            "should not be any temp nodes in tree"
        );
        eprintln!("\n\n{:?}\n\n", root_id.debug_pretty_print(&tree));

        Self { tree }
    }

    fn aabb_axis_comparator(axis: SplitAxis) -> fn(&ObjectType, &ObjectType) -> Ordering {
        match axis {
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
        }
    }

    /// Recursively processes the slice of `objects`, adding them to the `node` recursively until
    /// the tree is exhausted
    fn generate_nodes(
        objects: &[ObjectType],
        axis: SplitAxis,
        tree: &mut Arena<BvhNode>,
        node_id: NodeId,
    ) {
        let comparator = Self::aabb_axis_comparator(axis);

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
                Self::generate_nodes(left, *split_axis, tree, left_node);

                let right_aabb = Aabb::encompass_iter(right.iter().map(Object::bounding_box));
                let right_node = tree.new_node(BvhNode::Aabb(right_aabb));
                node_id.append(right_node.clone(), tree);
                Self::generate_nodes(right, *split_axis, tree, right_node);

                BvhNode::Aabb(Aabb::encompass(left_aabb, right_aabb))
            }
        };

        // Update the current node
        *tree[node_id].get_mut() = bvh_data;
    }

    /// Recursively processes the slice of `objects`, adding them to the `node` recursively until
    /// the tree is exhausted
    ///
    /// # **Surface-Area Heuristics** (SAH)
    /// This method is very similar to [Self::generate_nodes], however it uses SAH to optimise the choice
    /// of split axis, as well as split position. It does this by choosing the longest axis,
    fn generate_nodes_sah(objects: &[ObjectType], tree: &mut Arena<BvhNode>, node_id: NodeId) {
        let bvh_data = match objects {
            [obj] => BvhNode::Object(obj.clone()),
            [a, b] => {
                node_id.append(tree.new_node(BvhNode::Object(a.clone())), tree);
                node_id.append(tree.new_node(BvhNode::Object(b.clone())), tree);
                BvhNode::Aabb(Aabb::encompass(a.bounding_box(), b.bounding_box()))
            }
            objects => {
                // This is a port of [my C# port of] [Pete Shirley's code]
                // https://psgraphics.blogspot.com/2016/03/a-simple-sah-bvh-build.html
                // https://3.bp.blogspot.com/-PMG6dWk1i60/VuG9UHjsdlI/AAAAAAAACEo/BS1qJyut7LE/s1600/Screen%2BShot%2B2016-03-10%2Bat%2B11.25.08%2BAM.png

                let n = objects.len();
                let mut objects = Vec::from(objects);
                let aabbs = objects.iter().map(|o| *o.bounding_box()).collect_vec();
                let main_aabb = Aabb::encompass_iter(&aabbs);

                // Find the longest axis to split along, and sort for that axis
                // TODO: maybe choose the axis that gives the smallest overlap between the left & right splits?
                //  This means why try `product_of(all 3 axes, all split positions)` and find the optimal by `left.len()^2 + right.len()^2`

                {
                    let max_side = match main_aabb
                        .size()
                        .into_iter()
                        .position_max_by(Number::total_cmp)
                    {
                        Some(0) => SplitAxis::X,
                        Some(1) => SplitAxis::Y,
                        Some(2) => SplitAxis::Z,
                        None => unreachable!("Vector3::into_iter() cannot be empty iterator"),
                        Some(x) => unreachable!("invalid axis {}", x),
                    };
                    let comparator = Self::aabb_axis_comparator(max_side);
                    objects.sort_unstable_by(comparator);
                }

                // Calculate the areas of the left/right AABBs, for each given split position
                let (left_areas, right_areas) = {
                    let mut left_areas = Vec::new();
                    left_areas.resize(n, 0.);
                    let mut right_areas = Vec::new();
                    right_areas.resize(n, 0.);
                    //Calculate the area from the left towards right
                    let mut left_aabb = Aabb::default();
                    for (area, obj_aabb) in zip_eq(left_areas.iter_mut(), aabbs.iter()) {
                        left_aabb = Aabb::encompass(&left_aabb, obj_aabb);
                        *area = left_aabb.area();
                    }

                    //Calculate the area from the right towards the left
                    let mut right_aabb = Aabb::default();
                    for (area, obj_aabb) in zip_eq(right_areas.iter_mut().rev(), aabbs.iter().rev())
                    {
                        right_aabb = Aabb::encompass(&right_aabb, obj_aabb);
                        *area = right_aabb.area();
                    }
                    (left_areas, right_areas)
                };

                // Find the most optimal split index, using the areas calculated above
                let split_index = {
                    // NOTE: If doing in a for loop this would be `i: 0..n-1`, and `l=left[i], r=right[i+1]`
                    // This way we have non-overlapping left & right areas
                    let left_trimmed = left_areas.split_last().expect("left_area is empty").1;
                    let right_trimmed = right_areas.split_first().expect("right_area is empty").1;
                    let min_sa_idx = zip_eq(left_trimmed, right_trimmed)
                        .enumerate()
                        // calculate SA
                        .map(|(i, (&l, &r))| (i as Number * l) + ((n - i - 1) as Number * r))
                        .position_min_by(Number::total_cmp)
                        .expect("area iters have >1 elem");
                    min_sa_idx
                };

                let (left_split, right_split) = objects.split_at(split_index);

                let left_aabb = Aabb::encompass_iter(left_split.iter().map(Object::bounding_box));
                let left_node = tree.new_node(BvhNode::Aabb(left_aabb));
                node_id.append(left_node.clone(), tree);
                Self::generate_nodes_sah(left_split, tree, left_node);

                let right_aabb = Aabb::encompass_iter(right_split.iter().map(Object::bounding_box));
                let right_node = tree.new_node(BvhNode::Aabb(right_aabb));
                node_id.append(right_node.clone(), tree);
                Self::generate_nodes_sah(right_split, tree, right_node);

                BvhNode::Aabb(Aabb::encompass(left_aabb, right_aabb))
            }
        };

        // Update the current node
        *tree[node_id].get_mut() = bvh_data;
    }
}

// TODO: impl<O: Object> Object for NodeRef<O>
