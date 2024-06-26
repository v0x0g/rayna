pub mod bvh;
pub mod list;
pub mod simple;
pub mod transform;
pub mod volumetric;

use crate::core::types::Number;
use crate::material::Material;
use crate::mesh::Mesh as MeshTrait;
use crate::shared::aabb::Aabb;
use crate::shared::aabb::HasAabb;
use crate::shared::intersect::FullIntersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use rand_core::RngCore;

// noinspection ALL
use self::{bvh::BvhObject, list::ObjectList, simple::SimpleObject, volumetric::VolumetricObject};

// TODO: Should objects (as well as other traits) have some sort of identifier?

/// This trait is essentially an extension of [`MeshTrait`], but with a
/// [`FullIntersection`] not [Intersection](`crate::shared::intersect::Intersection`),
/// meaning the material of the mesh is also included.
///
/// This should only be implemented on [`SimpleObject`], and any objects that group multiple objects together.
#[doc(notable_trait)]
pub trait Object: RtRequirement + HasAabb {
    type Mesh: MeshTrait;
    type Mat: Material;
    /// Attempts to perform an intersection between the given ray and the target mesh
    ///
    /// # Return Value
    /// This should return the *first* intersection that is within the given range, else [`None`]
    fn full_intersect<'o>(
        &'o self,
        ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Self::Mat>>;
}

// region Static dispatch

#[derive(Clone, Debug)]
pub enum ObjectInstance<Mesh: MeshTrait + Clone, Mat: Material + Clone> {
    SimpleObject(SimpleObject<Mesh, Mat>),
    VolumetricObject(VolumetricObject<Mesh, Mat>),
    ObjectList(ObjectList<ObjectInstance<Mesh, Mat>>),
    Bvh(BvhObject<ObjectInstance<Mesh, Mat>>),
}

// `enum_dispatch` doesn't support associated type interval, so we have to do manual impl
impl<Mesh: MeshTrait + Clone, Mat: Material + Clone> Object for ObjectInstance<Mesh, Mat> {
    type Mesh = Mesh;
    type Mat = Mat;

    fn full_intersect<'o>(
        &'o self,
        ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Self::Mat>> {
        match self {
            Self::Bvh(v) => v.full_intersect(ray, interval, rng),
            Self::SimpleObject(v) => v.full_intersect(ray, interval, rng),
            Self::VolumetricObject(v) => v.full_intersect(ray, interval, rng),
            Self::ObjectList(v) => v.full_intersect(ray, interval, rng),
        }
    }
}

impl<Mesh: MeshTrait + Clone, Mat: Material + Clone> HasAabb for ObjectInstance<Mesh, Mat> {
    fn aabb(&self) -> Option<&Aabb> {
        match self {
            Self::Bvh(v) => v.aabb(),
            Self::SimpleObject(v) => v.aabb(),
            Self::VolumetricObject(v) => v.aabb(),
            Self::ObjectList(v) => v.aabb(),
        }
    }
}

// endregion Static dispatch

// region impl From<_> for ObjectInstance

// NOTE: Since [ObjectInstance] is [Clone], the wrapped meshes and materials also need to be [Clone]

impl<Mesh: MeshTrait + Clone, Mat: Material + Clone> From<SimpleObject<Mesh, Mat>> for ObjectInstance<Mesh, Mat> {
    fn from(value: SimpleObject<Mesh, Mat>) -> Self { Self::SimpleObject(value) }
}
impl<Mesh: MeshTrait + Clone, Mat: Material + Clone> From<VolumetricObject<Mesh, Mat>> for ObjectInstance<Mesh, Mat> {
    fn from(value: VolumetricObject<Mesh, Mat>) -> Self { Self::VolumetricObject(value) }
}
impl<Mesh: MeshTrait + Clone, Mat: Material + Clone> From<ObjectList<ObjectInstance<Mesh, Mat>>>
    for ObjectInstance<Mesh, Mat>
{
    fn from(value: ObjectList<ObjectInstance<Mesh, Mat>>) -> Self { Self::ObjectList(value) }
}
impl<Mesh: MeshTrait + Clone, Mat: Material + Clone> From<BvhObject<ObjectInstance<Mesh, Mat>>>
    for ObjectInstance<Mesh, Mat>
{
    fn from(value: BvhObject<ObjectInstance<Mesh, Mat>>) -> Self { Self::Bvh(value) }
}

// endregion impl From<_> for ObjectInstance
