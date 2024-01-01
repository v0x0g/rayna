use glam::swizzles::*;
use glam::BVec3;
use glamour::{Swizzle, ToRaw};
use itertools::multizip;
use std::ops::Sub;

use rayna_shared::def::types::{Number, Point3, Vector3};

use crate::accel::aabb::Aabb;
use crate::material::MaterialType;
use crate::object::Object;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;

/// A builder struct used to create a box
///
/// Call [Into::into] or [AxisBoxObject::from] to create the actual object
#[derive(Clone, Debug)]
pub struct AxisBoxBuilder {
    pub corner_1: Point3,
    pub corner_2: Point3,
    pub material: MaterialType,
}

///
#[derive(Clone, Debug)]
pub struct AxisBoxObject {
    centre: Point3,
    size: Vector3,
    inv_size: Vector3,
    aabb: Aabb,
    material: MaterialType,
}

impl From<AxisBoxBuilder> for AxisBoxObject {
    fn from(value: AxisBoxBuilder) -> Self {
        let aabb = Aabb::new(value.corner_1, value.corner_2);
        Self {
            centre: Point3::from((aabb.min().to_vector() + aabb.max().to_vector()) / 2.),
            size: aabb.size(),
            inv_size: aabb.size().recip(),
            aabb,
            material: value.material,
        }
    }
}

impl Object for AxisBoxObject {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        // Move to the box's reference frame. This is unavoidable and un-optimizable.
        let ro = ray.pos() - self.centre;
        let rd = ray.dir();

        // Rotation: `rd *= box.rot; ro *= box.rot;`

        // Winding direction: -1 if the ray starts inside of the box (i.e., and is leaving), +1 if it is starting outside of the box
        // let winding = ((ro.abs() * self.inv_size).max_element() - 1.).signum();
        let winding = if (ro.abs() * self.inv_size).max_element() < 1. {
            -1.
        } else {
            1.
        };

        // We'll use the negated sign of the ray direction in several places, so precompute it.
        // The sign() instruction is fast...but surprisingly not so fast that storing the result
        // temporarily isn't an advantage.
        let sgn = -rd.signum();

        // Ray-plane intersection. For each pair of planes, choose the one that is front-facing
        // to the ray and compute the distance to it.
        let mut plane_dist = self.size * winding * sgn - ro;

        // Rotation: `if oriented {r /= rd} else {d *= ray.inv_dir()};

        plane_dist *= ray.inv_dir();

        // Perform all three ray-box tests and cast to 0 or 1 on each axis.
        // Use a macro to eliminate the redundant code (no efficiency boost from doing so, of course!)
        // Could be written with
        // #   define TEST(U, VW)\
        // /* Is there a hit on this axis in front of the origin? Use multiplication instead of && for a small speedup */\
        // (distanceToPlane.U >= 0.0) && \
        // /* Is that hit within the face of the box? */\
        // all( lessThan(  abs(ray.origin.VW + ray.direction.VW * distanceToPlane.U), box.radius.VW  ) )

        (ro.to_raw().x >= 0.) &&
            // Check if LHS of subtract is less than RHS of subtract, in all element positions
            // But with faster math
            {
                let lhs = ((ro.to_raw().yz() + rd.to_raw().yz() * plane_dist.x).abs());
                let rhs = self.size.to_raw().yz();
                lhs.x < rhs.x && lhs.y < rhs.y;
            };

        macro_rules! test {
            ($u:ident, $vw:ident) => {
            // Is there a hit on this axis in front of the origin?
            (plane_dist.x >= 0.)
            && // Is that hit within the face of the box?math
            (((ro.to_raw().yz() + rd.to_raw().yz() * plane_dist.x).abs()) - self.size.to_raw().yz())
                .is_negative_bitmask() == 0b11_u32;

                (d.$u >= 0.) && all(lessThan((ro.to_raw().$vw() + rd.to_raw().$vw() * d.$u).abs(), self.size.$vw))
            };
        }
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
