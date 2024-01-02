use rayna_shared::def::types::{Number, Point3, Size3, Vector3};
use std::array::IntoIter;

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
    centre: Point3,
    size: Vector3,
    aabb: Aabb,
    material: MaterialType,
}

impl From<AxisBoxBuilder> for AxisBoxObject {
    fn from(value: AxisBoxBuilder) -> Self {
        let min = Point3::min(value.corner_1, value.corner_2);
        let max = Point3::max(value.corner_1, value.corner_2);
        let size = max - min;
        Self {
            aabb: Aabb::new(value.corner_1, value.corner_2),
            min,
            max,
            centre: min + (size / 2.).into(),
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

        let inv_dir = ray.inv_dir();
        let ray_pos = ray.pos();
        let dir_sign = ray.dir().signum();

        // m * (-ro +- s*rad); rad = size/2, -ro = centre-ro
        let v_dist_1 = inv_dir * ((self.centre - ray_pos) + (dir_sign * self.size / 2.));
        let v_dist_2 = inv_dir * ((self.centre - ray_pos) - (dir_sign * self.size / 2.));

        let v_dist_min = Vector3::min(v_dist_1, v_dist_2);
        let v_dist_max = Vector3::max(v_dist_1, v_dist_2);

        // NOTE: tmin will be negative, so `max_elem` gives closest to zero (nearest)
        let dist_min_max = v_dist_min.max_element();
        let dist_max_min = v_dist_max.min_element();

        // Closest intersection in bounds
        let dist = [dist_min_max, dist_max_min]
            .into_iter()
            .filter(|d| bounds.contains(d))
            .min_by(Number::total_cmp)?;

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
