use glam::swizzles::*;
use glamour::FromRaw;
use glamour::ToRaw;
use smallvec::SmallVec;

use rayna_shared::def::types::{Number, Point2, Point3, Vector2, Vector3};

use crate::accel::aabb::Aabb;
use crate::material::MaterialType;
use crate::object::{Object, ObjectType};
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

impl AxisBoxBuilder {
    pub fn new_corners(corner_1: Point3, corner_2: Point3, material: MaterialType) -> Self {
        Self {
            corner_1,
            corner_2,
            material,
        }
    }
    pub fn new_centred(centre: Point3, size: Vector3, material: MaterialType) -> Self {
        Self {
            corner_1: centre + size / 2.,
            corner_2: centre - size / 2.,
            material,
        }
    }
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

impl From<AxisBoxBuilder> for ObjectType {
    fn from(value: AxisBoxBuilder) -> ObjectType {
        AxisBoxObject::from(value).into()
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
            // Preserve exactly one element of `sgn`, with the correct sign
            // Also masks the distance by the non-zero axis
            // Dot product is faster than this CMOV chain, but doesn't work when distanceToPlane contains nans or infs.
            ($u:ident, $vw:ident) => {{
                let dist: Number = plane_dist.$u;
                // Is there a hit on this axis in the valid distance bounds?
                if bounds.contains(&dist) {
                    let uvs: Point2 =
                        Vector2::from_raw(ro.to_raw().$vw() + (rd.to_raw().$vw() * dist))
                            .abs()
                            .to_point();
                    let dims = self.radius.to_raw().$vw();
                    // Is that hit within the face of the box?
                    if (uvs.x < dims.x) && (uvs.y < dims.y) {
                        // Mask the sign to be the normal
                        let ray_normal = Vector3 {
                            $u: sgn.$u,
                            ..Vector3::ZERO
                        };
                        let pos_w = ray.at(dist);
                        return Some(Intersection {
                            pos_w,
                            pos_l: pos_w - self.centre.to_vector(),
                            normal: ray_normal * winding,
                            ray_normal,
                            front_face: winding.is_sign_positive(),
                            dist,
                            material: self.material.clone(),
                            uv: uvs,
                        });
                    }
                }
            }};
        }

        validate::vector3(&plane_dist);
        validate::vector3(&sgn);

        test!(x, yz);
        test!(y, zx);
        test!(z, xy);

        // None of the tests matched, so we didn't hit any sides
        return None;
    }

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>) {
        // Move to the box's reference frame. This is unavoidable and un-optimizable.
        let ro = ray.pos() - self.centre;
        let rd = ray.dir();

        // Rotation: `rd *= box.rot; ro *= box.rot;`

        // We'll use the negated sign of the ray direction in several places, so precompute it.
        // The sign() instruction is fast...but surprisingly not so fast that storing the result
        // temporarily isn't an advantage.
        let sgn = -rd.signum();

        // Winding direction: -1 if the ray starts inside of the box (i.e., and is leaving), +1 if it is starting outside of the box
        let winding = ((ro.abs() * self.inv_radius).max_element() - 1.).signum();

        // Ray-plane intersection. For each pair of planes, choose the one that is front-facing
        // to the ray and compute the distance to it.
        let plane_dist_1 = ((self.radius * winding * sgn) - ro) * ray.inv_dir();
        let plane_dist_2 = ((self.radius * -winding * sgn) - ro) * ray.inv_dir();

        // Perform all three ray-box tests on each axis.
        // Use a macro to eliminate the redundant code (no efficiency boost from doing so, of course!)
        macro_rules! test {
            (front: $u:ident, $vw:ident) => {{
                test!(@inner plane_dist_1, $u, $vw);
            }};
            (back: $u:ident, $vw:ident) => {{
                test!(@inner plane_dist_2, $u, $vw);
            }};
            (@inner $dist:ident, $u:ident, $vw:ident) => {{
                let dist: Number = $dist.$u;
                let uvs: Point2 = Vector2::from_raw(ro.to_raw().$vw() + (rd.to_raw().$vw() * dist)).abs().to_point();
                let dims = self.radius.to_raw().$vw();
                // Is that hit within the face of the box?
                if (uvs.x < dims.x) && (uvs.y < dims.y) {
                    // Preserve exactly one element of `sgn`, with the correct sign
                    // Also masks the distance by the non-zero axis
                    // Dot product is faster than this CMOV chain, but doesn't work when distanceToPlane contains nans or infs.
                    let ray_normal = Vector3 {
                        $u: sgn.$u,
                        ..Vector3::ZERO
                    };
                    let pos_w = ray.at(dist);
                    output.push(Intersection {
                        pos_w,
                        pos_l: pos_w - self.centre.to_vector(),
                        normal: ray_normal * winding,
                        ray_normal,
                        front_face: winding.is_sign_positive(),
                        dist,
                        material: self.material.clone(),
                        uv: uvs,
                    });
                }
            }};
        }

        validate::vector3(&plane_dist_1);
        validate::vector3(&plane_dist_2);
        validate::vector3(&sgn);

        test!(front: x, yz);
        test!(front: y, zx);
        test!(front: z, xy);
        test!(back: x, yz);
        test!(back: y, zx);
        test!(back: z, xy);
    }
    fn aabb(&self) -> Option<&Aabb> {
        Some(&self.aabb)
    }
}
