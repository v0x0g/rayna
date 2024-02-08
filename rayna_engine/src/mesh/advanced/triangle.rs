use crate::core::types::{Number, Point2, Point3, Vector3};
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use num_traits::Zero;
use rand_core::RngCore;
use std::fmt::Debug;
use std::ops::Add;

#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    /// The three corner vertices of the triangle
    vertices: [Point3; 3],
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
            vertices,
            normals,
            aabb: Aabb::encompass_points(vertices),
        }
    }
}

// region Mesh Impl

impl MeshProperties for Triangle {
    fn centre(&self) -> Point3 {
        let [a, b, c] = self.vertices.map(Vector3::from_point);
        ((a + b + c) / 3.).to_point()
    }
}

impl HasAabb for Triangle {
    fn aabb(&self) -> Option<&Aabb> { Some(&self.aabb) }
}

impl Mesh for Triangle {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        // // Check if ray is parallel to plane
        // let denominator = Vector3::dot(self.n, ray.dir());
        // if denominator.is_zero() {
        //     return None;
        // }
        //
        // let t = -(Vector3::dot(self.n, ray.pos().to_vector()) + self.d) / denominator;
        //
        // if !interval.contains(&t) {
        //     return None;
        // }
        //
        // let pos_w = ray.at(t);
        //
        // // Barycentric coordinates
        // let mut pos_b = Vector3::ZERO;
        // for i in 0..3 {
        //     let vp = pos_w - self.vertices[i];
        //     let c = Vector3::cross(self.edges[i], vp);
        //     let b = Vector3::dot(self.w, c);
        //     if b < 0. {
        //         return None;
        //     }
        //     // For some reason, these coordinates are slightly off and give results for the
        //     // previous vertex (i.e. `pos_b[0]` is how close the point is to `vertex[2]`
        //     // So counteract that here by rotating the index.
        //     pos_b[(i + 2) % 3] = b;
        // }
        //
        // // If we can't normalize, the vertex normals must have all added to (close to) zero
        // // Therefore they must be opposing. Current way of handling this is to skip the point
        // let normal = Self::interpolate_normals(self.normals, pos_b)?;
        //
        // return Some(Intersection {
        //     pos_w,
        //     pos_l: pos_b.to_point(),
        //     normal,
        //     ray_normal: normal * denominator.signum(),
        //     // if positive => ray and normal same dir => must be behind plane => backface
        //     front_face: denominator.is_sign_positive(),
        //     uv: [pos_b[1], pos_b[2]].into(),
        //     side: 0,
        //     dist: t,
        // });

        let [v0, v1, v2] = self.vertices;

        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        let p_vec = Vector3::cross(ray.dir(), v0v2);
        let det = v0v1.dot(p_vec);

        // ray and triangle are parallel
        if det.is_zero() {
            return None;
        }

        let inv_det = 1. / det;

        let t_vec = ray.pos() - v0;
        let u = Vector3::dot(t_vec, p_vec) * inv_det;
        if u < 0. || u > 1. {
            return None;
        }

        let q_vec = Vector3::cross(t_vec, v0v1);
        let v = Vector3::dot(ray.dir(), q_vec) * inv_det;
        if v < 0. || u + v > 1. {
            return None;
        }
        let t = Vector3::dot(v0v2, q_vec) * inv_det;

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
