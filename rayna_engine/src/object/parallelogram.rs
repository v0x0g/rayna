use crate::accel::aabb::Aabb;
use crate::material::MaterialType;
use crate::object::{Object, ObjectType};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use num_traits::Zero;
use rayna_shared::def::types::{Number, Point3, Vector3};

#[derive(Clone, Debug)]
pub struct ParallelogramBuilder {
    pub corner_origin: Point3,
    pub corner_upper: Point3,
    pub corner_right: Point3,
    pub material: MaterialType,
}

#[derive(Clone, Debug)]
pub struct ParallelogramObject {
    /// The point at which the parallelogram starts, aka the origin
    q: Point3,
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
    aabb: Aabb,
    material: MaterialType,
}

impl From<ParallelogramBuilder> for ParallelogramObject {
    fn from(p: ParallelogramBuilder) -> Self {
        let q = p.corner_origin;
        let u = p.corner_right - q;
        let v = p.corner_upper - q;
        let aabb = Aabb::new(p.corner_origin, q + u + v).min_padded(1e-10);
        let n_raw = Vector3::cross(u, v);
        let n = n_raw
            .try_normalize()
            .expect("couldn't normalise plane normal: cross(u, v) == 0");
        let d = n.dot(q.to_vector());
        // NOTE: using non-normalised normal here
        let w = n_raw / n_raw.length_squared();

        Self {
            q,
            u,
            v,
            n,
            d,
            w,
            aabb,
            material: p.material,
        }
    }
}

impl From<ParallelogramBuilder> for ObjectType {
    fn from(value: ParallelogramBuilder) -> Self {
        ParallelogramObject::from(value).into()
    }
}

impl Object for ParallelogramObject {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        // TODO: Extract 99% of this into a `planar` sub-struct

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
        if alpha < 0. || alpha > 1. || beta < 0. || beta > 1. {
            return None;
        }

        Some(Intersection {
            pos,
            material: self.material.clone(),
            dist: t,
            normal: self.n,
            front_face: true,
            ray_normal: -self.n * denominator.signum(),
        })
    }

    fn intersect_all<'a>(
        &'a self,
        ray: &'a Ray,
    ) -> Option<Box<dyn Iterator<Item = Intersection> + 'a>> {
        todo!()
    }

    fn bounding_box(&self) -> &Aabb {
        &self.aabb
    }
}
