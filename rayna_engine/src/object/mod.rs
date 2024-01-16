pub mod bvh;
pub mod list;
pub mod simple;

use crate::material::Material;
use crate::mesh::Mesh as MeshTrait;
use crate::shared::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use rand_core::RngCore;
use rayna_shared::def::types::Number;
use smallvec::SmallVec;

// noinspection ALL
use self::{bvh::BvhObject, list::ObjectList, simple::SimpleObject};

dyn_clone::clone_trait_object!(<Mesh: MeshTrait + Clone, Mat: Material + Clone> Object<Mesh=Mesh, Mat=Mat>);

/// This trait is essentially an extension of [Mesh], but with a [FullIntersection] not [Intersection],
/// meaning the material of the mesh is also included.
///
/// This should only be implemented on [SimpleObject], and any objects that group multiple objects together.
pub trait Object: RtRequirement {
    type Mesh: MeshTrait + Clone;
    type Mat: Material + Clone;
    /// Attempts to perform an intersection between the given ray and the target mesh
    ///
    /// # Return Value
    /// This should return the *first* intersection that is within the given range, else [None]
    fn full_intersect<'o>(
        &'o self,
        ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Self::Mat>>;

    /// Calculates all of the intersections for the given mesh.
    ///
    /// # Return Value
    /// This should append all the (unbounded) intersections, into the vector `output`.
    /// It can *not* be assumed this vector will be empty. The existing contents should not be modified
    fn full_intersect_all<'o>(
        &'o self,
        ray: &Ray,
        output: &mut SmallVec<[FullIntersection<'o, Self::Mat>; 32]>,
        rng: &mut dyn RngCore,
    );

    fn aabb(&self) -> Option<&Aabb>;
}

#[derive(Clone, Debug)]
pub enum ObjectInstance<Mesh: MeshTrait + Clone, Mat: Material + Clone> {
    SimpleObject(SimpleObject<Mesh, Mat>),
    ObjectList(ObjectList<Mesh, Mat, ObjectInstance<Mesh, Mat>>),
    Bvh(BvhObject<Mesh, Mat, ObjectInstance<Mesh, Mat>>),
}

// `enum_dispatch` doesn't support associated type bounds, so we have to do manual impl
impl<Mesh: MeshTrait + Clone, Mat: Material + Clone> Object for ObjectInstance<Mesh, Mat> {
    type Mesh = Mesh;
    type Mat = Mat;

    fn full_intersect<'o>(
        &'o self,
        ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Self::Mat>> {
        match self {
            Self::Bvh(v) => v.full_intersect(ray, bounds, rng),
            Self::SimpleObject(v) => v.full_intersect(ray, bounds, rng),
            Self::ObjectList(v) => v.full_intersect(ray, bounds, rng),
        }
    }

    fn full_intersect_all<'o>(
        &'o self,
        ray: &Ray,
        output: &mut SmallVec<[FullIntersection<'o, Self::Mat>; 32]>,
        rng: &mut dyn RngCore,
    ) {
        match self {
            Self::Bvh(v) => v.full_intersect_all(ray, output, rng),
            Self::SimpleObject(v) => v.full_intersect_all(ray, output, rng),
            Self::ObjectList(v) => v.full_intersect_all(ray, output, rng),
        }
    }

    fn aabb(&self) -> Option<&Aabb> {
        match self {
            Self::Bvh(v) => v.aabb(),
            Self::SimpleObject(v) => v.aabb(),
            Self::ObjectList(v) => v.aabb(),
        }
    }
}
