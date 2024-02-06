use crate::core::types::{Number, Point2, Point3, Vector3};
use crate::mesh::isosurface::SdfGeneratorFunction;
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use derivative::Derivative;
use getset::{CopyGetters, Getters};
use rand_core::RngCore;

/// A mesh struct that is created by ray-marching for a given SDF.
#[derive(CopyGetters, Getters, Derivative, Clone)]
#[derivative(Debug)]
pub struct RaymarchedIsosurfaceMesh {
    #[derivative(Debug = "ignore")]
    #[get = "pub"]
    sdf: Box<dyn SdfGeneratorFunction>,

    max_iterations: usize,
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
    pub fn new<F: SdfGeneratorFunction + 'static>(sdf: F) -> Self {
        Self {
            sdf: Box::new(sdf),
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
    pub fn new_custom<F: SdfGeneratorFunction + 'static>(sdf: F, max_iterations: usize, epsilon: Number) -> Self {
        Self {
            sdf: Box::new(sdf),
            epsilon,
            max_iterations,
        }
    }
}

// endregion Constructors

// region Mesh Impl

impl HasAabb for RaymarchedIsosurfaceMesh {
    fn aabb(&self) -> Option<&Aabb> { None }
}

impl MeshProperties for RaymarchedIsosurfaceMesh {
    fn centre(&self) -> Point3 { Point3::ZERO }
}

impl Mesh for RaymarchedIsosurfaceMesh {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        let epsilon = self.epsilon;

        // Start point at earliest pos on ray, or ray origin if unbounded
        let mut total_dist = interval.start.unwrap_or(0.0);
        let mut point = ray.at(total_dist);
        let mut i = 0;
        loop {
            // Ray march towards surface
            let dist = (self.sdf)(point);
            // Always step forwards
            total_dist += dist.abs();
            // point += dir * step; // Causes compounding floating-point errors
            point = ray.at(total_dist);

            // Arbitrarily close to surface, counts as an intersection
            // Also needs to be in valid bounds
            if dist.abs() < epsilon && interval.contains(&total_dist) {
                // let point_pos = point + Vector3::splat(EPSILON);
                // let point_neg = point - Vector3::splat(EPSILON);
                let p = point;
                let normal = Vector3::normalize(
                    [
                        (self.sdf)((p.x + epsilon, p.y, p.z).into()) - (self.sdf)((p.x - epsilon, p.y, p.z).into()),
                        (self.sdf)((p.x, p.y + epsilon, p.z).into()) - (self.sdf)((p.x, p.y - epsilon, p.z).into()),
                        (self.sdf)((p.x, p.y, p.z + epsilon).into()) - (self.sdf)((p.x, p.y, p.z - epsilon).into()),
                    ]
                    .into(),
                );

                return Some(Intersection {
                    pos_w: p,
                    pos_l: p,
                    uv: Point2::ZERO,
                    dist: total_dist,
                    front_face: dist.is_sign_positive(),
                    side: i,
                    normal,
                    ray_normal: normal,
                });
            }

            // Exceeded the limit
            if i > self.max_iterations {
                return None;
            }

            i += 1;
        }
    }
}

// endregion Mesh Impl
