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
    origin: Point3,
    /// The vector `p2 - p1`, that goes along the length of the cylinder, with a magnitude equal
    /// to the length of the cylinder.
    along: Vector3,
    /// Normalised version of [along]. This is the normal vector for the end caps
    along_norm: Vector3,
    /// The square magnitude of [along] (how long the cylinder is, squared)
    length_sqr: Number,
    /// How long the cylinder is
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
            origin: p1,
            radius,
            along,
            along_norm: along.normalize(),
            length_sqr: along.length_squared(),
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
        let rd = ray.dir();

        let oc = ray.pos() - self.origin;

        let bard = Vector3::dot(self.along, rd);
        let baoc = Vector3::dot(self.along, oc);

        let a = self.length_sqr - (bard * bard);
        let b = (self.length_sqr * Vector3::dot(oc, rd)) - (baoc * bard);
        let c =
            (self.length_sqr * Vector3::dot(oc, oc)) - (baoc * baoc) - (self.radius * self.radius * self.length_sqr);

        // let (k2, k1, k0) = (a,b,c);

        // Quadratic formula?
        let discriminant = (b * b) - (c * a);
        let sqrt_d = if discriminant < 0. {
            return None;
        } else {
            discriminant.sqrt()
        };

        let mut dist = (-b - sqrt_d) / a;

        let normal;
        let face;
        let uv;

        // Distance along the line segment (P1 -> P2) that the ray intersects
        // 0 means @ P1, `baba` means @ P2
        let dist_along_sqr = baoc + (dist * bard);

        // Intersected body
        if dist_along_sqr > 0. && dist_along_sqr < self.length_sqr {
            // Position of the intersection we are checking, relative to cylinder origin
            let pos_rel = oc + (rd * dist);
            // Position along the cylinder, relative from the origin. Normalised against length
            let norm_pos_along = (self.along * dist_along_sqr) / self.length_sqr;
            // The position "around" the origin that the intersection is.
            // This is the position on the surface, from the origin, at zero distance along the length
            let rel_pos_outwards = pos_rel - norm_pos_along;
            // Normalise the position, and we get our normal vector easy!
            normal = (rel_pos_outwards / self.radius).normalize();
            uv = Point2::new(0., dist_along_sqr.sqrt());

            face = 0;
        }
        // Intersected caps
        else {
            // Distance along the length, for whichever cap we are checking (the closer one)
            let cap_dist = if dist_along_sqr < 0. { 0. } else { self.length_sqr };
            // `dist` is distance along the ray that we intersect with the end caps
            dist = (cap_dist - baoc) / bard;
            if Number::abs(b + (a * dist)) >= sqrt_d {
                return None;
            }
            // `self.along_norm` is also the normal vector for the end caps
            normal = self.along_norm * dist_along_sqr.signum();
            face = 1;
            uv = Point2::new(dist, dist);
        }

        if !bounds.contains(&dist) {
            return None;
        }
        assert_ne!(dist, -1.);

        let pos_w = ray.at(dist);
        let pos_l = (pos_w - self.centre).into();
        let inside_sign = -Vector3::dot(rd, normal).signum();
        return Some(Intersection {
            pos_w,
            pos_l,
            normal,
            ray_normal: normal * inside_sign,
            front_face: inside_sign.is_sign_negative(),
            dist,
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
