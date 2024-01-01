use itertools::multizip;

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
    min: Point3,
    max: Point3,
    size: Vector3,
    aabb: Aabb,
    material: MaterialType,
}

impl From<AxisBoxBuilder> for AxisBoxObject {
    fn from(value: AxisBoxBuilder) -> Self {
        let aabb = Aabb::new(value.corner_1, value.corner_2);
        Self {
            max: aabb.max(),
            min: aabb.min(),
            size: aabb.size(),
            aabb,
            material: value.material,
        }
    }
}

impl Object for AxisBoxObject {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        // SOURCE:
        // Ingio Quilezles
        // https://iquilezles.org/articles/intersectors/

        let m = ray.inv_dir();
        let n = m * (ray.pos() - self.min);
        let k = m.abs() * self.size;
        let t1 = -n - k;
        let t2 = -n + k;
        let t_near = t1.max_element();
        let t_far = t2.min_element();

        // if( tN>tF || tF<0.0) return vec2(-1.0); // no intersection
        if t_near > t_far || t_far < 0. {
            return None;
        }
        let (mut out_normal, inside) = if t_near > 0. {
            // Ray originated outside the box
            (step(Vector3::splat(t_near), t1), false)
        } else {
            // Ray inside box
            (step(t2, Vector3::splat(t_far)), true)
        };

        fn step(edge: Vector3, inputs: Vector3) -> Vector3 {
            let mut arr = [0.; 3];
            for (out, e, i) in multizip((&mut arr, edge, inputs)) {
                *out = if i < e { 0. } else { 1. };
            }
            arr.into()
        }

        // let t = (bounds.clone()
        //     & Bounds {
        //         start: Some(t_near),
        //         end: Some(t_far),
        //     })
        // .start
        // .expect("bounds were validated");

        let t = if bounds.contains(&t_near) {
            t_near
        } else {
            t_far
        };

        out_normal *= -ray.dir().signum();

        Some(Intersection {
            pos: ray.at(t),
            dist: t,
            material: self.material.clone(),
            ray_normal: if inside { -out_normal } else { out_normal },
            normal: out_normal,
            front_face: !inside,
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
