use getset::CopyGetters;
use glam::swizzles::*;
use glamour::FromRaw;
use glamour::ToRaw;
use rand_core::RngCore;

use crate::core::types::{Number, Point3, Size3, Vector2, Vector3};

use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use crate::shared::validate;

/// Built instance of a box mesh
#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct AxisBoxMesh {
    centre: Point3,
    radius: Vector3,
    inv_radius: Vector3,
    aabb: Aabb,
}

// region Constructors

impl AxisBoxMesh {
    pub fn new(a: impl Into<Point3>, b: impl Into<Point3>) -> Self {
        let aabb = Aabb::new(a, b);
        Self {
            centre: Point3::from((aabb.min().to_vector() + aabb.max().to_vector()) / 2.),
            radius: aabb.size() / 2.,
            inv_radius: (aabb.size() / 2.).recip(),
            aabb,
        }
    }

    pub fn new_centred(centre: impl Into<Point3>, size: impl Into<Size3>) -> Self {
        let (centre, size) = (centre.into(), size.into().to_vector());
        Self::new(centre + size / 2., centre - size / 2.)
    }
}

impl From<(Point3, Point3)> for AxisBoxMesh {
    fn from((a, b): (Point3, Point3)) -> Self { Self::new(a, b) }
}

impl From<[Point3; 2]> for AxisBoxMesh {
    fn from([a, b]: [Point3; 2]) -> Self { Self::new(a, b) }
}

impl From<(Point3, Size3)> for AxisBoxMesh {
    /// Creates a box with the given centre and dimensions
    fn from((centre, size): (Point3, Size3)) -> Self { Self::new_centred(centre, size) }
}

// endregion Constructors

// region Mesh Implementation

impl Mesh for AxisBoxMesh {
    //noinspection RsLiveness
    fn intersect(&self, ray: &Ray, bounds: &Interval<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
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
                    let uvs_raw = Vector2::from_raw(ro.to_raw().$vw() + (rd.to_raw().$vw() * dist));
                    let radius = Vector2::from_raw(self.radius.to_raw().$vw());
                    // Is that hit within the face of the box?
                    if (uvs_raw.x.abs() < radius.x) && (uvs_raw.y.abs() < radius.y) {
                        // Mask the sign to be the normal
                        let ray_normal = Vector3 {
                            $u: sgn.$u,
                            ..Vector3::ZERO
                        };
                        let pos_w = ray.at(dist);
                        // Remap from [-radius..radius] to [0..1]
                        let uvs = (uvs_raw / radius + Vector2::ONE) / 2.;
                        return Some(Intersection {
                            pos_w,
                            pos_l: pos_w - self.centre.to_vector(),
                            normal: ray_normal * winding,
                            ray_normal,
                            front_face: winding.is_sign_positive(),
                            dist,
                            uv: uvs.to_point(),
                            // x: 0,1; y: 2,3; z: 4,5; -ve sign first then positive sign
                            face: ((glam::uvec3(1, 5, 9).$u + sgn.$u as u32) / 2) as usize,
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
}

impl HasAabb for AxisBoxMesh {
    fn aabb(&self) -> Option<&Aabb> { Some(&self.aabb) }
}
impl MeshProperties for AxisBoxMesh {
    fn centre(&self) -> Point3 { self.centre }
}

// endregion Mesh Implementation
