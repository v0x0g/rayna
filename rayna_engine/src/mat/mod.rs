use crate::mat::diffuse::DiffuseMaterial;
use crate::shared::intersect::Intersection;
use crate::shared::RtRequirement;
use rayna_shared::def::types::Vector;
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
    fn scatter(&self, intersection: &Intersection) -> Vector {
        match self {
            Self::Diffuse(mat) => mat.scatter(intersection),
            Self::Other(mat) => mat.scatter(intersection),
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
    /// use rayna_shared::def::types::Vector;
    ///
    /// fn scatter(intersection: Intersection) -> Vector {
    ///     let diffuse = false;
    ///     // Diffuse => random
    ///     if diffuse {
    ///         rng::vector_in_unit_hemisphere(&mut thread_rng(), intersection.normal)
    ///     }
    ///     // Reflective => reflect off normal
    ///     else {
    ///         let d = intersection.ray.dir();
    ///         let n = intersection.normal;
    ///         let r = d - n * (2.0 * Vector::dot(d, n));
    ///         r
    ///     }
    /// }
    /// ```
    fn scatter(&self, intersection: &Intersection) -> Vector;
}
