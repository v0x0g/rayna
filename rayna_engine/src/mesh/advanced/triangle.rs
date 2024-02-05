use crate::core::types::{Number, Point3, Vector3};
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use rand_core::RngCore;
use std::fmt::Debug;
use num_traits::Zero;

#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    /// The three corner vertices of the triangle
    vertices: [Point3; 3],
    /// The corresponding normal vectors at the vertices
    normals: [Vector3; 3],
    /// The normal for the plane that the triangle lays upon
    n: Vector3,
    /// Normal vector, but not normalised
    n_denorm: Vector3,
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
        let (u, v) = (a - b, c - b);
        let n_raw = Vector3::cross(u, v);
        let n = n_raw
            .try_normalize()
            .expect("couldn't normalise plane normal: cross(u, v) == 0");
        let d = -Vector3::dot(n, b.to_vector());
        // NOTE: using non-normalised normal here
        let w = n_raw / n_raw.length_squared();
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

        let C; // vector perpendicular to triangle's plane

        let [v0, v1, v2] = self.vertices;
        let [e0, e1, e2] = self.edges;

        let e0 = v1 - v0;
        let vp0 = pos_w - v0;
        let c0 = e0.crossProduct(vp0);
        if (self.n.dot(c0) < 0.0) return false; // pos_w is on the right side

        let e1 = v2 - v1;
        let vp1 = pos_w - v1;
        C = e1.crossProduct(vp1);
        if ((u = N.dotProduct(C)) < 0)  return false; // pos_w is on the right side

        let e2 = v0 - v2;
        let vp2 = pos_w - v2;
        C = e2.crossProduct(vp2);
        if ((v = N.dotProduct(C)) < 0) return false; // pos_w is on the right side;
    }
}
