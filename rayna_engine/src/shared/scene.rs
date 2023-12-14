use crate::obj::sphere::Sphere;
use crate::obj::Object;
use crate::shared::camera::Camera;
use crate::skybox::{DefaultSkybox, Skybox};
use rayna_shared::def::types::Vector;

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
                ),*],
                skybox: Box::new(DefaultSkybox {})
            }
        };
}

#[derive(Clone, Debug)]
pub struct Scene {
    // TODO: Maybe use [std::boxed::ThinBox] instead of [Box], might be better for perf
    pub objects: Vec<Box<dyn Object>>,
    pub camera: Camera,
    pub skybox: Box<dyn Skybox>,
}

impl Scene {
    pub fn empty() -> Self {
        scene! {
            camera: Camera {
                look_from: Vector::ZERO,
                // look_towards: Vector::ZERO,
                // up_vector: Vector::Y,
                // focus_dist: 1.,
                // lens_radius: 0.,
                // vertical_fov: 90.
            },
            objects: []
        }
    }

    pub fn simple() -> Self {
        scene! {
            camera: Camera {
                look_from: Vector::new(0., 0., 1.),
                // look_towards: Vector::ZERO,
                // up_vector: Vector::Y,
                // focus_dist: 1.,
                // lens_radius: 0.,
                // vertical_fov: 90.
            },
            objects: [
                Sphere {
                    pos: Vector::new(-0.2, 0., 0.),
                    radius: 0.25
                },
                Sphere {
                    pos: Vector::new(0.2, 0., 0.),
                    radius: 0.15
                },
                Sphere {
                    pos: Vector::new(0., 0.5, 0.),
                    radius: 0.1
                }
            ]
        }
    }
}
