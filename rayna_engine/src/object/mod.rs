//! # Module [crate::object]
//!
//! This module contains the submodules for different object (see [Object] and [ObjectInstance]) types.
//!
//! ## Related
//! - [Object]
//! - [ObjectInstance]
//! - [sphere]
//!
//! # DEV: Code Structure
//!
//! ## Object Modules
//! Objects (and their corresponding types) are placed into named submodules, and those submodules
//! are publicly exported. Objects should be split into a "Builder" struct, which contains the publicly accessible properties
//! for the type, and an "Object" struct which contains the 'built' object (which may contain cached values for performance, and
//! should be immutable/private fields)
//!
//! ## Example
//! Considering a "Sphere" object:
//!
//! - File: `./sphere.rs`
//! - Add module: `pub mod sphere;`
//! - Structs: `SphereBuilder`, which is translated into `SphereObject`, where `SphereObject: Object`
//! - Add an entry to [ObjectInstance] to correspond to the `SphereObject` for static-dispatch
//! - See [sphere] for an example

use crate::accel::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use enum_dispatch::enum_dispatch;
use rayna_shared::def::types::{Number, Point3};
use smallvec::SmallVec;
// noinspection ALL - Used by enum_dispatch macro
#[allow(unused_imports)]
use self::{
    axis_box::AxisBoxObject, dynamic::DynamicObject, infinite_plane::InfinitePlaneObject,
    parallelogram::ParallelogramObject, sphere::SphereObject,
};

pub mod axis_box;
pub mod dynamic;
mod infinite_plane;
pub mod parallelogram;
pub mod planar;
pub mod sphere;
pub mod transformed;
pub mod triangle;

dyn_clone::clone_trait_object!(Object);

#[enum_dispatch]
pub trait Object: ObjectProperties + RtRequirement {
    /// Attempts to perform an intersection between the given ray and the target object
    ///
    /// # Return Value
    /// This should return the *first* intersection that is within the given range, else [None]
    fn intersect<'o>(&'o self, ray: &Ray, bounds: &Bounds<Number>) -> Option<FullIntersection<'o>>;

    /// Returns (possibly) an iterator over all of the intersections for the given object.
    ///
    /// # Return Value
    /// This should place all the (unbounded) intersections, into the vector `output`.
    /// It can be assumed this vector will be empty.
    fn intersect_all<'o>(&'o self, ray: &Ray, output: &mut SmallVec<[FullIntersection<'o>; 32]>);

    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info (shadow checks)
}

/// An optimised implementation of [Object].
///
/// See [crate::material::MaterialInstance] for an explanation of the [macro@enum_dispatch] macro usage
#[enum_dispatch(Object, ObjectProperties)]
#[derive(Clone, Debug)]
pub enum ObjectInstance {
    SphereObject,
    AxisBoxObject,
    ParallelogramObject,
    InfinitePlaneObject,
    DynamicObject,
}

dyn_clone::clone_trait_object!(ObjectProperties);

/// This trait describes an [Object], and the properties it has
#[enum_dispatch]
pub trait ObjectProperties: RtRequirement {
    /// Gets the bounding box for this object. If the object can't be bounded (e.g. infinite plane), return [None]
    fn aabb(&self) -> Option<&Aabb>;

    /// Gets the centre of the object.
    ///
    /// Scaling and rotation will happen around this point
    fn centre(&self) -> Point3;
}
