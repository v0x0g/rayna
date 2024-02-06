use crate::core::types::{Number, Point3, Vector3};
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use itertools::Itertools;
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
        assert!(
            normals.into_iter().all(Vector3::is_normalized),
            "normals must be normalised"
        );
        let edges = [b - a, c - b, a - c];
        // Important to choose `edges[0, 1]`, else won't work
        let n_raw = Vector3::cross(edges[0], edges[1]);
        let n = n_raw
            .try_normalize()
            .expect("couldn't normalise plane normal: cross(u, v) == 0");
        let d = -Vector3::dot(n, b.to_vector());
        // NOTE: using non-normalised plane normal here
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

        // Barycentric coordinates
        // TODO: I believe that barycentric coordinates are currently slightly off, and are
        //  giving the distance to the adjacent vertex, not the opposing vertex
        let mut pos_b = Vector3::ZERO;
        for i in 0..3 {
            let vp = pos_w - self.vertices[i];
            let c = Vector3::cross(self.edges[i], vp);
            pos_b[i] = Vector3::dot(self.w, c);
            if pos_b[i] < 0. {
                return None;
            }
        }

        // If we can't normalize, the vertex normals must have all added to (close to) zero
        // Therefore they must be opposing. Current way of handling this is to skip the point
        let normal = Self::interpolate_normals(self.normals, pos_b)?;

        return Some(Intersection {
            pos_w,
            pos_l: pos_b.to_point(),
            normal,
            // ray_normal: normal * denominator.signum(),
            ray_normal: pos_b.normalize(),
            // if positive => ray and normal same dir => must be behind plane => backface
            front_face: denominator.is_sign_negative(),
            uv: [pos_b[1], pos_b[2]].into(),
            face: 0,
            dist: t,
        });
    }
}

impl Triangle {
    /// Interpolates across the vertex normals for a given point in barycentric coordinates
    fn interpolate_normals(normals: [Vector3; 3], mut coords: Vector3) -> Option<Vector3> {
        coords.as_array_mut().rotate_right(2);
        std::iter::zip(normals, coords)
            .map(|(n, u)| n * u)
            .fold(Vector3::ZERO, Vector3::add)
            .try_normalize()
    }
}
