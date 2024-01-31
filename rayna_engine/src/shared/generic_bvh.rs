//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-mesh intersection tests by narrowing the search space,
//! by skipping objects that obviously can't be intersected.

use getset::{CopyGetters, Getters};
use indextree::{Arena, NodeId};
use std::cmp::Ordering;

use crate::core::types::Number;
use smallvec::SmallVec;
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

        if objects.len() == 1 {
            return arena.new_node(GenericBvhNode::Object(objects.remove(0)));
        }

        // The number of objects under which we create leaf nodes, instead of
        // creating branches and splitting the objects
        const MAX_LEAF_NODES: usize = 4;
        if objects.len() <= MAX_LEAF_NODES {
            let aabb = Aabb::encompass_iter(objects.iter().map(HasAabb::expect_aabb));
            let node = arena.new_node(GenericBvhNode::Nested(aabb));
            objects.into_iter().for_each(|o| {
                node.append_value(GenericBvhNode::Object(o), arena);
            });
            return node;
        }

        {
            // This was originally based off Pete Shirley's SAH BVH algorithm
            // https://psgraphics.blogspot.com/2016/03/a-simple-sah-bvh-build.html
            // https://3.bp.blogspot.com/-PMG6dWk1i60/VuG9UHjsdlI/AAAAAAAACEo/BS1qJyut7LE/s1600/Screen%2BShot%2B2016-03-10%2Bat%2B11.25.08%2BAM.png

            // Find the longest axis to split along, and sort for that axis
            // TODO: Also attempt splitting more than twice
            let main_aabb = Aabb::encompass_iter(objects.iter().map(HasAabb::expect_aabb));

            let optimal_split_outer = Self::calculate_optimal_split(&mut objects);

            // Split the vector into the two halves. Annoyingly there is no nice API for boxed slices or vectors
            Self::sort_along_aabb_axis(optimal_split_outer.axis, &mut objects);

            let split = Self::split_objects::<1, 2>(objects, optimal_split_outer);

            // // // Repeat the split
            // let optimal_split_inner_1 = Self::calculate_optimal_split(&mut left_split);
            // let optimal_split_inner_2 = Self::calculate_optimal_split(&mut right_split);

            let main_node = arena.new_node(GenericBvhNode::Nested(main_aabb));
            let split_nodes =
                SmallVec::<[_; 4]>::from_iter(split.into_iter().map(|slice| Self::generate_nodes_sah(slice, arena)));
            split_nodes
                .into_iter()
                .for_each(|s_node| main_node.append(s_node, arena));
            return main_node;
        }
    }

    fn split_objects<const N_SPLIT: usize, const N_SPLIT_PLUS_ONE: usize>(
        mut objects: Vec<BNode>,
        split: BvhSplit<N_SPLIT>,
    ) -> [Vec<BNode>; N_SPLIT_PLUS_ONE] {
        // Unfortunately I can't use const assertions like `static_assertions::const_assert_eq()`
        // Since they create a `const _:()` and so use a generic from the outer item, which isn't allowed :(
        assert_eq!(N_SPLIT + 1, N_SPLIT_PLUS_ONE);

        Self::sort_along_aabb_axis(split.axis, &mut objects);

        let mut array: [Option<Vec<BNode>>; N_SPLIT_PLUS_ONE] = std::array::from_fn(|_| None);
        // We have to translate split positions so they are relative to the previous position
        let mut split_offset = 0;
        for i in 0..N_SPLIT {
            let split_count = split.pos[i] - split_offset;
            let remainder = objects.split_off(split_count);
            split_offset += split_count;
            array[i] = Some(objects);
            objects = remainder;
        }
        array[N_SPLIT] = Some(objects);

        array
            .try_map(std::convert::identity)
            .expect("something in my code fucked up")
    }

    /// Given a vec of objects, calculates the most optimal split position
    ///
    /// Requires mutable access to the vec, so that elements can be sorted along axes
    fn calculate_optimal_split(objects: &mut Vec<BNode>) -> BvhSplit<1> {
        assert!(objects.len() > 2, "cannot split with <=2 items");
        let n = objects.len();

        let mut bvh_splits = vec![];

        for sort_axis in SplitAxis::iter() {
            Self::sort_along_aabb_axis(sort_axis, objects);

            // Make sure we don't split with zero elements
            for split_pos in 1..n - 1 {
                let (split_l, split_r) = objects.split_at(split_pos);
                let splits = [split_l, split_r];

                let cost = splits
                    .iter()
                    .map(|s| {
                        let l = s.len() as Number;
                        let area = Aabb::encompass_iter(s.iter().map(HasAabb::expect_aabb)).area();
                        l * area
                    })
                    .sum();

                bvh_splits.push(BvhSplit {
                    axis: sort_axis,
                    pos: [split_pos],
                    cost,
                });
            }
        }

        bvh_splits
            .into_iter()
            .min_by(|a, b| Number::total_cmp(&a.cost, &b.cost))
            .unwrap()
    }
}

/// Enum for which axis we split along when doing SAH
#[derive(Copy, Clone, Debug, EnumIter)]
enum SplitAxis {
    X,
    Y,
    Z,
}

struct BvhSplit<const N: usize> {
    pub axis: SplitAxis,
    pub pos: [usize; N],
    pub cost: Number,
}
