use educe::Educe;
use getset::{CopyGetters, Getters};
use rand_core::RngCore;
use std::sync::Arc;

use crate::core::types::{Number, Point2, Vector3};
use crate::mesh::isosurface::SdfFunction;
use crate::mesh::Mesh;
use crate::shared::aabb::{Aabb, Bounded};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;

/// A mesh struct that is created by ray-marching for a given SDF.
#[derive(CopyGetters, Getters, Educe, Clone)]
#[educe(Debug)]
pub struct RaymarchedIsosurfaceMesh {
    #[educe(Debug(ignore))]
    #[get = "pub"]
    sdf: Arc<dyn SdfFunction>,
    #[get_copy = "pub"]
    max_iterations: usize,
    #[get_copy = "pub"]
    epsilon: Number,
}

// region Constructors

impl RaymarchedIsosurfaceMesh {
    pub const DEFAULT_EPSILON: Number = 1e-7;
    pub const DEFAULT_ITERATIONS: usize = 150;

    /// Creates a new mesh from the given isosurface, as defined by the **Signed-Distance Function** (**SDF**)
    ///
    /// # Arguments
    ///
    /// * `sdf`: The **SDF** that defines the surface for the mesh.
    /// This SDF will be evaluated in world-space coordinates
    pub fn new<SDF: SdfFunction + 'static>(sdf: SDF) -> Self {
        Self {
            sdf: Arc::new(sdf),
            epsilon: Self::DEFAULT_EPSILON,
            max_iterations: Self::DEFAULT_ITERATIONS,
        }
    }

    /// Creates a new mesh from the given isosurface, as defined by the **Signed-Distance Function** (**SDF**)
    ///
    /// # Arguments
    ///
    /// * `sdf`: The **SDF** that defines the surface for the mesh
    /// * `max_iterations`: The maximum number of ray-marching steps allowed for intersections
    /// * `epsilon`: The distance threshold at which a ray is considered to have intersected with the surface
    pub fn new_custom<SDF: SdfFunction + 'static>(sdf: SDF, max_iterations: usize, epsilon: Number) -> Self {
        Self {
            sdf: Arc::new(sdf),
            epsilon,
            max_iterations,
        }
    }
}

// endregion Constructors

// region Mesh Impl

impl Bounded for RaymarchedIsosurfaceMesh {
    fn aabb(&self) -> Aabb { Aabb::INFINITE }
}

impl Mesh for RaymarchedIsosurfaceMesh {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        // Start point at earliest pos on ray, or ray origin if unbounded
        let mut total_dist = interval.start.unwrap_or(0.0);
        let mut point = ray.at(total_dist);
        for i in 0..self.max_iterations {
            // Ray march towards surface
            let dist = (self.sdf)(point);
            // Always step forwards
            total_dist += dist.abs();
            // point += dir * step; // Causes compounding floating-point errors
            point = ray.at(total_dist);

            // Arbitrarily close to surface, counts as an intersection
            // Also needs to be in valid bounds
            if dist.abs() < self.epsilon && interval.contains(&total_dist) {
                // let point_pos = point + Vector3::splat(EPSILON);
                // let point_neg = point - Vector3::splat(EPSILON);
                let high = Vector3::new(
                    (self.sdf)((point.x + self.epsilon, point.y, point.z).into()),
                    (self.sdf)((point.x, point.y + self.epsilon, point.z).into()),
                    (self.sdf)((point.x, point.y, point.z + self.epsilon).into()),
                );
                let low = Vector3::new(
                    (self.sdf)((point.x - self.epsilon, point.y, point.z).into()),
                    (self.sdf)((point.x, point.y - self.epsilon, point.z).into()),
                    (self.sdf)((point.x, point.y, point.z - self.epsilon).into()),
                );
                let normal = (high - low).normalize();

                return Some(Intersection {
                    pos_w: point,
                    pos_l: point,
                    uv: Point2::ZERO,
                    dist: total_dist,
                    front_face: dist.is_sign_positive(),
                    side: i,
                    normal,
                    ray_normal: normal,
                });
            }
        }

        // Exceeded the limit
        return None;
    }
}

// endregion Mesh Impl
