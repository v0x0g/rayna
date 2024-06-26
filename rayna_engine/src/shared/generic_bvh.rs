//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-mesh intersection tests by narrowing the search space,
//! by skipping objects that obviously can't be intersected.

use derivative::Derivative;
use getset::{CopyGetters, Getters};
use indextree::{Arena, NodeId};
use itertools::Itertools;
use std::cmp::Ordering;

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
    /// Creates a new [`Self`] tree from the given slice of objects
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

        // root_id.map(|root_id| eprintln!("\n\n{:?}\n\n", root_id.debug_pretty_print(&arena)));

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

            // NOTE: Ideally, I would be able to use `N_SPLIT=3` for this, to partition into four separate chunks along the axis
            //  However, due to the time complexity (I think either `O(N_S!)` or `O(e^N_S)`, it's impossibly slow
            //  (i.e. `N_SPLIT=4` for 32K objects takes more than several hours (I gave up after four)
            //  Instead, just run the split twice, which should give four child nodes

            let main_node = arena.new_node(GenericBvhNode::Nested(main_aabb));

            let optimal_split_outer = Self::calculate_optimal_split::<1, 2>(&mut objects)
                .expect("outer split calculation should always succeed");
            let outer_split_objects = Self::split_objects(objects, optimal_split_outer);
            for mut outer_split in outer_split_objects {
                // Try split again
                if let Some(sub_split) = Self::calculate_optimal_split::<1, 2>(&mut outer_split) {
                    let sub_split_objects = Self::split_objects(outer_split, sub_split);
                    for chunk in sub_split_objects {
                        main_node.append(Self::generate_nodes_sah(chunk, arena), arena);
                    }
                } else {
                    main_node.append(Self::generate_nodes_sah(outer_split, arena), arena);
                }
            }

            // const N_SPLIT: usize = 1;
            // let optimal_split_outer = Self::calculate_optimal_split::<N_SPLIT, { N_SPLIT + 1 }>(&mut objects);
            // let split_objects = Self::split_objects(objects, optimal_split_outer);

            return main_node;
        }
    }

    fn split_objects<const N_SPLIT_PLUS_ONE: usize>(
        mut objects: Vec<BNode>,
        optimal_split: BvhSplit<N_SPLIT_PLUS_ONE>,
    ) -> [Vec<BNode>; N_SPLIT_PLUS_ONE] {
        Self::sort_along_aabb_axis(optimal_split.axis, &mut objects);

        optimal_split.split_lengths.map(|take| {
            let mut remainder = objects.split_off(take);
            // We want to keep the remainder (the right segment) for next iteration
            std::mem::swap(&mut objects, &mut remainder);
            remainder
        })
    }

    /// Given a vec of objects, calculates the most optimal split position(s)
    ///
    /// Requires mutable access to the vec, so that elements can be sorted along axes
    fn calculate_optimal_split<const N_SPLIT: usize, const N_SPLIT_PLUS_ONE: usize>(
        objects: &mut Vec<BNode>,
    ) -> Option<BvhSplit<N_SPLIT_PLUS_ONE>> {
        // Unfortunately I can't use const assertions like `static_assertions::const_assert_eq()`
        // Since they create a `const _:()` and so use a generic from the outer item, which isn't allowed :(
        assert_eq!(N_SPLIT + 1, N_SPLIT_PLUS_ONE);
        if objects.len() <= 2 * N_SPLIT {
            return None;
        }

        // When we have a large number of objects, we can potentially be checking millions or more split positions
        // So we can batch objects slightly, so that we don't check *all* combinations, hopefully speeding things up a bit
        // It will make the resulting BVH tree a bit less granular, but since the objects are sorted it shouldn't
        // affect performance/BVH quality much

        /// Higher values are more aggressive at skipping, and are faster to build. Range `0..1`
        const SKIP_AGGRESSION: Number = 0.0;
        static_assertions::const_assert!(0.0 <= SKIP_AGGRESSION && SKIP_AGGRESSION < 1.0);
        let batch_skip = (objects.len() as Number).powf(SKIP_AGGRESSION) - 1.0;
        let mut batch_counter = 0.0;

        let mut best_split = None;

        for sort_axis in SplitAxis::iter() {
            Self::sort_along_aabb_axis(sort_axis, objects);

            // We want to iterate over all combinations of split positions
            // So we would have [1, 2], [1,3], ... [n-1, n-1], for any length K
            // I would use `itertools`, but it allocates a new vector every time and is SLOW

            let combinator_values = (1..objects.len() - 1).collect_vec();
            let combinator_len = combinator_values.len();
            let mut combinator_indices: [usize; N_SPLIT] = std::array::from_fn(std::convert::identity);
            let mut combinator_first = true;

            'combinations_loop: loop {
                // region COMBINATORS CODE

                if combinator_first {
                    combinator_first = false;
                } else {
                    // Scan from the end, looking for an index to increment
                    let mut i: usize = N_SPLIT - 1;

                    while combinator_indices[i] == i + combinator_len - N_SPLIT {
                        if i > 0 {
                            i -= 1;
                        } else {
                            // Reached the last combination
                            break 'combinations_loop;
                        }
                    }

                    // Increment index, and reset the ones to its right
                    combinator_indices[i] += 1;
                    for j in i + 1..combinator_indices.len() {
                        combinator_indices[j] = combinator_indices[j - 1] + 1;
                    }
                }

                // endregion COMBINATORS CODE

                // Skip values occasionally so we can speed things up a bit
                if batch_counter > 0.0 {
                    batch_counter -= 1.0;
                    continue;
                } else {
                    batch_counter += batch_skip;
                }

                // Find absolute split positions from indices
                let abs_split_positions = combinator_indices.map(|i| combinator_values[i]);

                let split_lengths: [usize; N_SPLIT_PLUS_ONE] = {
                    // We have to translate split positions so they are relative to the previous position
                    let mut split_offset = 0;
                    std::array::from_fn(|slice_idx| {
                        // If we are on the last split segment, we want to take all the elements
                        // so essentially split at `objects.len()`
                        let split_count = abs_split_positions.get(slice_idx).unwrap_or(&objects.len()) - split_offset;
                        split_offset += split_count;
                        split_count
                    })
                };

                let splits: [&[BNode]; N_SPLIT_PLUS_ONE] = {
                    let mut array = objects.as_slice();
                    split_lengths.map(|take| {
                        let (chunk, rest) = array.split_at(take);
                        array = rest;
                        chunk
                    })
                };

                let cost = splits
                    .iter()
                    .map(|&s| {
                        let l = s.len() as Number;
                        let area = Aabb::encompass_iter(s.iter().map(HasAabb::expect_aabb)).area();
                        l * area
                    })
                    .sum();

                let curr_split = BvhSplit {
                    axis: sort_axis,
                    split_lengths,
                    cost,
                };

                best_split = Some(match best_split {
                    Some(best) => std::cmp::min_by(best, curr_split, |a, b| Number::total_cmp(&a.cost, &b.cost)),
                    None => curr_split,
                });
            }
        }

        Some(best_split.expect("best split was not set: did no iterations"))
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
struct BvhSplit<const SPLIT_PLUS_ONE: usize> {
    /// What axis to split along
    pub axis: SplitAxis,
    /// The lengths of each slice to make when doing this split. None should be zero
    pub split_lengths: [usize; SPLIT_PLUS_ONE],
    /// The cost of making this given split. Probably a function of SAH
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub cost: Number,
}
