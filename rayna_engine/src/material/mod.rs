//noinspection ALL
use self::{
    dielectric::DielectricMaterial, dynamic::DynamicMaterial, lambertian::LambertianMaterial,
    metal::MetalMaterial,
};
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use enum_dispatch::enum_dispatch;
use rand::RngCore;
use rayna_shared::def::types::{Pixel, Vector3};

pub mod dielectric;
pub mod dynamic;
pub mod lambertian;
pub mod metal;

/// The trait that defines what properties a material has
#[enum_dispatch]
pub trait Material: RtRequirement {
    // TODO: Should `scatter()` return a ray?

    /// Scatters the input ray, according to the material's properties
    ///
    /// # Arguments
    ///
    /// * `ray`: The incoming ray that should be scattered
    /// * `intersection`: Information about the intersection we are calculating the scatter for
    ///     Includes surface normals, etc
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::fmt::{Debug, DebugStruct, Formatter};
    /// # use rand::RngCore;
    /// # use rayna_engine::material::Material;
    /// # use rayna_engine::shared::intersect::Intersection;
    /// # use rayna_engine::shared::math::reflect;
    /// # use rayna_engine::shared::ray::Ray;
    /// # use rayna_engine::shared::{rng, RtRequirement};
    /// # use rayna_shared::def::types::{Pixel, Vector3};
    /// #
    /// # #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    /// pub struct Test;
    /// #     /// #
    /// impl Material for Test {
    ///     fn scatter(&self, ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Vector3 {
    ///         let diffuse = false;
    ///         // Diffuse => random
    ///         if diffuse {
    ///             rng::vector_in_unit_hemisphere(rng, intersection.normal)
    ///         }
    ///         // Reflective => reflect off normal
    ///         else {
    ///             let d = ray.dir();
    ///             let n = intersection.normal;
    ///             let r = reflect(d, n);
    ///             r
    ///         }
    ///     }
    /// #   fn calculate_colour(&self, ray: &Ray, intersection: &Intersection, future_ray: &Ray, future_col: &Pixel) -> Pixel { unimplemented!() }
    /// }
    /// ```
    fn scatter(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        rng: &mut dyn RngCore,
    ) -> Option<Vector3>;

    /// This function does the lighting calculations, based on the light from the future ray
    ///
    /// # Arguments
    ///
    /// * `intersection`: Information such as where the ray hit, surface normals, etc
    /// * `future_ray`: The ray for the future bounce that was made
    /// * `future_col`: The colour information for the future bounce that was made
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::fmt::{Debug, DebugStruct, Formatter};
    /// # use rand::RngCore;
    /// # use rayna_engine::material::Material;
    /// # use rayna_engine::shared::intersect::Intersection;
    /// # use rayna_engine::shared::math::reflect;
    /// # use rayna_engine::shared::ray::Ray;
    /// # use rayna_engine::shared::{rng, RtRequirement};
    /// # use rayna_shared::def::types::{Pixel, Vector3};
    /// #
    /// # #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    /// pub struct Test;
    /// #     /// #
    /// impl Material for Test {
    /// #   fn scatter(&self, ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Vector3 { unimplemented!() }
    ///     fn calculate_colour(&self, ray: &Ray, intersection: &Intersection, future_ray: &Ray, future_col: &Pixel) -> Pixel {
    ///         // Pure reflection
    ///         return *future_col;
    ///         // Pure absorbtion
    ///         return Pixel::from([0. ; 3]);
    ///     }
    /// }
    /// ```
    fn calculate_colour(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        future_ray: &Ray,
        future_col: &Pixel,
    ) -> Pixel;
}

/// An optimised implementation of [Material].
///
/// By using an enum, we can replace dynamic-dispatch with static dispatch.
/// Just in case we do require dynamic dispatch for some reason, there is a
/// [crate::material::MaterialType::DynamicMaterial] variant, which wraps a generic material in an [std::sync::Arc]
#[enum_dispatch(Material)]
#[derive(Clone, Debug)]
pub enum MaterialType {
    LambertianMaterial,
    MetalMaterial,
    DielectricMaterial,
    DynamicMaterial,
}
