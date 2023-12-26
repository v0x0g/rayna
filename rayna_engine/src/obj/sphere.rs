use crate::mat::MaterialType;
use crate::obj::Object;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use rayna_shared::def::types::{Number, Point3, Vector3};

#[derive(Clone, Debug)]
pub struct Sphere {
    pub pos: Point3,
    pub radius: Number,
    pub material: MaterialType,
}

impl Object for Sphere {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        //Do some ray-sphere intersection math to find if the ray intersects
        let ray_pos = ray.pos();
        let ray_dir = ray.dir();
        let ray_rel_pos = ray_pos - self.pos;

        // Quadratic formula variables
        let a = ray_dir.length_squared();
        let half_b = Vector3::dot(ray_rel_pos, ray_dir);
        let c = ray_rel_pos.length_squared() - (self.radius * self.radius);

        let discriminant = (half_b * half_b) - (a * c);
        if discriminant < 0. {
            return None;
        }; //No solutions to where ray intersects with sphere because of negative square root

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
        let local_point = world_point - self.pos;
        let outward_normal = local_point / self.radius;
        let ray_pos_inside = Vector3::dot(ray_dir, outward_normal) > 0.;
        //This flips the normal if the ray is inside the sphere
        //This forces the normal to always be going against the ray
        let ray_normal = if ray_pos_inside {
            -outward_normal
        } else {
            outward_normal
        };

        return Some(Intersection {
            pos: world_point,
            dist,
            normal: outward_normal,
            ray_normal,
            front_face: !ray_pos_inside,
            material: self.material.clone(),
        });
    }

    fn intersect_all(&self, ray: &Ray) -> Option<Box<dyn Iterator<Item = Intersection> + '_>> {
        //Do some ray-sphere intersection math to find if the ray intersects
        let ray_pos = ray.pos();
        let ray_dir = ray.dir();
        let ray_rel_pos = ray_pos - self.pos;

        // Quadratic formula
        let a = ray_dir.length_squared();
        let half_b = Vector3::dot(ray_rel_pos, ray_dir);
        let c = ray_rel_pos.length_squared() - (self.radius * self.radius);
        let discriminant = (half_b * half_b) - (a * c);

        if discriminant < 0. {
            return None;
        }; //No solutions to where ray intersects with sphere because of negative square root

        let sqrt_d = discriminant.sqrt();

        let root_1 = (-half_b - sqrt_d) / a;
        let root_2 = (-half_b + sqrt_d) / a;

        // TODO: Optimisation: if `approx::relative_eq!(root_1, 0.)`, then only one root
        //  In this case, return just a single boxed slice

        let intersections = [root_1, root_2].map(|k| {
            let world_point = ray.at(k);
            let local_point = world_point - self.pos;
            let outward_normal = local_point / self.radius;
            let inside = Vector3::dot(ray_dir, outward_normal) > 0.;
            //This flips the normal if the ray is inside the sphere
            //This forces the normal to always be going against the ray
            let ray_normal = if inside {
                -outward_normal
            } else {
                outward_normal
            };

            Intersection {
                pos: world_point,
                dist: k,
                normal: outward_normal,
                ray_normal,
                front_face: !inside,
                material: self.material.clone(),
            }
        });

        Some(Box::new(intersections.into_iter()))
    }
}
