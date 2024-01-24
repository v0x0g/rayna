use crate::core::types::{Number, Point2, Point3, Vector3};
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use getset::CopyGetters;
use glamour::AngleConsts;
use rand_core::RngCore;

/// The actual instance of a sphere that can be rendered.
/// Has precomputed values and therefore cannot be mutated
#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct SphereMesh {
    pos: Point3,
    radius: Number,
    // TODO: is `radius_sqr` a perf improvement?
    radius_sqr: Number,
    aabb: Aabb,
}

// region Constructors

impl SphereMesh {
    pub fn new(pos: impl Into<Point3>, radius: impl Into<Number>) -> Self {
        let (pos, radius) = (pos.into(), radius.into());
        Self {
            pos,
            radius,
            radius_sqr: radius * radius,
            // Cube centred around self
            aabb: Aabb::new(pos - Vector3::splat(radius), pos + Vector3::splat(radius)),
        }
    }
}

// endregion Constructors

// region Mesh Impl

impl Mesh for SphereMesh {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        //Do some ray-sphere intersection math to find if the ray intersects
        let ray_pos = ray.pos();
        let ray_dir = ray.dir();
        let ray_rel_pos = ray_pos - self.pos;

        // Quadratic formula variables
        let a = ray_dir.length_squared();
        let half_b = Vector3::dot(ray_rel_pos, ray_dir);
        let c = ray_rel_pos.length_squared() - self.radius_sqr;
        let discriminant = (half_b * half_b) - (a * c);

        //No solutions to where ray intersects with sphere because of negative square root
        if discriminant < 0. {
            return None;
        };

        let sqrt_d = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        //This way we do a double check on both, prioritizing the less-positive root (as it's closer)
        //And we only return null if neither is valid
        let mut root = (-half_b - sqrt_d) / a;
        if !bounds.contains(&root) {
            root = (-half_b + sqrt_d) / a;
            if !bounds.contains(&root) {
                return None;
            }
        }

        let dist = root;
        let world_point = ray.at(dist);
        let local_point = (world_point - self.pos) / self.radius;
        let outward_normal = local_point;
        let ray_pos_inside = Vector3::dot(ray_dir, outward_normal) > 0.;
        //This flips the normal if the ray is inside the sphere
        //This forces the normal to always be going against the ray
        let ray_normal = if ray_pos_inside {
            -outward_normal
        } else {
            outward_normal
        };

        return Some(Intersection {
            pos_w: world_point,
            pos_l: local_point.to_point(),
            dist,
            normal: outward_normal,
            ray_normal,
            front_face: !ray_pos_inside,
            uv: sphere_uv(local_point),
            face: 0,
        });
    }
}

impl HasAabb for SphereMesh {
    fn aabb(&self) -> Option<&Aabb> { Some(&self.aabb) }
}
impl MeshProperties for SphereMesh {
    fn centre(&self) -> Point3 { self.pos }
}

// endregion Mesh Impl

// region Helper

/// Converts a point on a sphere (centred at [Point3::ZERO], radius `1`), into a UV coordinate
pub fn sphere_uv(p: Vector3) -> Point2 {
    let theta = Number::acos(-p.y);
    let phi = Number::atan2(-p.z, p.x) + Number::PI;

    let u = phi / (2. * Number::PI);
    let v = theta / Number::PI;
    return Point2::new(u, v);
}

//endregion Helper
