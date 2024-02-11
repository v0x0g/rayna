use crate::core::types::{Number, Point2, Point3, Vector3};
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use core::ops::*;
use num_traits::Zero;
use rand_core::RngCore;
use std::arch::x86_64::{_mm256_mul_pd, _mm256_shuffle_pd, _mm256_sub_pd, _MM_SHUFFLE};
use std::fmt::Debug;
use std::simd::Simd;
use std::simd::{prelude::*, simd_swizzle};

#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    /// The three corner vertices of the triangle
    vertices: [Simd<Number, 4>; 3],
    /// The corresponding normal vectors at the vertices
    normals: [Vector3; 3],
    aabb: Aabb,
}

impl Triangle {
    pub fn new(vertices: impl Into<[Point3; 3]>, normals: impl Into<[Vector3; 3]>) -> Self {
        let (vertices, normals) = (vertices.into(), normals.into());

        let [a, b, c] = vertices;
        assert!(a != b && b != c && c != a, "triangles cannot have duplicate vertices");
        assert!(
            normals.into_iter().all(Vector3::is_normalized),
            "normals must be normalised"
        );
        Self {
            vertices: vertices.map(to_simd),
            normals,
            aabb: Aabb::encompass_points(vertices),
        }
    }
}

// region Mesh Impl

impl MeshProperties for Triangle {
    fn centre(&self) -> Point3 {
        let [a, b, c] = self.vertices;
        // Average the points
        let [x, y, z, _] = ((a + b + c) / Simd::splat(3.)).to_array();
        (x, y, z).into()
    }
}

impl HasAabb for Triangle {
    fn aabb(&self) -> Option<&Aabb> { Some(&self.aabb) }
}

#[inline(always)]
fn simd_cross_prod(a: Simd<Number, 4>, b: Simd<Number, 4>) -> Simd<Number, 4> {
    let vec_result = to_simd(Vector3::cross(
        Vector3::new(a[0], a[1], a[2]),
        Vector3::new(b[0], b[1], b[2]),
    ));

    /*
    x: self.y * rhs.z - rhs.y * self.z,
    y: self.z * rhs.x - rhs.z * self.x,
    z: self.x * rhs.y - rhs.x * self.y,

    lhs.yzxw * rhs.zxyw - rhs.yzxw * lhs.zxyw
     */

    let lhs_1 = simd_swizzle!(a, [1, 2, 0, 3] /* YZXW */);
    let lhs_2 = simd_swizzle!(b, [2, 0, 1, 3] /* ZXYW */);
    let rhs_1 = simd_swizzle!(b, [1, 2, 0, 3] /* YZXW */);
    let rhs_2 = simd_swizzle!(a, [2, 0, 1, 3] /* ZXYW */);

    let simd_result = (lhs_1 * lhs_2) - (rhs_1 * rhs_2);

    return simd_result;
}

#[inline(always)]
fn simd_dot_prod(a: Simd<Number, 4>, b: Simd<Number, 4>) -> Number { Simd::mul(a, b).reduce_sum() }

#[inline(always)]
fn to_simd(n: impl Into<[Number; 3]>) -> Simd<Number, 4> {
    let arr = n.into();
    [arr[0], arr[1], arr[2], 0.0].into()
}

impl Mesh for Triangle {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        /*
        CREDITS:

        Title:  "Ray-Tracing: Rendering a Triangle (Möller-Trumbore algorithm)"
        Author: Scratchapixel
        Url:    <https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection.html>

        ADAPTED USING:
        Title:  "Optimized SIMD Cross-Product"
        Author: imallet (Ian Mallet)
        Url:    <https://geometrian.com/programming/tutorials/cross-product/index.php>
        */

        let [v0, v1, v2] = self.vertices;
        let rd = to_simd(ray.dir());
        let ro = to_simd(ray.pos());

        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        let p_vec = simd_cross_prod(rd, v0v2);
        let det = simd_dot_prod(v0v1, p_vec);

        // ray and triangle are parallel
        if det.is_zero() {
            return None;
        }

        let inv_det = 1. / det;

        let t_vec = ro - v0;
        let u = simd_dot_prod(t_vec, p_vec) * inv_det;
        if u < 0. || u > 1. {
            return None;
        }

        let q_vec = simd_cross_prod(t_vec, v0v1);
        let v = simd_dot_prod(rd, q_vec) * inv_det;
        if v < 0. || u + v > 1. {
            return None;
        }
        let t = simd_dot_prod(v0v2, q_vec) * inv_det;

        if !interval.contains(&t) {
            return None;
        }

        let pos_w = ray.at(t);
        let bary_coords = Vector3::new(1. - u - v, u, v);
        // If we can't normalize, the vertex normals must have all added to (close to) zero
        // Therefore they must be opposing. Current way of handling this is to skip the point
        let normal = Self::interpolate_normals(self.normals, bary_coords)?;

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

impl Triangle {
    /// Interpolates across the vertex normals for a given point in barycentric coordinates
    fn interpolate_normals(normals: [Vector3; 3], bary_coords: Vector3) -> Option<Vector3> {
        std::iter::zip(normals, bary_coords)
            .map(|(n, u)| n * u)
            .fold(Vector3::ZERO, Vector3::add)
            .try_normalize()
    }
}

// endregion Mesh Impl
