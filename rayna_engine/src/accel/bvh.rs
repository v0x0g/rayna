//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-object intersection tests by narrowing the search space,
//! by skipping objects that obviously can't be intersected.

use indextree::{Arena, NodeId};
use std::cmp::Ordering;

use itertools::{zip_eq, Itertools};
use smallvec::SmallVec;

use rayna_shared::def::types::{Number, Point3};

use crate::accel::aabb::Aabb;
use crate::object::{Object, ObjectInstance, ObjectProperties};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;

#[derive(Clone, Debug)]
pub struct Bvh {
    arena: Arena<BvhNode>,
    root_id: Option<NodeId>,
}

#[derive(Copy, Clone, Debug)]
enum SplitAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Debug)]
enum BvhNode {
    // Don't need to keep track of children since the tree does that for us
    Nested(Aabb),
    Object(ObjectInstance),
}

/// Helper function to unwrap an AABB with a panic message
fn expect_aabb(o: &ObjectInstance) -> &Aabb { o.aabb().expect("aabb required as invariant of `Bvh`") }

impl Bvh {
    /// Creates a new [Bvh] tree from the given slice of objects
    ///
    /// # Note
    /// The given slice of `objects` should only contain *bounded* objects (i.e. [Object::aabb()] returns [`Some(_)`]).
    /// The exact behaviour is not specified, but will most likely result in a panic during building/accessing the tree
    pub fn new(objects: &[ObjectInstance]) -> Self {
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
    fn sort_along_aabb_axis(axis: SplitAxis, objects: &mut [ObjectInstance]) {
        fn sort_x(a: &ObjectInstance, b: &ObjectInstance) -> Ordering {
            PartialOrd::partial_cmp(&expect_aabb(a).min().x, &expect_aabb(b).min().x)
                .expect("should be able to cmp AABB x-bounds: should not be nan")
        }

        fn sort_y(a: &ObjectInstance, b: &ObjectInstance) -> Ordering {
            PartialOrd::partial_cmp(&expect_aabb(a).min().y, &expect_aabb(b).min().y)
                .expect("should be able to cmp AABB y-bounds: should not be nan")
        }
        fn sort_z(a: &ObjectInstance, b: &ObjectInstance) -> Ordering {
            PartialOrd::partial_cmp(&expect_aabb(a).min().z, &expect_aabb(b).min().z)
                .expect("should be able to cmp AABB z-bounds: should not be nan")
        }

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
    fn generate_nodes_sah(objects: &[ObjectInstance], arena: &mut Arena<BvhNode>) -> NodeId {
        return match objects {
            [] => panic!("Must pass in a non-empty slice for objects"),
            [obj] => arena.new_node(BvhNode::Object(obj.clone())),
            [a, b] => {
                let aabb = Aabb::encompass(expect_aabb(a), expect_aabb(b));

                let node = arena.new_node(BvhNode::Nested(aabb));
                node.append_value(BvhNode::Object(a.clone()), arena);
                node.append_value(BvhNode::Object(b.clone()), arena);
                node
            }
            objects => {
                // This is a port of [my C# port of] [Pete Shirley's code]
                // https://psgraphics.blogspot.com/2016/03/a-simple-sah-bvh-build.html
                // https://3.bp.blogspot.com/-PMG6dWk1i60/VuG9UHjsdlI/AAAAAAAACEo/BS1qJyut7LE/s1600/Screen%2BShot%2B2016-03-10%2Bat%2B11.25.08%2BAM.png

                let n = objects.len();
                let mut objects = Vec::from(objects);
                let aabbs = objects.iter().map(|o| *expect_aabb(o)).collect_vec();
                let main_aabb = Aabb::encompass_iter(&aabbs);

                // Find the longest axis to split along, and sort for that axis
                // TODO: maybe choose the axis that gives the smallest overlap between the left & right splits?
                //  This means why try `product_of(all 3 axes, all split positions)` and find the optimal by `left.len()^2 + right.len()^2`

                // TODO: Sometimes this seems to generate a node with a single object.
                //  It creates an (AABB->Object) node, which does double ray-aabb intersects (this is slow)

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

                let (left_split, right_split) = objects.split_at(split_index + 1);

                let left_node = Self::generate_nodes_sah(left_split, arena);
                let right_node = Self::generate_nodes_sah(right_split, arena);

                let node = arena.new_node(BvhNode::Nested(main_aabb));
                node.append(left_node, arena);
                node.append(right_node, arena);
                node
            }
        };
    }
}

/// Given a [NodeId] on the [Arena] tree, calculates the nearest intersection for the given `ray` and `bounds`
///
/// If the node is a [BvhNode::Object], it passes on the check to the object.
/// Otherwise, if it's a [BvhNode::Aabb], it:
///     - Tries to bail early if the [Aabb] is missed
///     - Collects all the child nodes
///     - Intersects on all those children (by calling itself recursively)
///     - Returns the closest intersection of the above
fn bvh_node_intersect<'o>(
    ray: &Ray,
    bounds: &Bounds<Number>,
    node: NodeId,
    arena: &'o Arena<BvhNode>,
) -> Option<FullIntersection<'o>> {
    return match arena.get(node).expect("node should exist in arena").get() {
        // An aabb will need to delegate to child nodes if not missed
        BvhNode::Nested(aabb) => {
            if !aabb.hit(ray, bounds) {
                return None;
            }

            // TODO: Rework this to use the new Bounds::bitor API to shrink the next child's search range
            //  So keep track of the bounds, and each iteration shrink with `bounds = bounds | ..intersection.dist`
            //  And if an intersect was found in that shrunk range then we know that

            node.children(arena)
                .filter_map(|child| bvh_node_intersect(ray, bounds, child, arena))
                .min()
        }
        // Objects can be delegated directly
        BvhNode::Object(obj) => {
            if !obj.aabb().expect("aabb missing").hit(ray, bounds) {
                None
            } else {
                obj.intersect(ray, bounds)
            }
        }
    };
}

/// Given a [NodeId] on the [Arena] tree, calculates the ALL intersection for the given `ray`
///
/// If the node is a [BvhNode::Object], it passes on the check to the object.
/// Otherwise, if it's a [BvhNode::Aabb], it:
///  - Tries to bail early if the [Aabb] is missed
///  - Collects all the child nodes
///  - Intersects on all those children (by calling itself recursively)
fn bvh_node_intersect_all<'o>(
    ray: &Ray,
    node: NodeId,
    arena: &'o Arena<BvhNode>,
    output: &mut SmallVec<[FullIntersection<'o>; 32]>,
) {
    match arena.get(node).expect("node should exist in arena").get() {
        // An aabb will need to delegate to child nodes if not missed
        BvhNode::Nested(aabb) => {
            if !aabb.hit(ray, &Bounds::FULL) {
                return;
            }

            node.children(arena)
                .for_each(|child| bvh_node_intersect_all(ray, child, arena, output));
        }
        // Objects can be delegated directly
        BvhNode::Object(obj) => {
            if !expect_aabb(obj).hit(ray, &Bounds::FULL) {
                return;
            }
            obj.intersect_all(ray, output)
        }
    }

    // // Possibly faster method, doesn't do any tree traversal at all, should be linear
    // arena
    //     .iter()
    //     .filter(|node| !node.is_removed())
    //     .filter_map(|node| match node.get() {BvhNode::Object(o) => Some(o), _ => None})
    //     .for_each(|obj| obj.intersect_all(ray, output))
}

impl Object for Bvh {
    fn intersect<'o>(&'o self, ray: &Ray, bounds: &Bounds<Number>) -> Option<FullIntersection<'o>> {
        // Pass everything on to our magical function
        bvh_node_intersect(ray, bounds, self.root_id?, &self.arena)
    }

    fn intersect_all<'o>(&'o self, ray: &Ray, output: &mut SmallVec<[FullIntersection<'o>; 32]>) {
        if let Some(root) = self.root_id {
            bvh_node_intersect_all(ray, root, &self.arena, output);
        }
    }
}

impl ObjectProperties for Bvh {
    fn aabb(&self) -> Option<&Aabb> {
        let root = self.root_id?;
        match self
            .arena
            .get(root)
            .expect(&format!("Arena should contain root node {root}"))
            .get()
        {
            BvhNode::Nested(aabb) => Some(aabb),
            BvhNode::Object(o) => Some(expect_aabb(o)),
        }
    }

    fn centre(&self) -> Point3 {
        unimplemented!("a Bvh tree doesn't have a centre: this method should never be called")
    }
}
