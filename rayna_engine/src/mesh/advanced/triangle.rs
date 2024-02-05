use crate::core::types::{Number, Point3, Vector3};
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
    /// The normal for the plane that the triangle lays upon
    n: Vector3,
    /// The vectors along the edges of the triangle
    /// Stored as `[v0->v1, v1->v2, v2->v0]`
    edges: [Vector3; 3],
    /// Part of the plane equation: `dot(normal, vertices[0])`
    d: Number,
    /// Part of the plane equation: `cross(u, v) / cross(u,v).length_squared()`. Used for UV projection
    w: Vector3,
    aabb: Aabb,
}

impl Triangle {
    pub fn new(vertices: impl Into<[Point3; 3]>, normals: impl Into<[Vector3; 3]>) -> Self {
        let (vertices, normals) = (vertices.into(), normals.into());

        let [a, b, c] = vertices;
        assert!(a != b && b != c && c != a, "triangles cannot have duplicate vertices");
        let edges @ [u, v, _] = [b - a, c - b, a - c];
        let n_raw = Vector3::cross(u, v);
        let n = n_raw
            .try_normalize()
            .expect("couldn't normalise plane normal: cross(u, v) == 0");
        let d = -Vector3::dot(n, b.to_vector());
        // NOTE: using non-normalised normal here
        let w = n_raw / n_raw.length_squared();
        Self {
            vertices,
            normals,
            n,
            d,
            w,
            edges,
            aabb: Aabb::encompass_points(vertices),
        }
    }
}

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
        // Check if ray is parallel to plane
        let denominator = Vector3::dot(self.n, ray.dir());
        if denominator.is_zero() {
            return None;
        }

        let t = -(Vector3::dot(self.n, ray.pos().to_vector()) + self.d) / denominator;

        if !interval.contains(&t) {
            return None;
        }

        let pos_w = ray.at(t);

        // let mut uv = [0.; 3];
        //
        // for i in 0..2 {
        //     let vp = pos_w - self.vertices[i];
        //     let c = Vector3::cross(self.edges[i], vp);
        //     uv[i] = Vector3::dot(self.w, c);
        //     if uv[i] < 0. {
        //         return None;
        //     }
        // }
        //
        // let pos_barycentric = Point3::from(uv);
        // // let normal = std::iter::zip(self.normals, uv)
        // //     .map(|(n, u)| n * u)
        // //     .fold(Vector3::ZERO, Vector3::add);
        // let normal = self.normals[0] * uv[0] + self.normals[1] * uv[1] + self.normals[2] * uv[2];

        let [v0, v1, v2] = self.vertices;
        let [e0, e1, e2] = self.edges;

        let vp0 = pos_w - v0;
        let c0 = Vector3::cross(e0, vp0);
        let uv0 = Vector3::dot(self.w, c0);
        if uv0 < 0. {
            return None;
        }

        let vp1 = pos_w - v1;
        let c1 = Vector3::cross(e1, vp1);
        let uv1 = Vector3::dot(self.w, c1);
        if uv1 < 0. {
            return None;
        }

        let vp2 = pos_w - v2;
        let c2 = Vector3::cross(e2, vp2);
        let uv2 = Vector3::dot(self.w, c2);
        if uv2 < 0. {
            return None;
        }

        let uvs = [uv0, uv1, uv2];
        let pos_barycentric = Point3::from(uvs);
        let normal = std::iter::zip(self.normals, uvs)
            .map(|(n, u)| n * u)
            .fold(Vector3::ZERO, Vector3::add);

        return Some(Intersection {
            pos_w,
            pos_l: pos_barycentric,
            normal,
            ray_normal: normal * denominator.signum(),
            // if positive => ray and normal same dir => must be behind plane => backface
            front_face: denominator.is_sign_negative(),
            // uv: [uv[1], uv[2]].into(),
            uv: [uv1, uv2].into(),
            face: 0,
            dist: t,
        });
    }
}
