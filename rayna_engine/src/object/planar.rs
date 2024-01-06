//! This module is not an object module per-se, but a helper module that provides abstractions for
//! planar types (such as planes, quads, triangles, etc)
//!
//! You should store an instance of [Planar] inside your object struct, and

use crate::material::MaterialType;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use num_traits::Zero;
use rayna_shared::def::types::{Number, Point3, Vector3};

/// The recommended amount of padding around AABB's for planar objects
pub const AABB_PADDING: Number = 1e-6;

#[derive(Copy, Clone, Debug)]
pub struct Planar {
    /// The vector for the `U` direction, typically the 'right' direction
    u: Vector3,
    /// The vector for the `V` direction, typically the 'up' direction
    v: Vector3,
    /// The normal vector for the plane, perpendicular to [u] and [v], and normalised
    n: Vector3,
    /// Part of the plane equation
    d: Number,
    /// Precalculated vector `n / dot(n, cross(u,v))` (using un-normalised `n`)
    w: Vector3,
}

// region Constructors

impl Planar {
    /// Creates a new planar struct from three points
    ///
    /// # Arguments
    ///
    /// * `q`: The origin point, treated as the UV coordinate `(0, 0)`
    /// * `a`: The first point on the plane. Traditionally this would be the "right" point
    /// * `b`: The second point on the plane. Traditionally this would be the "upper" point
    pub fn new_points(q: Point3, a: Point3, b: Point3) -> Self {
        let u = q - a;
        let v = q - b;

        let n_raw = Vector3::cross(u, v);
        let n = n_raw
            .try_normalize()
            .expect("couldn't normalise plane normal: cross(u, v) == 0");
        let d = n.dot(q.to_vector());
        // NOTE: using non-normalised normal here
        let w = n_raw / n_raw.length_squared();
        Self { u, v, n, d, w }
    }
}

// endregion

// region Intersection
impl Planar {
    /// Does a full ray-plane intersection check, returning the intersection if possible. If an intersection is not found,
    /// it means that the ray is perfectly parallel to the plane, or outside the given bounds.
    ///
    /// # Arguments
    ///
    /// * `ray`: The ray to intersect with
    /// * `bounds`: Bounds to restrict the range of valid distances
    /// * `material`: Material to be used for the [Intersection] in the case of an intersection
    /// * `validate_coords`: Callable to be used to validate whether the given point on the surface is considered valid.
    /// Arguments are `validate(u, v) -> point_is_valid`. Note that `u, v` will be with respect to the [Planar.u] and [Planar.v] values,
    /// so if creating a plane from three points, `u, v` will be equal to one *at those points*, as opposed to one unit in the direction of those points,
    /// meaning scaling those points will "enlarge" the resulting shape
    #[inline(always)]
    pub fn intersect_bounded(
        &self,
        ray: &Ray,
        bounds: &Bounds<Number>,
        material: &MaterialType,
        validate_coords: impl Fn(Number, Number) -> bool,
    ) -> Option<Intersection> {
        let denominator = Vector3::dot(self.n, ray.dir());

        // Ray is parallel to plane
        if denominator.is_zero() {
            return None;
        }

        let t = (self.d - Vector3::dot(self.n, ray.pos().to_vector())) / denominator;

        if !bounds.contains(&t) {
            return None;
        }

        let pos = ray.at(t);
        let pos_v = pos.to_vector();

        // We would normally project so the point is `P = Q + α*u + β*v`
        // But since the vectors `u, v` don't have to be orthogonal, have to account for that too
        let alpha = Vector3::dot(self.w, Vector3::cross(pos_v, self.v));
        let beta = Vector3::dot(self.w, Vector3::cross(self.u, pos_v));

        // Check in bounds for our segment of the plane
        if !validate_coords(alpha, beta) {
            return None;
        }

        Some(Intersection {
            pos,
            material: material.clone(),
            dist: t,
            normal: self.n,
            front_face: true,
            ray_normal: -self.n * denominator.signum(),
        })
    }
}
// endregion
