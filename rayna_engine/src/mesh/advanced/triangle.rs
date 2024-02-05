use crate::core::types::{Number, Point3, Vector3};
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use num_traits::Zero;
use rand_core::RngCore;
use std::fmt::Debug;

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
    fn aabb(&self) -> Option<&Aabb> { Some(&Aabb::encompass_points(self.vertices)) }
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
        // TODO: pos_l is pos in barycentric coordinates
        // let pos_l = pos_w - self.vertices[1];

        let [v0, v1, v2] = self.vertices;
        let [e0, e1, e2] = self.edges;

        let vp0 = pos_w - v0;
        let c0 = e0.crossProduct(vp0);
        let uv0 = Vector3::dot(self.w, c0);
        if uv0 < 0. {
            return None;
        }

        let vp1 = pos_w - v1;
        let c1 = e1.crossProduct(vp1);
        let uv1 = Vector3::dot(self.w, c1);
        if uv1 < 0. {
            return None;
        }

        let vp2 = pos_w - v2;
        let c2 = e2.crossProduct(vp2);
        let uv2 = Vector3::dot(self.w, c2);
        if uv2 < 0. {
            return None;
        }

        let pos_barycentric = [uv0, uv1, uv2].into();
        let normal = self.normals * pos_barycentric;

        return Some(Intersection {
            pos_w,
            pos_l: pos_barycentric.to_point(),
            normal,
            ray_normal: normal * -denominator.signum(),
            front_face: denominator.is_sign_negative(),
            uv: [uv1, uv2].into(),
            face: 0,
            dist: t,
        });
    }
}
