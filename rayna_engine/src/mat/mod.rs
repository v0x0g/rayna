use crate::shared::intersect::Intersection;
use crate::shared::RtRequirement;
use rayna_shared::def::types::Vector;

pub mod diffuse;

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
    fn scatter(&self, intersection: Intersection) -> Vector;
}
