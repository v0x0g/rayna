use glam::swizzles::*;
use glamour::ToRaw;

use rayna_shared::def::types::{Number, Point3, Vector3};

use crate::accel::aabb::Aabb;
use crate::material::MaterialType;
use crate::object::Object;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::validate;

/// A builder struct used to create a box
///
/// Call [Into::into] or [AxisBoxObject::from] to create the actual object
#[derive(Clone, Debug)]
pub struct AxisBoxBuilder {
    pub corner_1: Point3,
    pub corner_2: Point3,
    pub material: MaterialType,
}

/// Built instance of a box object
#[derive(Clone, Debug)]
pub struct AxisBoxObject {
    centre: Point3,
    radius: Vector3,
    inv_radius: Vector3,
    aabb: Aabb,
    material: MaterialType,
}

impl From<AxisBoxBuilder> for AxisBoxObject {
    fn from(value: AxisBoxBuilder) -> Self {
        let aabb = Aabb::new(value.corner_1, value.corner_2);
        Self {
            centre: Point3::from((aabb.min().to_vector() + aabb.max().to_vector()) / 2.),
            radius: aabb.size() / 2.,
            inv_radius: (aabb.size() / 2.).recip(),
            aabb,
            material: value.material,
        }
    }
}

impl Object for AxisBoxObject {
    //noinspection RsLiveness
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        /*
        CREDITS:

        Title: "A Ray-Box Intersection Algorithm and Efficient Dynamic Voxel Rendering"
        Authors:
            - Alexander Majercik
            - Cyril Crassin
            - Peter Shirley
            - Morgan McGuire
        URL: <https://jcgt.org/published/0007/03/04/>
        Publisher: Journal of Computer Graphics Techniques (JCGT)
        Version: vol. 7, no. 3, 66-81, 2018
        */

        // Move to the box's reference frame. This is unavoidable and un-optimizable.
        let ro = ray.pos() - self.centre;
        let rd = ray.dir();

        // Rotation: `rd *= box.rot; ro *= box.rot;`

        // Winding direction: -1 if the ray starts inside of the box (i.e., and is leaving), +1 if it is starting outside of the box
        let winding = ((ro.abs() * self.inv_radius).max_element() - 1.).signum();

        // We'll use the negated sign of the ray direction in several places, so precompute it.
        // The sign() instruction is fast...but surprisingly not so fast that storing the result
        // temporarily isn't an advantage.
        let sgn = -rd.signum();

        // Ray-plane intersection. For each pair of planes, choose the one that is front-facing
        // to the ray and compute the distance to it.
        let mut plane_dist = (self.radius * winding * sgn) - ro;
        plane_dist *= ray.inv_dir();

        // Perform all three ray-box tests on each axis.
        // Use a macro to eliminate the redundant code (no efficiency boost from doing so, of course!)
        macro_rules! test {
            ($u:ident, $vw:ident) => {
                // Is there a hit on this axis in the valid distance bounds?
                bounds.contains(&plane_dist.$u) && {
                    // Is that hit within the face of the box?
                    let plane_uvs_from_centre =
                        (ro.to_raw().$vw() + (rd.to_raw().$vw() * plane_dist.$u)).abs();
                    let side_dimensions = self.radius.to_raw().$vw();
                    (plane_uvs_from_centre.x < side_dimensions.x)
                        && (plane_uvs_from_centre.y < side_dimensions.y)
                }
            };
        }

        validate::vector3(&plane_dist);
        validate::vector3(&sgn);

        // Preserve exactly one element of `sgn`, with the correct sign
        // Also masks the distance by the non-zero axis
        // Dot product is faster than this CMOV chain, but doesn't work when distanceToPlane contains nans or infs.
        let (distance, ray_normal) = if test!(x, yz) {
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

        Some(Intersection {
            pos: ray.at(distance),
            normal: ray_normal * winding,
            ray_normal,
            front_face: winding.is_sign_positive(),
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
