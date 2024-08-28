//! # Module [crate::mesh]
//!
//! This module contains the submodules for different mesh (see [Mesh] and [MeshInstance]) types.
//!
//! ## Related
//! - [`self::Mesh`]
//! - [`self::MeshInstance`]
//! - [`sphere::SphereMesh`]
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
//! - See [`sphere`] for an example

use crate::core::aabb::Aabb;
use crate::core::aabb::Bounded;
use crate::core::component::Component;
use crate::core::intersect::MeshIntersection;
use crate::core::interval::Interval;
use crate::core::ray::Ray;
use crate::core::token::generate_component_token;
use crate::core::types::Number;
use crate::scene::Scene;
use rand_core::RngCore;

pub mod axis_box;
pub mod cylinder;
pub mod list;
pub mod planar;
pub mod polygonised;
pub mod raymarched;
pub mod sphere;
pub mod triangle;
// region Object traits

#[enum_dispatch::enum_dispatch]
pub trait Mesh: Component + Bounded {
    /// Attempts to perform an intersection between the given ray and the target mesh
    ///
    /// # Return Value
    /// This should return the *first* intersection that is within the given range, else [None]
    fn intersect(
        &self,
        scene: &Scene,
        ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<MeshIntersection>;

    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info (shadow checks)
}

/// An optimised implementation of [Mesh].
///
/// See [`crate::material::MaterialInstance`] for an explanation of the [`macro@enum_dispatch`] macro usage
#[enum_dispatch::enum_dispatch(Mesh, Bounded)]
#[derive(Clone, Debug)]
pub enum MeshInstance {
    SphereMesh(self::sphere::SphereMesh),
    CylinderMesh(self::cylinder::CylinderMesh),
    AxisBoxMesh(self::axis_box::AxisBoxMesh),
    ParallelogramMesh(self::planar::ParallelogramMesh),
    InfinitePlaneMesh(self::planar::InfinitePlaneMesh),
    RaymarchedIsosurfaceMesh(self::raymarched::RaymarchedIsosurfaceMesh),
    PolygonisedIsosurfaceMesh(self::polygonised::PolygonisedIsosurfaceMesh),
    TriangleMesh(self::triangle::TriangleMesh),
    MeshList(self::list::ListMesh),
}

generate_component_token!(MeshToken for MeshInstance);

// endregion Object traits
