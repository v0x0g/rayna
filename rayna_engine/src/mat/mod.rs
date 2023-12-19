use crate::mat::lambertian::LambertianMaterial;
use crate::mat::metal::MetalMaterial;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use rayna_shared::def::types::{Pixel, Vector3};
use std::sync::Arc;

mod dielectric;
pub mod lambertian;
pub mod metal;

/// An optimised implementation of [Material].
///
/// By using an enum, we can replace dynamic-dispatch with static dispatch.
/// Just in case we do require dynamic dispatch for some reason, there is a
/// [MaterialType::Other] variant, which wraps a generic material in an [Arc]
#[derive(Clone, Debug)]
pub enum MaterialType {
    Lambertian(LambertianMaterial),
    Metal(MetalMaterial),
    Other(Arc<dyn Material>),
}

impl RtRequirement for MaterialType {}

impl Material for MaterialType {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<Vector3> {
        match self {
            Self::Lambertian(mat) => mat.scatter(ray, intersection),
            Self::Metal(mat) => mat.scatter(ray, intersection),
            Self::Other(mat) => mat.scatter(ray, intersection),
        }
    }

    fn calculate_colour(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        future_ray: &Ray,
        future_col: &Pixel,
    ) -> Pixel {
        match self {
            Self::Lambertian(mat) => {
                mat.calculate_colour(ray, intersection, future_ray, future_col)
            }
            Self::Metal(mat) => mat.calculate_colour(ray, intersection, future_ray, future_col),
            Self::Other(mat) => mat.calculate_colour(ray, intersection, future_ray, future_col),
        }
    }
}

/// The trait that defines what properties a material has
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
    /// # use rand::thread_rng;
    /// # use rayna_engine::mat::Material;
    /// # use rayna_engine::shared::intersect::Intersection;
    /// # use rayna_engine::shared::math::reflect;
    /// # use rayna_engine::shared::ray::Ray;
    /// # use rayna_engine::shared::{rng, RtRequirement};
    /// # use rayna_shared::def::types::{Pixel, Vector3};
    /// #
    /// # #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    /// pub struct Test;
    /// #
    /// # impl RtRequirement for Test {}
    /// #
    /// impl Material for Test{
    ///     fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Vector3 {
    ///         let diffuse = false;
    ///         // Diffuse => random
    ///         if diffuse {
    ///             rng::vector_in_unit_hemisphere(&mut thread_rng(), intersection.normal)
    ///         }
    ///         // Reflective => reflect off normal
    ///         else {
    ///             let d = ray.dir();
    ///             let n = intersection.normal;
    ///             let r = reflect(d, n);
    ///             r
    ///         }
    ///     }
    /// #   fn calculate_colour(&self, ray: &Ray, intersection: &Intersection, future_ray: &Ray, future_col: &Pixel) -> Pixel { todo!() }
    /// }
    /// ```
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<Vector3>;

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
    /// # use rand::thread_rng;
    /// # use rayna_engine::mat::Material;
    /// # use rayna_engine::shared::intersect::Intersection;
    /// # use rayna_engine::shared::math::reflect;
    /// # use rayna_engine::shared::ray::Ray;
    /// # use rayna_engine::shared::{rng, RtRequirement};
    /// # use rayna_shared::def::types::{Pixel, Vector3};
    /// #
    /// # #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    /// pub struct Test;
    /// #
    /// # impl RtRequirement for Test {}
    /// #
    /// impl Material for Test{
    /// #   fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Vector3 { todo!() }
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
