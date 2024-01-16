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
//! are publicly exported. Objects should be split into a "Builder" struct, which contains the publicly accessible properties
//! for the type, and an "Object" struct which contains the 'built' mesh (which may contain cached values for performance, and
//! should be immutable/private fields)
//!
//! ## Example
//! Considering a "Sphere" mesh:
//!
//! - File: `./sphere.rs`
//! - Add module: `pub mod sphere;`
//! - Structs: `SphereBuilder`, which is translated into `SphereObject`, where `SphereObject: Object`
//! - Add an entry to [MeshInstance] to correspond to the `SphereObject` for static-dispatch
//! - See [sphere] for an example

use crate::shared::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use enum_dispatch::enum_dispatch;
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Point3};
use smallvec::SmallVec;
// noinspection ALL - Used by enum_dispatch macro
#[allow(unused_imports)]
use self::{
    axis_box::AxisBoxMesh, dynamic::DynamicMesh, homogenous_volume::HomogeneousVolumeMesh,
    infinite_plane::InfinitePlaneMesh, parallelogram::ParallelogramMesh, sphere::SphereMesh, triangle::TriangleMesh,
};

pub mod axis_box;
pub mod dynamic;
pub mod group;
pub mod homogenous_volume;
pub mod infinite_plane;
pub mod parallelogram;
pub mod planar;
pub mod sphere;
pub mod triangle;

// region Object traits

dyn_clone::clone_trait_object!(Mesh);

#[enum_dispatch]
pub trait Mesh: MeshProperties + RtRequirement {
    /// Attempts to perform an intersection between the given ray and the target mesh
    ///
    /// # Return Value
    /// This should return the *first* intersection that is within the given range, else [None]
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, rng: &mut dyn RngCore) -> Option<Intersection>;

    /// Returns (possibly) an iterator over all of the intersections for the given mesh.
    ///
    /// # Return Value
    /// This should place all the (unbounded) intersections, into the vector `output`.
    /// It can be assumed this vector will be empty.
    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>, rng: &mut dyn RngCore);

    // TODO: Should I remove `intersect_all()`?
    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info (shadow checks)
}

/// An optimised implementation of [Mesh].
///
/// See [crate::material::MaterialInstance] for an explanation of the [macro@enum_dispatch] macro usage
#[enum_dispatch(Mesh, MeshProperties)]
#[derive(Clone, Debug)]
pub enum MeshInstance {
    SphereMesh,
    AxisBoxMesh,
    ParallelogramMesh,
    InfinitePlaneMesh,
    TriangleMesh,
    HomogeneousVolumeMesh(HomogeneousVolumeMesh<DynamicMesh>),
    DynamicMesh,
}

dyn_clone::clone_trait_object!(MeshProperties);

/// This trait describes an [Mesh], and the properties it has
#[enum_dispatch]
pub trait MeshProperties: RtRequirement {
    /// Gets the bounding box for this mesh. If the mesh can't be bounded (e.g. infinite plane), return [None]
    fn aabb(&self) -> Option<&Aabb>;

    /// Gets the centre of the mesh.
    ///
    /// Scaling and rotation will happen around this point
    fn centre(&self) -> Point3;
}

// endregion Object traits
