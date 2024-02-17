use crate::core::types::{Number, Point2, Point3, Vector3};
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use crate::shared::simd_math::{SimdConstants, SimdVector};
use core::ops::*;
use itertools::Itertools;
use rand_core::RngCore;
use std::fmt::Debug;
use std::simd::prelude::*;
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

#[derive(Copy, Clone, Debug)]
pub struct BatchTriangle<const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    // The points are stored as [[x1, x2, ..., xn], [y1, y2, ..., yn], [z1, z2, ..., zn]]
    // So we have all the packed X values, then packed Y values, then packed Z values
    v0: SimdVector<N, 3>,
    v1: SimdVector<N, 3>,
    v2: SimdVector<N, 3>,
    // Normals don't use SIMD acceleration yet, so just store as a plain array
    normals: [[Vector3; 3]; N],
    /// Bitmask for which elements are disabled
    disabled_mask: Mask<<Number as SimdElement>::Mask, N>,
    aabb: Aabb,
}

impl<const N: usize> BatchTriangle<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    pub fn new(vertices: &[[Point3; 3]], normals: &[[Vector3; 3]]) -> Self {
        assert!(vertices.len() <= N, "too many vertices");
        assert!(normals.len() <= N, "too many normals");
        assert_eq!(normals.len(), vertices.len(), "must have one normal per vertex");

        // let [a, b, c] = vertices;
        // assert!(a != b && b != c && c != a, "triangles cannot have duplicate vertices");
        // assert!(
        //     normals.into_iter().all(Vector3::is_normalized),
        //     "normals must be normalised"
        // );

        // Create arrays from slices, padding the unused space with NaN vectors
        let disabled_mask = {
            let mut m = [true; N];
            (&mut m[..vertices.len()]).fill(false);
            Mask::from_array(m)
        };
        let vertices: [[Point3; 3]; N] = slice_to_array(vertices, [Point3::NAN; 3]);
        let normals: [[Vector3; 3]; N] = slice_to_array(normals, [Vector3::NAN; 3]);

        // Unpack the vertices, so we have arrays for all the 0'th vertices for all triangles, etc
        let v0_aos: [Point3; N] = vertices.map(|v| v[0]);
        let v1_aos: [Point3; N] = vertices.map(|v| v[1]);
        let v2_aos: [Point3; N] = vertices.map(|v| v[2]);
        // Now transpose, so we do all the `X` components, then `Y` components
        // We have the numbers interleaved as AOS,
        // and we want to unpack it to SOA

        /// Helper function that converts an AoS (**Array of Structs**) of vertices to
        /// an SoA (**Struct of Arrays**) of vertices.
        fn aos_to_soa<const N: usize>(v_n: [Point3; N]) -> SimdVector<N, 3>
        where
            LaneCount<N>: SupportedLaneCount,
        {
            let x: Simd<Number, N> = v_n.map(|v| v.x).into();
            let y: Simd<Number, N> = v_n.map(|v| v.y).into();
            let z: Simd<Number, N> = v_n.map(|v| v.z).into();
            SimdVector([x, y, z])
        }

        fn slice_to_array<const N: usize, T: Copy>(slice: &[T], default: T) -> [T; N] {
            // array::from_fn(|v_idx| slice.get(v_idx).cloned().unwrap_or_default());
            let mut v = [default; N];
            (&mut v[..slice.len()]).copy_from_slice(slice);
            v
        }

        let v0 = aos_to_soa(v0_aos);
        let v1 = aos_to_soa(v1_aos);
        let v2 = aos_to_soa(v2_aos);

        Self {
            v0,
            v1,
            v2,
            normals,
            disabled_mask,
            aabb: Aabb::encompass_points(vertices.flatten()),
        }
    }
}

// region Mesh Impl

impl<const N: usize> MeshProperties for BatchTriangle<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    fn centre(&self) -> Point3 {
        // TODO: BatchTriangle centre
        let [[x1, y1, z1], [x2, y2, z2], [x3, y3, z3]] = [self.v0.0, self.v1.0, self.v2.0];

        let xs = (x1 + x2 + x3) / Simd::splat(3.);
        let ys = (y1 + y2 + y3) / Simd::splat(3.);
        let zs = (z1 + z2 + z3) / Simd::splat(3.);

        let x = xs.reduce_sum() / N as Number;
        let y = ys.reduce_sum() / N as Number;
        let z = zs.reduce_sum() / N as Number;

        (x, y, z).into()
    }
}

impl<const N: usize> HasAabb for BatchTriangle<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    fn aabb(&self) -> Option<&Aabb> { Some(&self.aabb) }
}

impl<const N: usize> Mesh for BatchTriangle<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        /*
            CREDITS:

            Title:  "Ray-Tracing: Rendering a Triangle (Möller-Trumbore algorithm)"
            Author: Scratchapixel
            Url:    <https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection.html>

            ADAPTED USING:

            Author: StudenteChamp
            Url: <https://stackoverflow.com/a/45626274>
        */

        let rd = SimdVector([
            Simd::splat(ray.dir().x),
            Simd::splat(ray.dir().y),
            Simd::splat(ray.dir().z),
        ]);
        let ro = SimdVector([
            Simd::splat(ray.pos().x),
            Simd::splat(ray.pos().y),
            Simd::splat(ray.pos().z),
        ]);

        let v0v1 = self.v1 - self.v0; // v1 - v0
        let v0v2 = self.v2 - self.v0; // v2 - v0
        let p_vec = SimdVector::cross(rd, v0v2); // rd X v0v2
        let det = SimdVector::dot(v0v1, p_vec); // v0v1 * p_vec

        let mut failed_mask = Mask::from_array([false; N]);

        // Check if ray and triangle are parallel
        failed_mask |= Simd::simd_eq(det, SimdConstants::ZERO);

        let inv_det = Simd::splat(1.) / det;

        let t_vec = ro - self.v0;
        let u = SimdVector::dot(t_vec, p_vec) * inv_det;
        // Validate `u` in the range `0..=1`
        failed_mask |= Simd::simd_lt(u, SimdConstants::ZERO) | Simd::simd_gt(u, SimdConstants::ONE);

        let q_vec = SimdVector::cross(t_vec, v0v1);
        let v = SimdVector::dot(rd, q_vec) * inv_det;
        // Validate `v` in the range `0..=1`
        failed_mask |= Simd::simd_lt(v, SimdConstants::ZERO) | Simd::simd_gt(v, SimdConstants::ONE);

        let t = SimdVector::dot(v0v2, q_vec) * inv_det;
        // Validate `t` is in the given interval

        // Set intervals to `NaN` if there is no bound. This way we can use the fact that `NaN`
        // cannot be compared (always is false), to check if it is out of (maybe missing) bounds
        // It's a bit faster than a branching `if let Some(...) = interval.xxx`
        let interval_min = Simd::splat(interval.start.unwrap_or(Number::NAN));
        let interval_max = Simd::splat(interval.end.unwrap_or(Number::NAN));
        failed_mask |= Simd::simd_lt(t, interval_min);
        failed_mask |= Simd::simd_gt(t, interval_max);

        failed_mask |= self.disabled_mask;

        // Choose smallest `t`
        // Replace any failed values with `Number::POS_INF`
        // This way when we find the min, the failed ones are all at the end
        // Can't use `filter()` because then we'd get the position after filtering
        if failed_mask.all() {
            return None;
        }
        let mr_nice_t = failed_mask.select(SimdConstants::POS_INFINITY, t);
        let tri_idx = mr_nice_t
            .to_array()
            .into_iter()
            .position_min_by(Number::total_cmp)
            .expect("already checked at least one triangle didn't fail");

        let (t, u, v, norms, det) = (t[tri_idx], u[tri_idx], v[tri_idx], self.normals[tri_idx], det[tri_idx]);

        let pos_w = ray.at(t);
        let bary_coords = Vector3::new(1. - u - v, u, v);
        // If we can't normalize, the vertex normals must have all added to (close to) zero
        // Therefore they must be opposing. Current way of handling this is to skip the point
        let normal = Self::interpolate_normals(norms, bary_coords)?;

        Some(Intersection {
            pos_w,
            pos_l: bary_coords.to_point(),
            front_face: det.is_sign_negative(),
            dist: t,
            uv: Point2::new(u, v),
            side: 0,
            ray_normal: normal * -det.signum(),
            normal,
        })
    }
}
// endregion Mesh Impl

// region Helper
impl<const N: usize> BatchTriangle<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    /// Interpolates across the vertex normals for a given point in barycentric coordinates
    fn interpolate_normals(normals: [Vector3; 3], bary_coords: Vector3) -> Option<Vector3> {
        std::iter::zip(normals, bary_coords)
            .map(|(n, u)| n * u)
            .fold(Vector3::ZERO, Vector3::add)
            .try_normalize()
    }
}
// endregion Helper
