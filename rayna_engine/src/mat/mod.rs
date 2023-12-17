use crate::mat::diffuse::DiffuseMaterial;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::RtRequirement;
use rayna_shared::def::types::{Pixel, Vector3};
use std::sync::Arc;

pub mod diffuse;

/// An optimised implementation of [Material].
///
/// By using an enum, we can replace dynamic-dispatch with static dispatch.
/// Just in case we do require dynamic dispatch for some reason, there is a
/// [MaterialType::Other] variant, which wraps a generic material in an [Arc]
#[derive(Clone, Debug)]
pub enum MaterialType {
    Diffuse(DiffuseMaterial),
    Other(Arc<dyn Material>),
}

impl RtRequirement for MaterialType {}

impl Material for MaterialType {
    fn scatter(&self, intersection: &Intersection) -> Option<Vector3> {
        match self {
            Self::Diffuse(mat) => mat.scatter(intersection),
            Self::Other(mat) => mat.scatter(intersection),
        }
    }

    fn calculate_colour(
        &self,
        intersection: &Intersection,
        future_ray: Ray,
        future_col: Pixel,
    ) -> Pixel {
        match self {
            Self::Diffuse(mat) => mat.calculate_colour(intersection, future_ray, future_col),
            Self::Other(mat) => mat.calculate_colour(intersection, future_ray, future_col),
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
    /// * `intersection`: Information about the intersection we are calculating the scatter for
    ///     Includes surface normals, etc
    ///
    /// # Examples
    ///
    /// ```
    /// use rand::thread_rng;
    /// use rayna_engine::shared::intersect::Intersection;
    /// use rayna_engine::shared::rng;
    /// use rayna_shared::def::types::Vector3;
    ///
    /// fn scatter(intersection: Intersection) -> Vector3 {
    ///     let diffuse = false;
    ///     // Diffuse => random
    ///     if diffuse {
    ///         rng::vector_in_unit_hemisphere(&mut thread_rng(), intersection.normal)
    ///     }
    ///     // Reflective => reflect off normal
    ///     else {
    ///         let d = intersection.ray.dir();
    ///         let n = intersection.normal;
    ///         let r = d - n * (2.0 * Vector3::dot(d, n));
    ///         r
    ///     }
    /// }
    /// ```
    fn scatter(&self, intersection: &Intersection) -> Option<Vector3>;

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
    /// use rayna_engine::shared::intersect::Intersection;
    /// use rayna_engine::shared::ray::Ray;
    /// use rayna_shared::def::types::Pixel;
    ///
    /// fn calculate_colour(intersection: &Intersection, future_ray: Ray, future_col: Pixel) -> Pixel {
    ///     // Pure reflection
    ///     return future_col;
    ///     // Pure absorbtion
    ///     return Pixel::from([0. ; 3]);
    /// }
    /// ```
    fn calculate_colour(
        &self,
        intersection: &Intersection,
        future_ray: Ray,
        future_col: Pixel,
    ) -> Pixel;
}
