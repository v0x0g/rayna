use crate::mat::lambertian::LambertianMaterial;
use crate::mat::metal::MetalMaterial;
use crate::mat::MaterialType;
use crate::obj::sphere::Sphere;
use crate::obj::Object;
use crate::shared::camera::Camera;
use crate::skybox::default_skybox::DefaultSkybox;
use crate::skybox::SkyboxType;
use rayna_shared::def::types::{Angle, Pixel, Point3, Vector3};

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
                skybox: SkyboxType::Default(DefaultSkybox {})
            }
        };
}

#[derive(Clone, Debug)]
pub struct Scene {
    // TODO: Maybe use [std::boxed::ThinBox] instead of [Box], might be better for perf
    pub objects: Vec<Box<dyn Object>>,
    pub camera: Camera,
    pub skybox: SkyboxType,
}

impl Scene {
    pub fn simple() -> Self {
        scene! {
            camera: Camera {
                pos: Point3::new(0., 0.5, -3.),
                fwd: Vector3::Z,
                v_fov: Angle::from_degrees(45.),
            },
            objects: [
                Sphere { // Small, top
                    pos: Point3::new(0., 0., 1.),
                    radius: 0.5,
                    material: MaterialType::Metal(MetalMaterial {
                        albedo: Pixel::from([0.8; 3]),
                    })
                },
                Sphere { // Ground
                    pos: Point3::new(0., -100.5, -1.),
                    radius: 100.,
                    material: MaterialType::Lambertian(LambertianMaterial {
                        albedo: Pixel::from([0.5;3]),
                        diffusion: 1.0
                    })
                }
            ]
        }
    }

    pub fn trio() -> Self {
        let material = MaterialType::Lambertian(LambertianMaterial {
            albedo: Pixel::from([1.; 3]),
            diffusion: 0.,
        });
        scene! {
            camera: Camera {
                pos: Point3::new(0., 0., -3.),
                fwd: Vector3::Z,
                v_fov: Angle::from_degrees(45.),
            },
            objects: [
                Sphere { // Left, big
                    pos: Point3::new(-0.2, 0., 0.),
                    radius: 0.25,
                    material: material.clone()
                },
                Sphere { // Right, mid
                    pos: Point3::new(0.2, 0., 0.),
                    radius: 0.15,
                    material: material.clone()
                },
                Sphere { // Small, top
                    pos: Point3::new(0., 0.5, 0.),
                    radius: 0.1,
                    material: material.clone()
                },
                Sphere { // Ground
                    pos: Point3::new(0., -100.5, -1.),
                    radius: 100.,
                    material: material.clone()
                }
            ]
        }
    }

    pub fn ballz() -> Self {
        let camera = Camera {
            pos: Point3::new(13., 2., 3.),
            fwd: Vector3::new(-13., -2., -3.).normalize(),
            v_fov: Angle::from_degrees(20.),
        };

        let mut objects = Vec::<Box<dyn Object>>::new();

        // objects.push(Box::new(Sphere{
        //     pos: Point3::new(0., 1., 0.),
        //     radius: 1.,
        //     material: MaterialType::Dielectric(DielectricMaterial{refractive_index: 1.5})
        // }));
        // objects.push(Box::new(Sphere{
        //     pos: Point3::new(-4., 1., 0.),
        //     radius: 1.,
        //     material: MaterialType::Lambertian(LambertianMaterial {albedo: [0.7, 0.6, 0.5].into()})
        // }));
        // objects.push(Box::new(Sphere{
        //     pos: Point3::new(4., 1., 0.),
        //     radius: 1.,
        //     material: MaterialType::Metal(MetalMaterial{albedo: [0.7, 0.6, 0.5].into(), fuzz: 0.})
        // }));

        Scene {
            camera,
            objects,
            skybox: SkyboxType::Default(DefaultSkybox {}),
        }
    }
}
