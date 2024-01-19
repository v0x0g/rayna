//! This module contains utility functions for helping with scene mesh/object transformations
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

use crate::shared::aabb::Aabb;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use getset::Getters;
use rayna_shared::def::types::{Point3, Transform3, Vector3};

/// A struct that holds both a [Transform3] and it's inverse.
#[derive(Copy, Clone, Debug, Getters)]
#[get = "pub"]
pub struct ObjectTransform {
    // TODO: I would like to have this generic over `<Src, Dst>`, but I can't access the traits to properly
    //  constrain the values (e.g. `glamour::scalar::FloatScalar`). Find out if we can fix this somehow.
    //  Maybe make a PR/issue or contact the devs?
    /// Object to world transform
    transform: Transform3,
    /// World to object transform
    inv_transform: Transform3,
    /// Is this transform the identity transform?
    is_identity: bool,
}

// region Creating

impl ObjectTransform {
    pub const IDENTITY: Self = Self {
        transform: Transform3::IDENTITY,
        inv_transform: Transform3::IDENTITY,
        is_identity: true,
    };

    /// Creates a new (uncorrected) transform object
    pub fn new(transform: Transform3) -> Self {
        Self {
            transform,
            inv_transform: transform.inverse(),
            is_identity: transform == Transform3::IDENTITY,
        }
    }

    /// Creates a new transformed mesh instance, using the given mesh and transform matrix.
    ///
    /// Unlike [Self::new()], this *does* account for the mesh's translation from the origin,
    /// using the `obj_centre` parameter. See field documentation ([Self::transform]) for explanation
    /// and example of this position offset correction
    pub fn new_corrected(transform: Transform3, obj_centre: impl Into<Point3>) -> Self {
        let obj_centre = obj_centre.into();
        let correct_transform = Transform3::from_translation(-obj_centre.to_vector())
            .then(transform)
            .then_translate(obj_centre.to_vector());

        Self::new(correct_transform)
    }

    /// Applies transform correction to the `self` transform
    pub fn with_correction(&self, obj_centre: impl Into<Point3>) -> Self {
        Self::new_corrected(self.transform, obj_centre)
    }
}

impl From<Transform3> for ObjectTransform {
    fn from(value: Transform3) -> Self { Self::new(value) }
}

// Allows us to use `None` as a transform
impl From<Option<ObjectTransform>> for ObjectTransform {
    fn from(t: Option<ObjectTransform>) -> Self { t.unwrap_or_default() }
}

impl Default for ObjectTransform {
    fn default() -> Self { Self::IDENTITY }
}

// endregion Creating

// region Transformation

impl ObjectTransform {
    /// Transforms the incoming ray from world-space to mesh-space, using the mesh's inverse transform
    pub fn incoming_ray(&self, incoming_ray: &Ray) -> Ray {
        if self.is_identity {
            return *incoming_ray;
        }

        let (pos, dir) = incoming_ray.into();
        Ray::new(self.inv_transform.map_point(pos), self.inv_transform.map_vector(dir))
    }

    /// Transforms the outgoing intersection from mesh-space to world-space
    pub fn outgoing_intersection(&self, original_ray: &Ray, mut intersection: Intersection) -> Intersection {
        if self.is_identity {
            return intersection;
        }

        // PANICS:
        // We use `.unwrap()` on the results of the transformations
        // Since it is of type `Transform3`, it must be an invertible matrix and can't collapse
        // the dimensional space to <3 dimensions,
        // Therefore all transformation should be valid (and for vectors: nonzero), so we can unwrap

        let point = |p: &mut Point3| *p = self.transform.matrix.transform_point(*p);
        let normal = |n: &mut Vector3| {
            let t = self.transform.map_vector(*n);
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

    /// Given a transform and (optional) AABB, calculates the new AABB given that transform
    pub fn calculate_aabb(&self, aabb: Option<&Aabb>) -> Option<Aabb> {
        if self.is_identity {
            aabb.copied()
        } else {
            aabb.map(Aabb::corners)
                .map(|corners| corners.map(|c| self.transform.map_point(c)))
                .map(Aabb::encompass_points)
        }
    }
}

// endregion Transformation
