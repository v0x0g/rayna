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

use crate::core::types::{Number, Point3};
use crate::shared::aabb::Bounded;
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use crate::shared::token::generate_component_token;
use crate::shared::ComponentRequirements;
use enum_dispatch::enum_dispatch;
use rand_core::RngCore;
// noinspection ALL - Used by enum_dispatch macro
#[allow(unused_imports)]
use axis_box::AxisBoxMesh;
// noinspection ALL - Used by enum_dispatch macro
#[allow(unused_imports)]
use cylinder::CylinderMesh;
// noinspection ALL - Used by enum_dispatch macro
#[allow(unused_imports)]
use list::ListMesh;
// noinspection ALL - Used by enum_dispatch macro
#[allow(unused_imports)]
use polygonised::PolygonisedIsosurfaceMesh;
// noinspection ALL - Used by enum_dispatch macro
#[allow(unused_imports)]
use raymarched::RaymarchedIsosurfaceMesh;
// noinspection ALL - Used by enum_dispatch macro
use crate::scene::Scene;
#[allow(unused_imports)]
use sphere::SphereMesh;
// noinspection ALL - Used by enum_dispatch macro
#[allow(unused_imports)]
use self::{
    advanced::{batch_triangle::BatchTriangle, bvh_mesh::BvhMesh, dynamic::DynamicMesh},
    planar::{infinite_plane::InfinitePlaneMesh, parallelogram::ParallelogramMesh},
};

pub mod advanced;
pub mod axis_box;
pub mod cylinder;
pub mod isosurface;
pub mod list;
pub mod planar;
pub mod polygonised;
pub mod primitive;
pub mod raymarched;
pub mod sphere;
pub mod triangle;
// region Object traits

#[enum_dispatch]
#[doc(notable_trait)]
pub trait Mesh: ComponentRequirements {
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
    ) -> Option<Intersection>;

    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info (shadow checks)
}

/// An optimised implementation of [Mesh].
///
/// See [`crate::material::MaterialInstance`] for an explanation of the [`macro@enum_dispatch`] macro usage
#[enum_dispatch(Mesh)]
#[derive(Clone, Debug)]
pub enum MeshInstance {
    SphereMesh,
    CylinderMesh,
    AxisBoxMesh,
    ParallelogramMesh,
    InfinitePlaneMesh,
    RaymarchedIsosurfaceMesh,
    PolygonisedIsosurfaceMesh,
    TriangleMesh,
    MeshList,
}

generate_component_token!(MeshToken for MeshInstance);

// endregion Object traits
