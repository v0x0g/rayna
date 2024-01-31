//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-mesh intersection tests by narrowing the search space,
//! by skipping objects that obviously can't be intersected.

use derivative::Derivative;
use getset::{CopyGetters, Getters};
use indextree::{Arena, NodeId};
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::HashSet;

use crate::core::types::Number;
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

        if objects.len() == 1 {
            return arena.new_node(GenericBvhNode::Object(objects.remove(0)));
        }

        // The number of objects under which we create leaf nodes, instead of
        // creating branches and splitting the objects
        const MAX_LEAF_NODES: usize = 8;
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
            let main_aabb = Aabb::encompass_iter(objects.iter().map(HasAabb::expect_aabb));

            let optimal_split_outer = Self::calculate_optimal_split(&mut objects);

            let split_objects = Self::split_objects::<1, 2>(objects, optimal_split_outer);
            let main_node = arena.new_node(GenericBvhNode::Nested(main_aabb));

            for mut chunk in split_objects {
                // Attempt to split *again* if there are enough objects
                // So instead of having at most two slices at each depth,
                // we attempt to split *four* times each time
                // So each node on the tree has hopefully four children
                if chunk.len() > 2 {
                    let optimal = Self::calculate_optimal_split::<1>(&mut chunk);
                    let sub = Self::split_objects::<1, 2>(chunk, optimal);
                    let nodes = sub.map(|slice| Self::generate_nodes_sah(slice, arena));
                    nodes.into_iter().for_each(|s_node| main_node.append(s_node, arena));
                } else {
                    main_node.append(Self::generate_nodes_sah(chunk, arena), arena);
                }
            }

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
            .expect("all elements of the array should have been set")
    }

    /// Given a vec of objects, calculates the most optimal split position
    ///
    /// Requires mutable access to the vec, so that elements can be sorted along axes
    fn calculate_optimal_split<const N_S: usize>(objects: &mut Vec<BNode>) -> BvhSplit<N_S> {
        assert!(
            objects.len() > 2 * N_S,
            "cannot split with <=2 items per split (have {})",
            objects.len()
        );
        let n = objects.len();

        let mut bvh_splits = HashSet::new();

        for sort_axis in SplitAxis::iter() {
            Self::sort_along_aabb_axis(sort_axis, objects);

            // Make sure we don't split with zero elements
            let valid_split_indices = 1..n - 1;
            for absolute_split_positions in valid_split_indices
                .combinations(N_S)
                .map(|i| <[usize; N_S]>::try_from(i).unwrap())
            {
                let split_positions: [usize; N_S] = {
                    let mut split_offset = 0;
                    std::array::from_fn(|i| {
                        // How many elements to split off. Remove offset since we want relative pos from absolute pos
                        let split_count = absolute_split_positions[i] - split_offset;
                        split_offset += split_count;
                        split_count
                    })
                };

                let splits: [&[BNode]; N_S] = {
                    let mut array = objects.as_slice();
                    split_positions.map(|take| {
                        let (chunk, rest) = array.split_at(take);
                        array = rest;
                        chunk
                    })
                };

                let cost = splits
                    .iter()
                    .map(|s| {
                        let l = s.len() as Number;
                        let area = Aabb::encompass_iter(s.iter().map(HasAabb::expect_aabb)).area();
                        l * area
                    })
                    .sum();

                bvh_splits.insert(BvhSplit {
                    axis: sort_axis,
                    pos: split_positions,
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
#[derive(Copy, Clone, Debug, EnumIter, Hash, Ord, PartialOrd, Eq, PartialEq)]
enum SplitAxis {
    X = 0,
    Y = 1,
    Z = 2,
}

#[derive(Copy, Clone, Debug, Derivative)]
#[derivative(Hash, Eq, PartialEq)]
struct BvhSplit<const N: usize> {
    pub axis: SplitAxis,
    pub pos: [usize; N],
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub cost: Number,
}
