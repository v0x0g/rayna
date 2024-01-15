//! This module contains utility functions for helping with scene mesh transformations
//!
//! # Terminology
//!
//! ## Transform
//! The mesh's 3D affine transform matrix (see [Transform3]). This represents the transformation from
//! mesh-space to world-space; e.g. a [Transform3::from_scale()] with a scale of `Vector3::splat(2.)`,
//! would cause the mesh to appear twice as large.
//!
//! ## Inverse Transform
//! The matrix inverse of `transform`. This is the matrix corresponding to the transformation from
//! mesh-space to world-space

use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use rayna_shared::def::types::{Point3, Transform3, Vector3};

/// Transforms the incoming ray from world-space to mesh-space, using the mesh's inverse transform
pub fn transform_incoming_ray(incoming_ray: &Ray, inv_transform: &Transform3) -> Ray {
    let (pos, dir) = incoming_ray.into();
    Ray::new(inv_transform.map_point(pos), inv_transform.map_vector(dir))
}

/// Transforms the outgoing intersection from mesh-space to world-space
pub fn transform_outgoing_intersection(
    original_ray: &Ray,
    intersection: &Intersection,
    transform: &Transform3,
) -> Intersection {
    // PANICS:
    // We use `.unwrap()` on the results of the transformations
    // Since it is of type `Transform3`, it must be an invertible matrix and can't collapse
    // the dimensional space to <3 dimensions,
    // Therefore all transformation should be valid (and for vectors: nonzero), so we can unwrap

    let mut intersection = intersection.clone();

    let point = |p: &mut Point3| *p = transform.matrix.transform_point(*p);
    let normal = |n: &mut Vector3| {
        let t = transform.map_vector(*n);
        *n = t.try_normalize().expect(&format!(
            "transformation failed: vector {n:?} transformed to {t:?} couldn't be normalised"
        ))
    };

    normal(&mut intersection.normal);
    normal(&mut intersection.ray_normal);
    point(&mut intersection.pos_l);
    point(&mut intersection.pos_w);

    // Minor hack, calculate the intersection distance instead of transforming it
    // I don't know how else to do this lol
    intersection.dist = (intersection.pos_w - original_ray.pos()).length();

    return intersection;
}
