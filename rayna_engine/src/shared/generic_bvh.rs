//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-mesh intersection tests by narrowing the search space,
//! by skipping objects that obviously can't be intersected.

use getset::{CopyGetters, Getters};
use indextree::{Arena, NodeId};
use std::cmp::Ordering;
use std::ops::Range;

use itertools::{zip_eq, Itertools};
use rayna_shared::def::types::Number;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::shared::aabb::{Aabb, HasAabb};

#[derive(Getters, CopyGetters, Clone, Debug)]
pub struct GenericBvh<Node: HasAabb> {
    /// The backing store containing all of our objects, as well as their hierarchy
    #[get = "pub"]
    arena: Arena<GenericBvhNode<Node>>,
    /// The node of the root object in the tree
    #[get_copy = "pub"]
    root_id: Option<NodeId>,
}

/// Enum for which axis we split along when doing SAH
#[derive(Copy, Clone, Debug, EnumIter)]
enum SplitAxis {
    X,
    Y,
    Z,
}

/// The type for each node in the BVH tree
///
/// Nodes are either a branch point [GenericBvhNode::Nested] (which has children),
/// or a leaf [GenericBvhNode::Object] (which is an object)
#[derive(Clone, Debug)]
pub enum GenericBvhNode<Node: HasAabb> {
    // Don't need to keep track of children since the tree does that for us
    Nested(Aabb),
    Object(Node),
}

impl<BNode: HasAabb> GenericBvh<BNode> {
    /// Creates a new [BvhObject] tree from the given slice of objects
    ///
    /// # Note
    /// The given slice of `objects` should only contain *bounded* objects (i.e. [HasAabb::aabb()] returns [`Some(_)`]).
    /// The exact behaviour is not specified, but will most likely result in a panic during building/accessing the tree
    pub fn new(objects: impl IntoIterator<Item = BNode>) -> Self {
        let objects = objects.into_iter().collect::<Vec<BNode>>();

        assert!(
            objects.iter().all(|o| o.aabb().is_some()),
            "objects should all be bounded"
        );

        let mut arena = Arena::with_capacity(objects.len());
        let root_id = if objects.is_empty() {
            None
        } else {
            Some(Self::generate_nodes_sah(objects, &mut arena))
        };

        // eprintln!("\n\n{:?}\n\n", root_id.debug_pretty_print(&tree));

        Self { arena, root_id }
    }

    /// Sorts the given slice of objects along the chosen `axis`
    /// This sort is *unstable* (see [sort_unstable_by](https://doc.rust-lang.org/std/primitive.slice.html#method.sort_unstable_by))
    fn sort_along_aabb_axis(axis: SplitAxis, objects: &mut [BNode]) {
        let sort_x = |a: &BNode, b: &BNode| -> Ordering {
            PartialOrd::partial_cmp(&a.expect_aabb().min().x, &b.expect_aabb().min().x)
                .expect("should be able to cmp AABB x-bounds: should not be nan")
        };
        let sort_y = |a: &BNode, b: &BNode| -> Ordering {
            PartialOrd::partial_cmp(&a.expect_aabb().min().y, &b.expect_aabb().min().y)
                .expect("should be able to cmp AABB y-bounds: should not be nan")
        };
        let sort_z = |a: &BNode, b: &BNode| -> Ordering {
            PartialOrd::partial_cmp(&a.expect_aabb().min().z, &b.expect_aabb().min().z)
                .expect("should be able to cmp AABB z-bounds: should not be nan")
        };

        match axis {
            SplitAxis::X => objects.sort_unstable_by(sort_x),
            SplitAxis::Y => objects.sort_unstable_by(sort_y),
            SplitAxis::Z => objects.sort_unstable_by(sort_z),
        }
    }

    /// Recursively processes the slice of `objects`, processing recursively until
    /// the objects are exhausted and the tree is created
    ///
    /// # **Surface-Area Heuristics** (SAH)
    /// This method uses SAH to optimise the choice of split axis, as well as split position.
    /// It does this by choosing the longest axis, and splitting at the point where the overall surface areas are optimal
    ///
    /// # Panics
    /// The slice of `objects` passed in must be non-empty.
    fn generate_nodes_sah(mut objects: Vec<BNode>, arena: &mut Arena<GenericBvhNode<BNode>>) -> NodeId {
        if 0 == objects.len() {
            panic!("internal invariant fail: must pass in a non-empty slice for objects")
        }

        // I really hate this code, but there's not much I can do about it
        // Can't match on a `Vec<O>`, can't move out of a `Box<[O]>` (even with `#![feature(box_patterns)]`)
        // Can't use `if let Ok(..)` since it moves the Vec

        objects = match <[BNode; 1]>::try_from(objects) {
            Ok([obj]) => {
                return arena.new_node(GenericBvhNode::Object(obj));
            }
            Err(v) => v,
        };

        objects = match <[BNode; 2]>::try_from(objects) {
            Ok([a, b]) => {
                let aabb = Aabb::encompass(a.expect_aabb(), b.expect_aabb());

                let node = arena.new_node(GenericBvhNode::Nested(aabb));
                node.append_value(GenericBvhNode::Object(a), arena);
                node.append_value(GenericBvhNode::Object(b), arena);
                return node;
            }
            Err(v) => v,
        };

        {
            // This is a port of [my C# port of] [Pete Shirley's code]
            // https://psgraphics.blogspot.com/2016/03/a-simple-sah-bvh-build.html
            // https://3.bp.blogspot.com/-PMG6dWk1i60/VuG9UHjsdlI/AAAAAAAACEo/BS1qJyut7LE/s1600/Screen%2BShot%2B2016-03-10%2Bat%2B11.25.08%2BAM.png

            // Normally with SAH, we would choose the longest axis to split along.
            // Instead, this calculates the cartesian product of all axes and positions, and chooses the best from them

            let n = objects.len();
            let aabbs = objects.iter().map(HasAabb::expect_aabb).copied().collect_vec();

            pub struct BvhSplit<const N: usize> {
                pub axis: SplitAxis,
                pub pos: [usize; N],
                pub cost: Number,
            }

            const NUM_SPLITS: usize = 2;
            let mut splits = Vec::new();
            for axis in SplitAxis::iter() {
                Self::sort_along_aabb_axis(axis, &mut objects);

                // Array of [0..n; NUM_SPLITS]
                let split_ranges = [0; NUM_SPLITS].map(|_| 0..n);
                // Calculate the areas of the left/right AABBs, for each given split position
                for split_pos in split_ranges.iter().combinations(NUM_SPLITS) {
                    let aabb_l = Aabb::encompass_iter(&aabbs[..pos]);
                    let aabb_r = Aabb::encompass_iter(&aabbs[pos + 1..]);

                    let area_l = aabb_l.area();
                    let area_r = aabb_l.area();

                    let p_l = pos as Number * area_l;
                    let p_r = (n - pos - 1) as Number * area_r;

                    let cost = (area_l * p_l) + (area_r * p_r);
                    splits.push(BvhSplit { pos, cost, axis });
                }

                // Calculate costs for positions
                {}
            }

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

            // Split the vector into the two halves. Annoyingly there is no nice API for boxed slices or vectors
            let (left_split, right_split) = {
                let mut l = Vec::from(objects);
                let r = l.split_off(split_index + 1);
                (l, r)
            };

            let left_node = Self::generate_nodes_sah(left_split, arena);
            let right_node = Self::generate_nodes_sah(right_split, arena);

            let node = arena.new_node(GenericBvhNode::Nested(main_aabb));
            node.append(left_node, arena);
            node.append(right_node, arena);
            return node;
        }

        /*

               {
            // This is a port of [my C# port of] [Pete Shirley's code]
            // https://psgraphics.blogspot.com/2016/03/a-simple-sah-bvh-build.html
            // https://3.bp.blogspot.com/-PMG6dWk1i60/VuG9UHjsdlI/AAAAAAAACEo/BS1qJyut7LE/s1600/Screen%2BShot%2B2016-03-10%2Bat%2B11.25.08%2BAM.png

            let n = objects.len();
            let aabbs = objects.iter().map(HasAabb::expect_aabb).copied().collect_vec();
            let main_aabb = Aabb::encompass_iter(&aabbs);

            // Find the longest axis to split along, and sort for that axis
            // TODO: maybe choose the axis that gives the smallest overlap between the left & right splits?
            //  This means why try `product_of(all 3 axes, all split positions)` and find the optimal by `left.len()^2 + right.len()^2`

            // Sort along longest axis
            {
                let max_side = match main_aabb.size().into_iter().position_max_by(Number::total_cmp) {
                    Some(0) => SplitAxis::X,
                    Some(1) => SplitAxis::Y,
                    Some(2) => SplitAxis::Z,
                    None => unreachable!("Vector3::into_iter() cannot be empty iterator"),
                    Some(x) => unreachable!("invalid axis {}", x),
                };
                Self::sort_along_aabb_axis(max_side, &mut objects);
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
                for (area, obj_aabb) in zip_eq(right_areas.iter_mut().rev(), aabbs.iter().rev()) {
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

            // Split the vector into the two halves. Annoyingly there is no nice API for boxed slices or vectors
            let (left_split, right_split) = {
                let mut l = Vec::from(objects);
                let r = l.split_off(split_index + 1);
                (l, r)
            };

            let left_node = Self::generate_nodes_sah(left_split, arena);
            let right_node = Self::generate_nodes_sah(right_split, arena);

            let node = arena.new_node(GenericBvhNode::Nested(main_aabb));
            node.append(left_node, arena);
            node.append(right_node, arena);
            return node;
        }


         */
    }
}
