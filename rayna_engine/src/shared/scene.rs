use crate::def::types::Vec3;
use crate::obj::sphere::Sphere;
use crate::obj::Object;
use crate::shared::camera::Camera;

#[macro_export]
macro_rules! scene {
    {
        camera: $cam:expr,
        objects: [ $(
                $value:expr
        ),* $(,)? ]
    } => {
            $crate::shared::scene::Scene {
                camera: $cam,
                objects: vec![$(
                    std::boxed::Box::new($value) as std::boxed::Box<dyn $crate::obj::Object>
                ),*]
            }
        };
}

#[derive(Clone, Debug)]
pub struct Scene {
    // TODO: Maybe use [std::boxed::ThinBox] instead of [Box], might be better for perf
    pub objects: Vec<Box<dyn Object>>,
    pub camera: Camera,
}

impl Scene {
    pub fn empty() -> Self {
        scene! {
            camera: Camera {
                look_from: Vec3::ZERO,
                look_towards: Vec3::ZERO,
                up_vector: Vec3::Y,
                focus_dist: 1.,
                lens_radius: 0.,
                vertical_fov: 90.
            },
            objects: []
        }
    }

    pub fn simple() -> Self {
        scene! {
            camera: Camera {
                look_from: Vec3::new(0., 0., -1.),
                look_towards: Vec3::ZERO,
                up_vector: Vec3::Y,
                focus_dist: 1.,
                lens_radius: 0.,
                vertical_fov: 90.
            },
            objects: [
                Sphere {
                    pos: Vec3::new(0., 0., 0.),
                    radius: 0.5
                }
            ]
        }
    }
}
