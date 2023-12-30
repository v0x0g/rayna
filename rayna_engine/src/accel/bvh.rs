//! Module containing **Bounding Volume Hierarchy** (BVH) structures
//!
//! These are used to accelerate ray-object intersection tests by narrowing the search space,
//! by skipping objects that obviously can't be intersected.

use crate::accel::aabb::Aabb;
use crate::object::{Object, ObjectType};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use itertools::{zip_eq, Itertools};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rayna_shared::def::types::Number;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct Bvh {
    root: BvhNode,
}

#[derive(Copy, Clone, Debug)]
enum SplitAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Debug)]
enum BvhNode {
    Nested {
        aabb: Aabb,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
    // TODO: Dual {
    //     aabb: Aabb,
    //     left: ObjectType,
    //     right: ObjectType,
    // },
    Object(ObjectType),
}

impl Bvh {
    pub fn new(objects: &[ObjectType]) -> Self {
        let root = *Self::generate_nodes(objects);

        // eprintln!("\n\n{:?}\n\n", root_id.debug_pretty_print(&tree));

        Self { root }
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
    fn generate_nodes(objects: &[ObjectType]) -> Box<BvhNode> {
        let bvh_data = match objects {
            [obj] => BvhNode::Object(obj.clone()),
            [a, b] => {
                let left = Box::new(BvhNode::Object(a.clone()));
                let right = Box::new(BvhNode::Object(b.clone()));
                let aabb = Aabb::encompass(a.bounding_box(), b.bounding_box());
                BvhNode::Nested { left, right, aabb }
            }
            objects => {
                let mut objects = Vec::from(objects);
                let rng = &mut thread_rng();
                let &split_axis = [SplitAxis::X, SplitAxis::Y, SplitAxis::Z]
                    .choose(rng)
                    .unwrap();
                let comparator = Self::aabb_axis_comparator(split_axis);
                objects.sort_by(comparator);

                // split in half and repeat tree
                let (left_split, right_split) = objects.split_at(objects.len() / 2);

                let left_aabb = Aabb::encompass_iter(left_split.iter().map(Object::bounding_box));
                let left_node = Self::generate_nodes(left_split);

                let right_aabb = Aabb::encompass_iter(right_split.iter().map(Object::bounding_box));
                let right_node = Self::generate_nodes(right_split);

                let aabb = Aabb::encompass(left_aabb, right_aabb);

                BvhNode::Nested {
                    left: left_node,
                    right: right_node,
                    aabb,
                }
            }
        };

        Box::new(bvh_data)
    }

    /// Recursively processes the slice of `objects`, processing recursively until
    /// the objects are exhausted and the tree is created
    ///
    /// # **Surface-Area Heuristics** (SAH)
    /// This method is very similar to [Self::generate_nodes], however it uses SAH to optimise the choice
    /// of split axis, as well as split position. It does this by choosing the longest axis,
    fn generate_nodes_sah(objects: &[ObjectType]) -> Box<BvhNode> {
        let bvh_data = match objects {
            [obj] => BvhNode::Object(obj.clone()),
            [a, b] => {
                let left = Box::new(BvhNode::Object(a.clone()));
                let right = Box::new(BvhNode::Object(b.clone()));
                let aabb = Aabb::encompass(a.bounding_box(), b.bounding_box());
                BvhNode::Nested { aabb, left, right }
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

                // TODO: Sometimes this seems to generate a node with a single object.
                //  It creates an (AABB->Object) node, which does double ray-aabb intersects (this is slow)

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

                let (left_split, right_split) = objects.split_at(split_index + 1);

                let left = Self::generate_nodes_sah(left_split);
                let right = Self::generate_nodes_sah(right_split);

                BvhNode::Nested {
                    aabb: main_aabb,
                    left,
                    right,
                }
            }
        };

        return Box::new(bvh_data);
    }
}

/// Given a [BvhNode] on the [Arena] tree, calculates the nearest intersection for the given `ray` and `bounds`
///
/// If the node is a [BvhNode::Object], it passes on the check to the object.
/// Otherwise, if it's a [BvhNode::Aabb], it:
///     - Tries to bail early if the [Aabb] is missed
///     - Collects all the child nodes
///     - Intersects on all those children (by calling itself recursively)
///     - Returns the closest intersection
fn bvh_node_intersect(ray: &Ray, bounds: &Bounds<Number>, node: &BvhNode) -> Option<Intersection> {
    return match node {
        // An aabb will need to delegate to child nodes if not missed
        BvhNode::Nested { left, right, aabb } => {
            if !aabb.hit(ray, bounds) {
                return None;
            }

            // TODO: Rework this to use the new Bounds::bitor API to shrink the next child's search range
            //  So keep track of the bounds, and each iteration shrink with `bounds = bounds | ..intersection.dist`
            //  And if an intersect was found in that shrunk range then we know that

            if let Some(left_int) = bvh_node_intersect(ray, bounds, left) {
                if let Some(right_int) = bvh_node_intersect(ray, bounds, right) {
                    if left_int.dist < right_int.dist {
                        Some(left_int)
                    } else {
                        Some(right_int)
                    }
                } else {
                    Some(left_int)
                }
            } else {
                bvh_node_intersect(ray, bounds, right)
            }
        }
        // Objects can be delegated directly
        BvhNode::Object(obj) => obj.intersect(ray, bounds),
    };
}

/// Given a [BvhNode] on the [Arena] tree, calculates the intersection for the given `ray`
///
/// If the node is a [BvhNode::Object], it passes on the check to the object.
/// Otherwise, if it's a [BvhNode::Aabb], it:
///     - Tries to bail early if the [Aabb] is missed
///     - Collects all the child nodes
///     - Intersects on all those children (by calling itself recursively)
///     - Returns the closest intersection
fn bvh_node_intersect_all<'a>(
    ray: &'a Ray,
    node: &'a BvhNode,
) -> Option<Box<dyn Iterator<Item = Intersection> + 'a>> {
    match node {
        // An aabb will need to delegate to child nodes if not missed
        BvhNode::Nested { left, right, aabb } => {
            if !aabb.hit(ray, &Bounds::Full(..)) {
                return None;
            }

            // TODO: Rework this to use the new Bounds::bitor API to shrink the next child's search range
            //  So keep track of the bounds, and each iteration shrink with `bounds = bounds | ..intersection.dist`
            //  And if an intersect was found in that shrunk range then we know that

            let mut intersects = [left, right]
                .into_iter()
                .filter_map(|child| bvh_node_intersect_all(ray, child))
                .flatten()
                .peekable();

            // Optimise if no elements
            intersects.peek()?;

            Some(Box::new(intersects))
        }
        // Objects can be delegated directly
        BvhNode::Object(obj) => obj.intersect_all(ray),
    }
}

impl Object for Bvh {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        // Pass everything on to our magical function
        bvh_node_intersect(ray, bounds, &self.root)
    }

    fn intersect_all<'a>(
        &'a self,
        ray: &'a Ray,
    ) -> Option<Box<dyn Iterator<Item = Intersection> + 'a>> {
        bvh_node_intersect_all(ray, &self.root)
    }

    fn bounding_box(&self) -> &Aabb {
        match &self.root {
            BvhNode::Nested { aabb, .. } => aabb,
            BvhNode::Object(o) => o.bounding_box(),
        }
    }
}

// use crate::accel::aabb::Aabb;
// use crate::object::{Object, ObjectType};
// use crate::shared::bounds::Bounds;
// use crate::shared::intersect::Intersection;
// use crate::shared::ray::Ray;
// use indextree::{Arena, Node, NodeId};
// use itertools::{zip_eq, Itertools};
// use rand::prelude::SliceRandom;
// use rand::thread_rng;
// use rayna_shared::def::types::Number;
// use std::cmp::Ordering;
//
// #[derive(Clone, Debug)]
// pub struct Bvh {
//     tree: Arena<crate::accel::bvh::BvhNode>,
//     root_id: NodeId,
// }
//
// #[derive(Copy, Clone, Debug)]
// enum SplitAxis {
//     X,
//     Y,
//     Z,
// }
//
// #[derive(Clone, Debug)]
// enum BvhNode {
//     Aabb(Aabb),
//     Object(ObjectType),
//     /// Marker for a temporary node, should not be present in the final tree
//     TempNode,
// }
//
// impl crate::accel::bvh::Bvh {
//     pub fn new(objects: &[ObjectType]) -> Self {
//         let mut tree = Arena::with_capacity(objects.len());
//         let root_id = tree.new_node(crate::accel::bvh::BvhNode::TempNode);
//
//         Self::generate_nodes_sah(objects, &mut tree, root_id);
//
//         // Ensure there are no temp nodes left in the tree
//         assert!(
//             tree.iter()
//                 .filter(|n| !n.is_removed())
//                 .all(|n| !matches!(n.get(), BvhNode::TempNode)),
//             "should not be any temp nodes in tree"
//         );
//
//         // eprintln!("\n\n{:?}\n\n", root_id.debug_pretty_print(&tree));
//
//         Self { tree, root_id }
//     }
//
//     fn aabb_axis_comparator(axis: crate::accel::bvh::SplitAxis) -> fn(&ObjectType, &ObjectType) -> Ordering {
//         match axis {
//             crate::accel::bvh::SplitAxis::X => |a, b| {
//                 PartialOrd::partial_cmp(&a.bounding_box().min().x, &b.bounding_box().min().x)
//                     .expect("should be able to cmp AABB bounds: should not be nan")
//             },
//             crate::accel::bvh::SplitAxis::Y => |a, b| {
//                 PartialOrd::partial_cmp(&a.bounding_box().min().y, &b.bounding_box().min().y)
//                     .expect("should be able to cmp AABB bounds: should not be nan")
//             },
//             crate::accel::bvh::SplitAxis::Z => |a, b| {
//                 PartialOrd::partial_cmp(&a.bounding_box().min().z, &b.bounding_box().min().z)
//                     .expect("should be able to cmp AABB bounds: should not be nan")
//             },
//         }
//     }
//
//     /// Recursively processes the slice of `objects`, adding them to the `node` recursively until
//     /// the tree is exhausted
//     fn generate_nodes(
//         objects: &[ObjectType],
//         axis: crate::accel::bvh::SplitAxis,
//         tree: &mut Arena<crate::accel::bvh::BvhNode>,
//         node_id: NodeId,
//     ) {
//         let comparator = Self::aabb_axis_comparator(axis);
//
//         let bvh_data = match objects {
//             [obj] => crate::accel::bvh::BvhNode::Object(obj.clone()),
//             [a, b] => {
//                 node_id.append(tree.new_node(crate::accel::bvh::BvhNode::Object(a.clone())), tree);
//                 node_id.append(tree.new_node(crate::accel::bvh::BvhNode::Object(b.clone())), tree);
//                 crate::accel::bvh::BvhNode::Aabb(Aabb::encompass(a.bounding_box(), b.bounding_box()))
//             }
//             objects => {
//                 let mut objects = Vec::from(objects);
//                 objects.sort_by(comparator);
//
//                 let rng = &mut thread_rng();
//                 let split_axis = [crate::accel::bvh::SplitAxis::X, crate::accel::bvh::SplitAxis::Y, crate::accel::bvh::SplitAxis::Z]
//                     .choose(rng)
//                     .unwrap();
//
//                 // split in half and repeat tree
//                 let (left, right) = objects.split_at(objects.len() / 2);
//
//                 let left_aabb = Aabb::encompass_iter(left.iter().map(Object::bounding_box));
//                 let left_node = tree.new_node(crate::accel::bvh::BvhNode::Aabb(left_aabb));
//                 node_id.append(left_node.clone(), tree);
//                 Self::generate_nodes(left, *split_axis, tree, left_node);
//
//                 let right_aabb = Aabb::encompass_iter(right.iter().map(Object::bounding_box));
//                 let right_node = tree.new_node(crate::accel::bvh::BvhNode::Aabb(right_aabb));
//                 node_id.append(right_node.clone(), tree);
//                 Self::generate_nodes(right, *split_axis, tree, right_node);
//
//                 crate::accel::bvh::BvhNode::Aabb(Aabb::encompass(left_aabb, right_aabb))
//             }
//         };
//
//         // Update the current node
//         *tree[node_id].get_mut() = bvh_data;
//     }
//
//     /// Recursively processes the slice of `objects`, adding them to the `node` recursively until
//     /// the tree is exhausted
//     ///
//     /// # **Surface-Area Heuristics** (SAH)
//     /// This method is very similar to [Self::generate_nodes], however it uses SAH to optimise the choice
//     /// of split axis, as well as split position. It does this by choosing the longest axis,
//     fn generate_nodes_sah(objects: &[ObjectType], tree: &mut Arena<crate::accel::bvh::BvhNode>, node_id: NodeId) {
//         let bvh_data = match objects {
//             [obj] => crate::accel::bvh::BvhNode::Object(obj.clone()),
//             [a, b] => {
//                 node_id.append(tree.new_node(crate::accel::bvh::BvhNode::Object(a.clone())), tree);
//                 node_id.append(tree.new_node(crate::accel::bvh::BvhNode::Object(b.clone())), tree);
//                 crate::accel::bvh::BvhNode::Aabb(Aabb::encompass(a.bounding_box(), b.bounding_box()))
//             }
//             objects => {
//                 // This is a port of [my C# port of] [Pete Shirley's code]
//                 // https://psgraphics.blogspot.com/2016/03/a-simple-sah-bvh-build.html
//                 // https://3.bp.blogspot.com/-PMG6dWk1i60/VuG9UHjsdlI/AAAAAAAACEo/BS1qJyut7LE/s1600/Screen%2BShot%2B2016-03-10%2Bat%2B11.25.08%2BAM.png
//
//                 let n = objects.len();
//                 let mut objects = Vec::from(objects);
//                 let aabbs = objects.iter().map(|o| *o.bounding_box()).collect_vec();
//                 let main_aabb = Aabb::encompass_iter(&aabbs);
//
//                 // Find the longest axis to split along, and sort for that axis
//                 // TODO: maybe choose the axis that gives the smallest overlap between the left & right splits?
//                 //  This means why try `product_of(all 3 axes, all split positions)` and find the optimal by `left.len()^2 + right.len()^2`
//
//                 // TODO: Sometimes this seems to generate a node with a single object.
//                 //  It creates an (AABB->Object) node, which does double ray-aabb intersects (this is slow)
//
//                 {
//                     let max_side = match main_aabb
//                         .size()
//                         .into_iter()
//                         .position_max_by(Number::total_cmp)
//                     {
//                         Some(0) => crate::accel::bvh::SplitAxis::X,
//                         Some(1) => crate::accel::bvh::SplitAxis::Y,
//                         Some(2) => crate::accel::bvh::SplitAxis::Z,
//                         None => unreachable!("Vector3::into_iter() cannot be empty iterator"),
//                         Some(x) => unreachable!("invalid axis {}", x),
//                     };
//                     let comparator = Self::aabb_axis_comparator(max_side);
//                     objects.sort_unstable_by(comparator);
//                 }
//
//                 // Calculate the areas of the left/right AABBs, for each given split position
//                 let (left_areas, right_areas) = {
//                     let mut left_areas = Vec::new();
//                     left_areas.resize(n, 0.);
//                     let mut right_areas = Vec::new();
//                     right_areas.resize(n, 0.);
//                     //Calculate the area from the left towards right
//                     let mut left_aabb = Aabb::default();
//                     for (area, obj_aabb) in zip_eq(left_areas.iter_mut(), aabbs.iter()) {
//                         left_aabb = Aabb::encompass(&left_aabb, obj_aabb);
//                         *area = left_aabb.area();
//                     }
//
//                     //Calculate the area from the right towards the left
//                     let mut right_aabb = Aabb::default();
//                     for (area, obj_aabb) in zip_eq(right_areas.iter_mut().rev(), aabbs.iter().rev())
//                     {
//                         right_aabb = Aabb::encompass(&right_aabb, obj_aabb);
//                         *area = right_aabb.area();
//                     }
//                     (left_areas, right_areas)
//                 };
//
//                 // Find the most optimal split index, using the areas calculated above
//                 let split_index = {
//                     // NOTE: If doing in a for loop this would be `i: 0..n-1`, and `l=left[i], r=right[i+1]`
//                     // This way we have non-overlapping left & right areas
//                     let left_trimmed = left_areas.split_last().expect("left_area is empty").1;
//                     let right_trimmed = right_areas.split_first().expect("right_area is empty").1;
//                     let min_sa_idx = zip_eq(left_trimmed, right_trimmed)
//                         .enumerate()
//                         // calculate SA
//                         .map(|(i, (&l, &r))| (i as Number * l) + ((n - i - 1) as Number * r))
//                         .position_min_by(Number::total_cmp)
//                         .expect("area iters have >1 elem");
//                     min_sa_idx
//                 };
//
//                 let (left_split, right_split) = objects.split_at(split_index + 1);
//
//                 let left_aabb = Aabb::encompass_iter(left_split.iter().map(Object::bounding_box));
//                 let left_node = tree.new_node(crate::accel::bvh::BvhNode::Aabb(left_aabb));
//                 node_id.append(left_node.clone(), tree);
//                 Self::generate_nodes_sah(left_split, tree, left_node);
//
//                 let right_aabb = Aabb::encompass_iter(right_split.iter().map(Object::bounding_box));
//                 let right_node = tree.new_node(crate::accel::bvh::BvhNode::Aabb(right_aabb));
//                 node_id.append(right_node.clone(), tree);
//                 Self::generate_nodes_sah(right_split, tree, right_node);
//
//                 crate::accel::bvh::BvhNode::Aabb(Aabb::encompass(left_aabb, right_aabb))
//             }
//         };
//
//         // Update the current node
//         *tree[node_id].get_mut() = bvh_data;
//     }
// }
//
// /// Given a [crate::accel::bvh::BvhNode] on the [Arena] tree, calculates the nearest intersection for the given `ray` and `bounds`
// ///
// /// If the node is a [crate::accel::bvh::BvhNode::Object], it passes on the check to the object.
// /// Otherwise, if it's a [crate::accel::bvh::BvhNode::Aabb], it:
// ///     - Tries to bail early if the [Aabb] is missed
// ///     - Collects all the child nodes
// ///     - Intersects on all those children (by calling itself recursively)
// ///     - Returns the closest intersection
// fn bvh_node_intersect(
//     ray: &Ray,
//     bounds: &Bounds<Number>,
//     node_id: NodeId,
//     node: &Node<crate::accel::bvh::BvhNode>,
//     tree: &Arena<crate::accel::bvh::BvhNode>,
// ) -> Option<Intersection> {
//     match node.get() {
//         crate::accel::bvh::BvhNode::TempNode => unreachable!("asserted that tree has no temp nodes"),
//         // An aabb will need to delegate to child nodes if not missed
//         crate::accel::bvh::BvhNode::Aabb(aabb) => {
//             if !aabb.hit(ray, bounds) {
//                 return None;
//             }
//
//             let children = node_id
//                 .children(tree)
//                 .map(|node_id| (node_id, &tree[node_id]))
//                 // .collect_vec()
//                 ;
//
//             // TODO: Rework this to use the new Bounds::bitor API to shrink the next child's search range
//             //  So keep track of the bounds, and each iteration shrink with `bounds = bounds | ..intersection.dist`
//             //  And if an intersect was found in that shrunk range then we know that
//             let intersects = children.into_iter().filter_map(|(child_id, child)| {
//                 crate::accel::bvh::bvh_node_intersect(ray, bounds, child_id, child, tree)
//             });
//
//             intersects.min_by(|a, b| Number::total_cmp(&a.dist, &b.dist))
//         }
//         // Objects can be delegated directly
//         crate::accel::bvh::BvhNode::Object(obj) => {
//             assert!(
//                 node_id.children(tree).next().is_none(),
//                 "a node with an object attached should have no children"
//             );
//             obj.intersect(ray, bounds)
//         }
//     }
// }
//
// /// Given a [crate::accel::bvh::BvhNode] on the [Arena] tree, calculates the intersection for the given `ray`
// ///
// /// If the node is a [crate::accel::bvh::BvhNode::Object], it passes on the check to the object.
// /// Otherwise, if it's a [crate::accel::bvh::BvhNode::Aabb], it:
// ///     - Tries to bail early if the [Aabb] is missed
// ///     - Collects all the child nodes
// ///     - Intersects on all those children (by calling itself recursively)
// ///     - Returns the closest intersection
// fn bvh_node_intersect_all<'a>(
//     ray: &'a Ray,
//     node_id: NodeId,
//     node: &'a Node<crate::accel::bvh::BvhNode>,
//     tree: &'a Arena<crate::accel::bvh::BvhNode>,
// ) -> Option<Box<dyn Iterator<Item = Intersection> + 'a>> {
//     match node.get() {
//         crate::accel::bvh::BvhNode::TempNode => unreachable!("asserted that tree has no temp nodes"),
//         // An aabb will need to delegate to child nodes if not missed
//         crate::accel::bvh::BvhNode::Aabb(aabb) => {
//             if !aabb.hit(ray, &Bounds::Full(..)) {
//                 return None;
//             }
//
//             let children = node_id
//                 .children(tree)
//                 .map(|node_id| (node_id, &tree[node_id]))
//                 // .collect_vec()
//                 ;
//
//             // TODO: Rework this to use the new Bounds::bitor API to shrink the next child's search range
//             //  So keep track of the bounds, and each iteration shrink with `bounds = bounds | ..intersection.dist`
//             //  And if an intersect was found in that shrunk range then we know that
//             let mut intersects = children
//                 .into_iter()
//                 .filter_map(|(child_id, child)| crate::accel::bvh::bvh_node_intersect_all(ray, child_id, child, tree))
//                 .flatten()
//                 .peekable();
//
//             // Optimise if no elements
//             intersects.peek()?;
//
//             Some(Box::new(intersects))
//         }
//         // Objects can be delegated directly
//         crate::accel::bvh::BvhNode::Object(obj) => {
//             assert!(
//                 node_id.children(tree).next().is_none(),
//                 "a node with an object attached should have no children"
//             );
//
//             obj.intersect_all(ray)
//         }
//     }
// }
//
// impl Object for crate::accel::bvh::Bvh {
//     fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
//         // Pass everything on to our magical function
//         crate::accel::bvh::bvh_node_intersect(
//             ray,
//             bounds,
//             self.root_id,
//             &self.tree[self.root_id],
//             &self.tree,
//         )
//     }
//
//     fn intersect_all<'a>(
//         &'a self,
//         ray: &'a Ray,
//     ) -> Option<Box<dyn Iterator<Item = Intersection> + 'a>> {
//         crate::accel::bvh::bvh_node_intersect_all(ray, self.root_id, &self.tree[self.root_id], &self.tree)
//     }
//
//     fn bounding_box(&self) -> &Aabb {
//         match self.tree[self.root_id].get() {
//             crate::accel::bvh::BvhNode::TempNode => unreachable!(),
//             crate::accel::bvh::BvhNode::Aabb(a) => a,
//             crate::accel::bvh::BvhNode::Object(o) => o.bounding_box(),
//         }
//     }
// }
