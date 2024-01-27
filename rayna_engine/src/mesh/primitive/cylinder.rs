use crate::core::types::{Number, Point2, Point3, Vector3};
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use getset::CopyGetters;
use glamour::AngleConsts;
use rand_core::RngCore;

#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct CylinderMesh {
    centre: Point3,
    /// The first point along the line of the cylinder
    origin: Point3,
    /// The vector `p2 - p1`, that goes along the length of the cylinder, with a magnitude equal
    /// to the length of the cylinder. The normalised value of this is the surface normal for the end caps
    along: Vector3,
    /// The square magnitude of [along] (how long the cylinder is, squared)
    length_sqr: Number,
    /// How long the cylinder is
    length: Number,
    radius: Number,
    /// Two arbitrary "outward" directions that points from the centre of the cylinder to the surface.
    /// Aka, two arbitrary, orthogonal surface normals
    orthogonals: (Vector3, Vector3),
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
        let length_sqr = along.length_squared();
        let length = length_sqr.sqrt();
        let orthogonals = Vector3::any_orthonormal_pair(&(along / length));

        Self {
            origin: p1,
            radius,
            along,
            length_sqr,
            length,
            orthogonals,
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

        // Quadratic formula
        let discriminant = (b * b) - (c * a);
        let sqrt_d = if discriminant < 0. {
            return None;
        } else {
            discriminant.sqrt()
        };

        // Check both of the intersections along the ray. We only check the second (further) distance
        // if the ray can start *inside* the cylinder (IQ's code didn't have this originally)
        // This is **not** checking the front-face/back-face, but checking the entering/exiting intersections
        // (e.g. when ray inside volume, the entering intersect is behind, so we have to check exiting intersect too)
        let mut dist = (-b - sqrt_d) / a;
        if !bounds.contains(&dist) {
            dist = (-b + sqrt_d) / a;
        }
        if !bounds.contains(&dist) {
            return None;
        }

        let normal;
        let face;
        let uv;

        // Distance along the line segment (P1 -> P2) that the ray intersects
        // 0 means @ P1, `1` means @ P2 (it's normalised). Not sure why `/len_sqr` not `/len`
        let dist_along_norm = (baoc + (dist * bard)) / self.length_sqr;

        // Intersect with body, only if the intersection is along the length segment of the cylinder
        // This will only check the front-face of the cylinder (where normal faces towards ray origin)
        // The back-face will always be obscured by the end caps
        if dist_along_norm > 0. && dist_along_norm < 1. {
            // Position of the intersection we are checking, relative to cylinder origin
            let pos_rel = oc + (rd * dist);
            // Position along the cylinder, relative from the origin. Normalised against length
            let pos_along = self.along * dist_along_norm;
            // The position "around" the origin that the intersection is.
            // This is the position on the surface, from the origin, at zero distance along the length
            let rel_pos_outwards = pos_rel - pos_along;
            // Normalise the relative position, and we get our normal vector easy!
            normal = rel_pos_outwards / self.radius;
            // Use orthogonals so we have reference frame for calculating UV coords
            // Both are normalised so we can skip normalising them
            let theta = Vector3::dot(normal, self.orthogonals.1).acos();
            // Use `signum()` of dot with second orthogonal, so we can tell which side of `self.orthogonals.0` it was
            let theta_signed = theta * Vector3::dot(normal, self.orthogonals.0).signum();
            // Remap from `-pi..pi`to `0..1`
            let u = (theta_signed / Number::PI / 2.) + 0.5;
            let v = dist_along_norm;
            uv = Point2::new(u, v);

            face = 0;
        }
        // Intersection wasn't along the (front-facing) body section, so check the end caps.
        // See note above about back-faces.
        else {
            // Find the distances to either of the end caps
            // If `baoc` and `bard` have the same sign, the P2 cap is nearer
            let (dist_near, dist_far) = if baoc.is_sign_positive() == bard.is_sign_positive() {
                ((0.0 - baoc) / bard, (self.length_sqr - baoc) / bard)
            } else {
                ((self.length_sqr - baoc) / bard, (0.0 - baoc) / bard)
            };
            let dist_near = Number::min(dist_near, dist_far);
            let dist_far = Number::max(dist_near, dist_far);

            dist = {
                if bounds.contains(&dist_near) && Number::abs(b + (a * dist_near)) < sqrt_d {
                    dist_near
                } else {
                    // Closer one failed, try other
                    if bounds.contains(&dist_far) && Number::abs(b + (a * dist_far)) < sqrt_d {
                        dist_far
                    } else {
                        // Neither cap matched
                        return None;
                    }
                }
            };
            // `self.along.normalised()` is also the normal vector for the end caps
            normal = self.along / self.length * dist_along_norm.signum();
            face = if dist_along_norm.is_sign_negative() { 1 } else { 2 };

            // Position of the intersection we are checking, relative to cylinder origin
            let pos_rel = oc + (rd * dist);

            let u = (pos_rel / self.radius).dot(self.orthogonals.0) / 2. + 0.5;
            let v = (pos_rel / self.radius).dot(self.orthogonals.1) / 2. + 0.5;

            // TODO: Get back to cylinder and fix at a later date
            uv = Point2::new(u, v);
        }

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
