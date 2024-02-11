use crate::core::types::{Number, Point2, Point3, Vector3};
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use core::ops::*;
use itertools::Itertools;
use rand_core::RngCore;
use std::fmt::Debug;
use std::simd::{prelude::*, SimdElement};
use std::simd::{LaneCount, Simd, SupportedLaneCount};

#[derive(Copy, Clone, Debug)]
pub struct BatchTriangle<const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    // The points are stored as [[x1, x2, ..., xn], [y1, y2, ..., yn], [z1, z2, ..., zn]]
    // So we have all the packed X values, then packed Y values, then packed Z values
    v0: [Simd<Number, N>; 3],
    v1: [Simd<Number, N>; 3],
    v2: [Simd<Number, N>; 3],
    // Normals don't use SIMD acceleration yet, so just store as a plain array
    normals: [[Vector3; 3]; N],
    aabb: Aabb,
}

impl<const N: usize> BatchTriangle<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    pub fn new<Vert: Into<Point3>, Norm: Into<Vector3>>(vertices: [[Vert; 3]; N], normals: [[Norm; 3]; N]) -> Self {
        // Convert the vertices and normals into actual points and vectors
        let vertices: [[Point3; 3]; N] = vertices.map(|vs| vs.map(Vert::into));
        let normals: [[Vector3; 3]; N] = normals.map(|ns| ns.map(Norm::into));

        // let [a, b, c] = vertices;
        // assert!(a != b && b != c && c != a, "triangles cannot have duplicate vertices");
        // assert!(
        //     normals.into_iter().all(Vector3::is_normalized),
        //     "normals must be normalised"
        // );
        // Unpack the vertices, so we have arrays for all the 0'th vertices for all triangles, etc
        let v0_aos: [Point3; N] = vertices.map(|v| v[0]);
        let v1_aos: [Point3; N] = vertices.map(|v| v[1]);
        let v2_aos: [Point3; N] = vertices.map(|v| v[2]);
        // Now transpose, so we do all the `X` components, then `Y` components
        // We have the numbers interleaved as AOS,
        // and we want to unpack it to SOA
        fn aos_to_soa<const N: usize>(v_n: [Point3; N]) -> [Simd<Number, N>; 3]
        where
            LaneCount<N>: SupportedLaneCount,
        {
            let x: Simd<Number, N> = v_n.map(|v| v.x).into();
            let y: Simd<Number, N> = v_n.map(|v| v.y).into();
            let z: Simd<Number, N> = v_n.map(|v| v.z).into();
            [x, y, z]
        }
        let v0 = aos_to_soa(v0_aos);
        let v1 = aos_to_soa(v1_aos);
        let v2 = aos_to_soa(v2_aos);
        Self {
            v0,
            v1,
            v2,
            normals,
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
        let [[x1, y1, z1], [x2, y2, z2], [x3, y3, z3]] = [self.v0, self.v1, self.v2];

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

        let rd = [
            Simd::splat(ray.dir().x),
            Simd::splat(ray.dir().y),
            Simd::splat(ray.dir().z),
        ];
        let ro = [
            Simd::splat(ray.pos().x),
            Simd::splat(ray.pos().y),
            Simd::splat(ray.pos().z),
        ];

        let v0v1 = Self::simd_multi_sub(self.v1, self.v0); // v1 - v0
        let v0v2 = Self::simd_multi_sub(self.v2, self.v0); // v2 - v0
        let p_vec = Self::simd_multi_cross(rd, v0v2); // rd X v0v2
        let det = Self::simd_multi_dot(v0v1, p_vec); // v0v1 * p_vec

        let mut failed_mask = Mask::<<Number as SimdElement>::Mask, N>::from_array([false; N]);

        // Check if ray and triangle are parallel
        failed_mask |= Simd::simd_eq(det, Self::ZERO);

        let inv_det = Simd::splat(1.) / det;

        let t_vec = Self::simd_multi_sub(ro, self.v0);
        let u = Self::simd_multi_dot(t_vec, p_vec) * inv_det;
        // Validate `u` in the range `0..=1`
        failed_mask |= Simd::simd_lt(u, Self::ZERO) | Simd::simd_gt(u, Self::ONE);

        let q_vec = Self::simd_multi_cross(t_vec, v0v1);
        let v = Self::simd_multi_dot(rd, q_vec) * inv_det;
        // Validate `v` in the range `0..=1`
        failed_mask |= Simd::simd_lt(v, Self::ZERO) | Simd::simd_gt(v, Self::ONE);

        let t = Self::simd_multi_dot(v0v2, q_vec) * inv_det;
        // Validate `t` is in the given interval

        // let interval_min = Simd::splat(interval.start.unwrap_or(Number::NAN));
        // let interval_max = Simd::splat(interval.start.unwrap_or(Number::NAN));
        // // Set intervals to `NaN` if there is no bound. This way we can use the fact that `NaN`
        // // cannot be compared (always is false), to check if it is out of bounds

        if let Some(start) = interval.start {
            // Lower than start bound
            failed_mask |= Simd::simd_lt(t, Simd::splat(start))
        }
        if let Some(end) = interval.end {
            // Greater than end bound
            failed_mask |= Simd::simd_gt(t, Simd::splat(end))
        }

        // Choose smallest `t`
        // Replace any failed values with `Number::POS_INF`
        // This way when we find the min, the failed ones are all at the end
        // Can't use `filter()` because then we'd get the position after filtering
        let mr_nice_t = failed_mask.select(Self::POS_INF, t);
        let tri_idx = mr_nice_t.to_array().into_iter().position_min_by(Number::total_cmp)?;
        // If all values failed, this will be `Number::POS_INF`
        if failed_mask.test(tri_idx) {
            return None;
        }

        // let test: [_; N] = std::array::from_fn(|i| if failed_mask.test(i) { None } else { Some(t[i]) });
        // dbg!(t);
        // dbg!(failed_mask);
        // dbg!(test, tri_idx);

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
    const ZERO: Simd<Number, N> = Simd::from_array([0.; N]);
    const ONE: Simd<Number, N> = Simd::from_array([1.; N]);
    const POS_INF: Simd<Number, N> = Simd::from_array([Number::INFINITY; N]);

    #[inline(always)]
    fn simd_multi_cross(a: [Simd<Number, N>; 3], b: [Simd<Number, N>; 3]) -> [Simd<Number, N>; 3] {
        [
            (a[1] * b[2]) - (b[1] * a[2]),
            (a[2] * b[0]) - (b[2] * a[0]),
            (a[0] * b[1]) - (b[0] * a[1]),
        ]
    }

    #[inline(always)]
    fn simd_multi_dot(a: [Simd<Number, N>; 3], b: [Simd<Number, N>; 3]) -> Simd<Number, N> {
        std::iter::zip(a, b).map(|(u, v)| u * v).sum()
        // (a[0] * b[0]) + (a[1] * b[1]) + (a[2] * b[2])
    }

    #[inline(always)]
    fn simd_multi_sub(a: [Simd<Number, N>; 3], b: [Simd<Number, N>; 3]) -> [Simd<Number, N>; 3] {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    }

    /// Interpolates across the vertex normals for a given point in barycentric coordinates
    fn interpolate_normals(normals: [Vector3; 3], bary_coords: Vector3) -> Option<Vector3> {
        std::iter::zip(normals, bary_coords)
            .map(|(n, u)| n * u)
            .fold(Vector3::ZERO, Vector3::add)
            .try_normalize()
    }
}
// endregion Helper
