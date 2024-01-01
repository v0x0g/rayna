use glam::{Vec3Swizzles, Vec4Swizzles};
use glamour::AsRaw;

use rayna_shared::def::types::{Number, Point3, Transform3, Vector2, Vector3};

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

/// Built instance of a box object
#[derive(Clone, Debug)]
pub struct AxisBoxObject {
    world_to_box: Transform3,
    box_to_world: Transform3,
    aabb: Aabb,
    material: MaterialType,
}

impl AxisBoxObject {
    pub fn new(transform: Transform3, material: MaterialType) -> Self {
        // Calculate bounding volume by getting all the corners in box coords,
        // translating to world coords, and encompassing in an AABB
        const CORNERS: [Point3; 8] = [
            Point3 {
                x: -1.,
                y: -1.,
                z: -1.,
            },
            Point3 {
                x: -1.,
                y: -1.,
                z: 1.,
            },
            Point3 {
                x: -1.,
                y: 1.,
                z: -1.,
            },
            Point3 {
                x: -1.,
                y: 1.,
                z: 1.,
            },
            Point3 {
                x: 1.,
                y: -1.,
                z: -1.,
            },
            Point3 {
                x: 1.,
                y: -1.,
                z: 1.,
            },
            Point3 {
                x: 1.,
                y: 1.,
                z: -1.,
            },
            Point3 {
                x: 1.,
                y: 1.,
                z: 1.,
            },
        ];

        let inverse = transform.inverse();
        let translated_corners = CORNERS.map(|p| transform.map_point(p));
        let aabb = Aabb::encompass_points(translated_corners);

        Self {
            box_to_world: transform,
            world_to_box: inverse,
            aabb,
            material,
        }
    }
}

impl From<AxisBoxBuilder> for AxisBoxObject {
    fn from(value: AxisBoxBuilder) -> Self {
        let size = value.corner_1 - value.corner_2;
        let centre = value.corner_1 + size / 2.;
        let transform = Transform3::from_scale(size)
            //.then_rotate()
            .then_translate(centre.into());
        Self::new(transform, value.material)
    }
}

impl Object for AxisBoxObject {
    //noinspection RsLiveness
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        /*
        CREDITS:
        Inigo Quilez
        https://iquilezles.org/articles/boxfunctions/ (Box Intersection Generic)
        https://www.shadertoy.com/view/ld23DV

        MODIFICATIONS:
            - Rename variables
            - Refactored to match my API
            - Replace OpenGL features
            - Add NaN and range checks
            - Combine into one matrix: `rad` (half-size) is contained inside it
        */

        // convert from world to box space
        let ro = self.world_to_box.map_point(ray.pos()).to_vector();
        let rd = self.world_to_box.map_vector(ray.dir());

        // TODO: RADIUS???
        let rad = Vector3::ONE;

        // Ray-box intersection, in box space
        let inv_d = ray.inv_dir();
        let s = -rd.signum();
        let t1 = (-ro + s * rad) * inv_d;
        let t2 = (-ro - s * rad) * inv_d;

        // Calculate distances

        let t_n = t1.x.max(t1.y).max(t1.z);
        let t_f = t2.x.min(t2.y).min(t2.z);
        // let t_n = t1.max_element();
        // let t_f = t2.min_element();

        if t_n > t_f || t_f < 0.0 {
            return None;
        }
        // if !bounds.range_overlaps(&t_n, &t_f) {return None;}

        // compute normal (in world space), face and UV
        // TODO: implement these
        let txi = self.box_to_world.matrix;
        let normal: Vector3;
        let _uv: Vector2;
        let _face: usize;
        let _local: Point3;

        if t1.x > t1.y && t1.x > t1.z {
            normal = (txi.x_axis.as_raw().xyz() * s.x).into();
            _uv = (ro.as_raw().yz() + rd.as_raw().yz() * t1.x).into();
            _face = 1 + (s.x as usize) / 2;
        } else if t1.y > t1.z {
            normal = (txi.y_axis.as_raw().xyz() * s.y).into();
            _uv = (ro.as_raw().zx() + rd.as_raw().zx() * t1.y).into();
            _face = 5 + (s.y as usize) / 2;
        } else {
            normal = (txi.z_axis.as_raw().xyz() * s.z).into();
            _uv = (ro.as_raw().xy() + rd.as_raw().xy() * t1.z).into();
            _face = 9 + (s.z as usize) / 2;
        };

        let dist = if bounds.contains(&t_n) {
            t_n
        } else if bounds.contains(&t_f) {
            t_f
        } else {
            unreachable!()
        };

        let pos = ray.at(dist);
        _local = self.box_to_world.map_point(pos);
        let inside = _local.max_element().abs() < 1.;

        Some(Intersection {
            pos,
            normal,
            material: self.material.clone(),
            dist,
            front_face: !inside,
            ray_normal: if inside { -normal } else { normal },
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
