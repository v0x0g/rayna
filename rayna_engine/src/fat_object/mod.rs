mod transformed;

use crate::mesh::Object;
use crate::shared::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;
use rand_core::RngCore;
use rayna_shared::def::types::Number;
use smallvec::SmallVec;
use transformed::TransformedFatObject;

/// This trait is essentially an extension of [Object], but with a [FullIntersection] not [Intersection],
/// meaning the material of the mesh is also included.
///
/// This should only be implemented on [TransformedFatObject], and any objects that group multiple objects together.
///
/// It's a bit of an implementation detail
pub trait FullObject {
    /// Attempts to perform an intersection between the given ray and the target mesh
    ///
    /// # Return Value
    /// This should return the *first* intersection that is within the given range, else [None]
    fn full_intersect<'o>(
        &'o self,
        ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o>>;

    /// Calculates all of the intersections for the given mesh.
    ///
    /// # Return Value
    /// This should append all the (unbounded) intersections, into the vector `output`.
    /// It can *not* be assumed this vector will be empty. The existing contents should not be modified
    fn full_intersect_all<'o>(
        &'o self,
        ray: &Ray,
        output: &mut SmallVec<[FullIntersection<'o>; 32]>,

        rng: &mut dyn RngCore,
    );

    fn aabb(&self) -> Option<&Aabb>;
}
