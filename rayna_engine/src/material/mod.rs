//noinspection ALL
use self::{
    dielectric::DielectricMaterial, dynamic::DynamicMaterial, isotropic::IsotropicMaterial,
    lambertian::LambertianMaterial, light::LightMaterial, metal::MetalMaterial,
};
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use enum_dispatch::enum_dispatch;
use rand::RngCore;
use rayna_shared::def::types::{Pixel, Vector3};

pub mod dielectric;
pub mod dynamic;
pub mod isotropic;
pub mod lambertian;
pub mod light;
pub mod metal;

/// The trait that defines what properties a material has
#[enum_dispatch]
pub trait Material: RtRequirement {
    // TODO: For all materials that currently take a TextureInstance, make them take generic
    //  Also make `Derive(Copy)`

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
    /// #   fn emitted_light(&self, ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Pixel {
    /// #       unimplemented!("code example")
    /// #   }
    /// #
    /// #   fn reflected_light(&self, ray: &Ray, intersection: &Intersection, future_ray: &Ray, future_col: &Pixel, rng: &mut dyn RngCore) -> Pixel {
    /// #       unimplemented!("code example")
    /// #   }
    /// }
    /// ```
    fn scatter(&self, ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Option<Vector3>;

    /// This function calculates the amount of light that is emitted by the material
    ///
    /// # Notes
    /// This function will always be called, even if the material does not scatter (see [Material::scatter()])
    ///
    /// # Return Value
    /// Returns the light (colour) of emission for the given intersection and ray. The default implementation
    /// is to return black (`Pixel([0.; 3])`)
    #[allow(unused_variables)]
    fn emitted_light(&self, ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Pixel {
        const BLACK: Pixel = Pixel { 0: [0.; 3] };
        BLACK
    }

    /// This function calculates what light should be reflected, based off the future light/ray information
    ///
    /// # Arguments
    ///
    /// * `intersection`: Information such as where the ray hit, surface normals, etc
    /// * `future_ray`: The ray for the future bounce that was made
    /// * `future_col`: The colour information for the future bounce that was made
    ///
    /// # Notes
    ///
    /// This function will only be called if the material scattered (see [Material::scatter()]) a ray. If there was no scatter,
    /// then only [Material::emitted_light()] will be called
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
    /// #   fn emitted_light(&self, ray: &Ray, intersection: &Intersection, rng: &mut dyn RngCore) -> Pixel { unimplemented!() }
    ///     fn reflected_light(&self, ray: &Ray, intersection: &Intersection, future_ray: &Ray, future_col: &Pixel, rng: &mut dyn RngCore) -> Pixel {
    ///         // Pure reflection
    ///         return *future_col;
    ///         // Pure absorbtion
    ///         return Pixel::from([0. ; 3]);
    ///     }
    /// }
    /// ```
    fn reflected_light(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        future_ray: &Ray,
        future_col: &Pixel,
        rng: &mut dyn RngCore,
    ) -> Pixel;
}

/// An optimised implementation of [Material].
///
/// By using an enum, we can replace dynamic-dispatch with static dispatch.
/// Just in case we do require dynamic dispatch for some reason, there is a
/// [MaterialInstance::DynamicMaterial] variant, which wraps a generic material in an [std::sync::Arc]
///
/// # Using This Type
/// You generally don't want to instantiate this type directly using the variants (as the names and variants might change),
/// instead prefer to use the [Into::into()] or [From::from()] implementations.
///
/// If using it as a parameter or type argument in a library, constrain over `T:` [Material],
/// and only use `T = ` [MaterialInstance] at the highest level where possible
#[enum_dispatch(Material)]
#[derive(Clone, Debug)]
pub enum MaterialInstance {
    LambertianMaterial,
    MetalMaterial,
    DielectricMaterial,
    IsotropicMaterial,
    LightMaterial,
    DynamicMaterial,
}

impl Default for MaterialInstance {
    fn default() -> Self { LambertianMaterial::default().into() }
}
