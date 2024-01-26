use crate::core::types::{Number, Point2, Point3, Vector3};
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use getset::CopyGetters;
use rand_core::RngCore;

#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct CylinderMesh {
    centre: Point3,
    /// The first point along the line of the cylinder
    p1: Point3,
    /// The second point along the line of the cylinder
    p2: Point3,
    /// The vector `p2 - p1`, that goes along the length of the cylinder, with a magnitude equal
    /// to the length of the cylinder.
    along: Vector3,
    /// The magnitude of [along] (how long the cylinder is)
    length: Number,
    radius: Number,
    aabb: Aabb,
}

// region Constructors

impl CylinderMesh {
    pub fn new(p1: impl Into<Point3>, p2: impl Into<Point3>, radius: Number) -> Self {
        let (p1, p2) = (p1.into(), p2.into());
        let aabb = Aabb::new(
            Point3::min(p1, p2) - Vector3::splat(radius),
            Point3::max(p1, p2) + Vector3::splat(radius),
        );
        let centre = ((p1.to_vector() + p2.to_vector()) / 2.).to_point();
        let along = p2 - p1;

        Self {
            p1,
            p2,
            radius,
            along,
            length: along.length(),
            centre,
            aabb,
        }
    }
}

// endregion Constructors

// region Mesh Impl

impl Mesh for CylinderMesh {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        let (ro, rd) = (ray.pos(), ray.dir());
        let (p1, p2, rad) = (self.p1, self.p2, self.radius);

        // TODO: Optimise `ba`
        let ba = self.along;
        let oc = ro - p1;

        let baba = self.length;
        let bard = Vector3::dot(ba, rd);
        let baoc = Vector3::dot(ba, oc);

        let a = baba - (bard * bard);
        let b = (baba * Vector3::dot(oc, rd)) - (baoc * bard);
        let c = (baba * Vector3::dot(oc, oc)) - (baoc * baoc) - (rad * rad * baba);

        // let (k2, k1, k0) = (a,b,c);

        // Quadratic formula?
        let discriminant = (b * b) - (c * a);
        let sqrt_d = if discriminant < 0. {
            return None;
        } else {
            discriminant.sqrt()
        };

        let mut t = (-b - sqrt_d) / a;

        let normal;
        let face;

        // Distance along the line segment (P1 -> P2) that the ray intersects
        // 0 means @ P1, `baba` means @ P2
        let dist_along = baoc + (t * bard);

        // Intersected body
        if dist_along > 0. && dist_along < baba {
            normal = (((oc + (rd * t)) - ((ba * dist_along) / baba)) / rad).normalize();

            face = 0;
        }
        // Intersected caps
        else {
            t = ((if dist_along < 0. { 0. } else { baba }) - baoc) / bard;
            if Number::abs(b + (a * t)) >= sqrt_d {
                return None;
            }
            normal = (ba * dist_along.signum()).normalize();
            face = 1;
        }

        if !bounds.contains(&t) {
            return None;
        }
        assert_ne!(t, -1.);

        let pos_w = ray.at(t);
        let pos_l = (pos_w - self.centre).into();
        let inside_sign = -Vector3::dot(rd, normal).signum();
        let uv = Point2::ZERO; // TODO: Cylinder UV coords
        return Some(Intersection {
            pos_w,
            pos_l,
            normal,
            ray_normal: normal * inside_sign,
            front_face: inside_sign.is_sign_negative(),
            dist: t,
            uv,
            face,
        });
    }
}

impl HasAabb for CylinderMesh {
    fn aabb(&self) -> Option<&Aabb> { Some(&self.aabb) }
}

impl MeshProperties for CylinderMesh {
    fn centre(&self) -> Point3 { self.centre }
}

// endregion Mesh Impl
