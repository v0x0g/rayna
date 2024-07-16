//! # Module [crate::mesh]
//!
//! This module contains the submodules for different mesh (see [Mesh] and [MeshInstance]) types.
//!
//! ## Related
//! - [`self::Mesh`]
//! - [`self::MeshInstance`]
//! - [`self::primitive::sphere::SphereMesh`]
//!
//! # Code Structure
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
//! - See [`self::primitive::sphere`] for an example

use crate::core::types::{Number, Point3};
use crate::shared::aabb::HasAabb;
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use crate::shared::ComponentRequirements;
use enum_dispatch::enum_dispatch;
use rand_core::RngCore;
// noinspection ALL - Used by enum_dispatch macro
#[allow(unused_imports)]
use self::{
    advanced::{bvh::BvhMesh, dynamic::DynamicMesh, list::MeshList, triangle::BatchTriangle},
    isosurface::{polygonised::PolygonisedIsosurfaceMesh, raymarched::RaymarchedIsosurfaceMesh},
    planar::{infinite_plane::InfinitePlaneMesh, parallelogram::ParallelogramMesh},
    primitive::{axis_box::AxisBoxMesh, cylinder::CylinderMesh, sphere::SphereMesh},
};

pub mod advanced;
pub mod isosurface;
pub mod planar;
pub mod primitive;

// region Object traits

#[enum_dispatch]
#[doc(notable_trait)]
pub trait Mesh: MeshProperties + ComponentRequirements {
    /// Attempts to perform an intersection between the given ray and the target mesh
    ///
    /// # Return Value
    /// This should return the *first* intersection that is within the given range, else [None]
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, rng: &mut dyn RngCore) -> Option<Intersection>;

    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info (shadow checks)
}

/// An optimised implementation of [Mesh].
///
/// See [`crate::material::MaterialInstance`] for an explanation of the [`macro@enum_dispatch`] macro usage
#[enum_dispatch(Mesh, MeshProperties, HasAabb)]
#[derive(Clone, Debug)]
pub enum MeshInstance {
    SphereMesh,
    CylinderMesh,
    AxisBoxMesh,
    ParallelogramMesh,
    InfinitePlaneMesh,
    RaymarchedIsosurfaceMesh,
    PolygonisedIsosurfaceMesh,
    BatchTriangle1(BatchTriangle<1>),
    BatchTriangle2(BatchTriangle<2>),
    BatchTriangle4(BatchTriangle<4>),
    BatchTriangle8(BatchTriangle<8>),
    BatchTriangle16(BatchTriangle<16>),
    TriangleMesh(primitive::triangle::Triangle),
    BvhMesh(BvhMesh<MeshInstance>),
    MeshList(MeshList<MeshInstance>),
    DynamicMesh,
}

/// This trait describes an [Mesh], and the properties it has
#[enum_dispatch]
pub trait MeshProperties: ComponentRequirements + HasAabb {
    /// Gets the centre of the mesh.
    ///
    /// Scaling and rotation will happen around this point
    fn centre(&self) -> Point3;
}

// endregion Object traits
