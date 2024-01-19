//! # Module [crate::mesh]
//!
//! This module contains the submodules for different mesh (see [Mesh] and [MeshInstance]) types.
//!
//! ## Related
//! - [Mesh]
//! - [MeshInstance]
//! - [sphere]
//!
//! # DEV: Code Structure
//!
//! ## Object Modules
//! Objects (and their corresponding types) are placed into named submodules, and those submodules
//! are publicly exported. The object type should have an entry placed in [MeshInstance]
//!
//! TODO: Move this to a readme, or the project-level documentation
//!
//! ## Example
//! Considering a "Sphere" mesh:
//!
//! - File: `./sphere.rs`
//! - Add module: `pub mod sphere;`
//! - Structs: `SphereObject`, with constructors `SphereObject::new()` etc
//! - Add an entry to [MeshInstance] to correspond to the `SphereObject` for static-dispatch
//! - See [sphere] for an example

use crate::shared::aabb::HasAabb;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use enum_dispatch::enum_dispatch;
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Point3};
// noinspection ALL - Used by enum_dispatch macro
#[allow(unused_imports)]
use self::{
    axis_box::AxisBoxMesh, bvh::BvhMesh, dynamic::DynamicMesh, infinite_plane::InfinitePlaneMesh, list::MeshList,
    parallelogram::ParallelogramMesh, sphere::SphereMesh, triangle::TriangleMesh,
};

pub mod axis_box;
pub mod bvh;
pub mod dynamic;
pub mod infinite_plane;
pub mod list;
pub mod parallelogram;
pub mod planar;
pub mod sphere;
pub mod triangle;

// region Object traits

#[enum_dispatch]
pub trait Mesh: MeshProperties + RtRequirement {
    /// Attempts to perform an intersection between the given ray and the target mesh
    ///
    /// # Return Value
    /// This should return the *first* intersection that is within the given range, else [None]
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, rng: &mut dyn RngCore) -> Option<Intersection>;

    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info (shadow checks)
}

/// An optimised implementation of [Mesh].
///
/// See [crate::material::MaterialInstance] for an explanation of the [macro@enum_dispatch] macro usage
#[enum_dispatch(Mesh, MeshProperties, HasAabb)]
#[derive(Clone, Debug)]
pub enum MeshInstance {
    SphereMesh,
    AxisBoxMesh,
    ParallelogramMesh,
    InfinitePlaneMesh,
    TriangleMesh,
    BvhMesh(BvhMesh<MeshInstance>),
    MeshList(MeshList<MeshInstance>),
    DynamicMesh,
}

/// This trait describes an [Mesh], and the properties it has
#[enum_dispatch]
pub trait MeshProperties: RtRequirement + HasAabb {
    /// Gets the centre of the mesh.
    ///
    /// Scaling and rotation will happen around this point
    fn centre(&self) -> Point3;
}

// endregion Object traits
