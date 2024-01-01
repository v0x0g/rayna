use glam::swizzles::*;
use glamour::ToRaw;

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
        let mut plane_dist = (self.size * winding * sgn) - ro;
        plane_dist *= ray.inv_dir();

        // Perform all three ray-box tests and cast to 0 or 1 on each axis.
        // Use a macro to eliminate the redundant code (no efficiency boost from doing so, of course!)
        macro_rules! test {
            ($u:ident, $vw:ident) => {
                // Is there a hit on this axis in front of the origin?
                bounds.contains(&plane_dist.$u) && {
                    // Is that hit within the face of the box?
                    let lhs = ray.at(plane_dist.$u).to_raw().$vw().abs();
                    let rhs = self.size.to_raw().$vw();
                    (lhs.x < rhs.x) && (lhs.y < rhs.y)
                }
            };
        }

        // Preserve exactly one element of `sgn`, with the correct sign
        // Also masks the distance by the non-zero axis
        // Dot product is faster than this CMOV chain, but doesn't work when distanceToPlane contains nans or infs.
        let (distance, sgn) = if test!(x, yz) {
            (plane_dist.x, Vector3::new(sgn.x, 0., 0.))
        } else if test!(y, zx) {
            (plane_dist.y, Vector3::new(0., sgn.y, 0.))
        } else if test!(z, xy) {
            (plane_dist.z, Vector3::new(0., 0., sgn.z))
        } else {
            // None of the tests matched, so we didn't hit any sides
            return None;
        };

        // Normal must face back along the ray. If you need
        // to know whether we're entering or leaving the box,
        // then just look at the value of winding. If you need
        // texture coordinates, then use box.invDirection * hitPoint.

        let ray_normal = sgn;

        Some(Intersection {
            pos: ray.at(distance),
            normal: ray_normal * winding,
            ray_normal,
            front_face: winding == 1.,
            dist: distance,
            material: self.material.clone(),
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
