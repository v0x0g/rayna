use glam::Vec3Swizzles;
use glamour::AsRaw;

use rayna_shared::def::types::{Number, Point3, Size3, Transform3, Vector2, Vector3};

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
    min: Point3,
    max: Point3,
    size: Size3,
    aabb: Aabb,
    material: MaterialType,
}

impl From<AxisBoxBuilder> for AxisBoxObject {
    fn from(value: AxisBoxBuilder) -> Self {
        let min = Point3::min(value.corner_1, value.corner_2);
        let max = Point3::max(value.corner_1, value.corner_2);
        let size = Size3::from(max - min);
        Self {
            aabb: Aabb::new(value.corner_1, value.corner_2),
            min,
            max,
            size,
            material: value.material.clone(),
        }
    }
}

impl Object for AxisBoxObject {
    //noinspection RsLiveness
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        /*
        CREDITS:
        Tavianator
        See [aabb.rs]

        Modified and extended beyond mere boolean checking
        */

        // let tx1 = (self.min.x - ray.pos().x) * ray.inv_dir().x;
        // let tx2 = (self.max.x - ray.pos().x) * ray.inv_dir().x;
        //
        // let ty1 = (self.min.y - ray.pos().y) * ray.inv_dir().y;
        // let ty2 = (self.max.y - ray.pos().y) * ray.inv_dir().y;
        //
        // let tz1 = (self.min.z - ray.pos().z) * ray.inv_dir().z;
        // let tz2 = (self.max.z - ray.pos().z) * ray.inv_dir().z;

        let t1 = (self.min - ray.pos()) * ray.inv_dir();
        let t2 = (self.max - ray.pos()) * ray.inv_dir();

        let mut tmin;
        let mut tmax;

        tmin = Number::min(t1.x, t2.x);
        tmax = Number::max(t1.x, t2.x);

        tmin = Number::max(tmin, Number::min(t1.y, t2.y));
        tmax = Number::min(tmax, Number::max(t1.y, t2.y));

        tmin = Number::max(tmin, Number::min(t1.z, t2.z));
        tmax = Number::min(tmax, Number::max(t1.z, t2.z));

        let dist = if tmin < tmax && bounds.contains(&tmin) {
            tmin
        } else if tmin < tmax && bounds.contains(&tmax) {
            tmax
        } else {
            return None;
        };

        let pos = ray.at(dist);
        // If clamped doesn't change then was in between `min..max`
        let inside = pos.clamp(self.min, self.max) == pos;

        // Find most significant element of ray dir,
        let dir = ray.dir();
        let dir_max = ray.dir().max_element();
        // NOTE: Negate so we face against the ray's direction
        let ray_normal = if dir_max == dir.x {
            -Vector3::X * dir.x.signum()
        } else if dir_max == dir.y {
            -Vector3::Y * dir.y.signum()
        } else {
            -Vector3::Z * dir.z.signum()
        };
        let ray_normal = Vector3::Y;

        //TODO: Find outer normal not ray normal
        Some(Intersection {
            pos,
            dist,
            front_face: !inside,
            normal: ray_normal,
            ray_normal,
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
