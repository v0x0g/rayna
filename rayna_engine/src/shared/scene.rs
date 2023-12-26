use crate::material::dielectric::DielectricMaterial;
use crate::material::lambertian::LambertianMaterial;
use crate::material::metal::MetalMaterial;
use crate::material::MaterialType;
use crate::object::sphere::Sphere;
use crate::object::ObjectType;
use crate::shared::camera::Camera;
use crate::skybox::SkyboxType;
use rayna_shared::def::types::{Angle, Pixel, Point3, Vector3};

#[macro_export]
#[rustfmt::skip] // rustfmt is shit with macros
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
                     ($value).into()
                ),*],
                skybox: SkyboxType::default()
            }
        };
}

#[derive(Clone, Debug)]
pub struct Scene {
    pub objects: Vec<ObjectType>,
    pub camera: Camera,
    pub skybox: SkyboxType,
}

impl Scene {
    pub fn simple() -> Self {
        #[rustfmt::skip]
        scene! {
            camera: Camera {
                pos: Point3::new(0., 0.5, -3.),
                fwd: Vector3::Z,
                v_fov: Angle::from_degrees(45.),
                focus_dist: 3.,
                defocus_angle: Angle::from_degrees(10.)
            },
            objects: [
                Sphere { // Small, top
                    pos: Point3::new(0., 0., 1.),
                    radius: 0.5,
                    material: MetalMaterial {
                        albedo: Pixel::from([0.8; 3]),
                        fuzz: 1.
                    }.into()
                },
                Sphere { // Ground
                    pos: Point3::new(0., -100.5, -1.),
                    radius: 100.,
                    material: LambertianMaterial {
                        albedo: Pixel::from([0.5;3]),
                    }.into()
                }
            ]
        }
    }

    pub fn trio() -> Self {
        let material: MaterialType = LambertianMaterial {
            albedo: Pixel::from([1.; 3]),
        }
        .into();
        scene! {
            camera: Camera {
                pos: Point3::new(0., 0., -3.),
                fwd: Vector3::Z,
                v_fov: Angle::from_degrees(45.),
                focus_dist: 3.,
                defocus_angle: Angle::from_degrees(3.)
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

    pub fn glass() -> Self {
        let camera = Camera {
            pos: Point3::new(0., 0., 4.),
            fwd: Vector3::new(0., 0., -1.).normalize(),
            v_fov: Angle::from_degrees(45.),
            focus_dist: 3.,
            defocus_angle: Angle::from_degrees(3.),
        };

        let mut objects = Vec::<ObjectType>::new();

        // Ground
        objects.push(
            Sphere {
                pos: Point3::new(0., -100.5, 0.),
                radius: 100.,
                material: LambertianMaterial {
                    albedo: [0.8, 0.8, 0.0].into(),
                }
                .into(),
            }
            .into(),
        );

        // Left
        objects.push(
            Sphere {
                pos: Point3::new(-1., 0., 0.),
                radius: 0.5,
                material: DielectricMaterial {
                    albedo: [1.; 3].into(),
                    refractive_index: 1.5,
                }
                .into(),
            }
            .into(),
        );
        // Mid
        objects.push(
            Sphere {
                pos: Point3::new(0., 0., 0.),
                radius: 0.5,
                material: LambertianMaterial {
                    albedo: [0.1, 0.2, 0.5].into(),
                }
                .into(),
            }
            .into(),
        );
        // Right
        objects.push(
            Sphere {
                pos: Point3::new(1., 0., 0.),
                radius: 0.5,
                material: MetalMaterial {
                    albedo: [0.8, 0.6, 0.2].into(),
                    fuzz: 0.,
                }
                .into(),
            }
            .into(),
        );

        Scene {
            camera,
            objects,
            skybox: SkyboxType::default(),
        }
    }

    pub fn ballz() -> Self {
        let camera = Camera {
            pos: Point3::new(13., 2., 3.),
            fwd: Vector3::new(-13., -2., -3.).normalize(),
            v_fov: Angle::from_degrees(20.),
            focus_dist: 3.,
            defocus_angle: Angle::from_degrees(10.),
        };

        let mut objects = Vec::<ObjectType>::new();

        objects.push(
            Sphere {
                pos: Point3::new(0., 1., 0.),
                radius: 1.,
                material: DielectricMaterial {
                    refractive_index: 1.5,
                    albedo: [1.; 3].into(),
                }
                .into(),
            }
            .into(),
        );
        objects.push(
            Sphere {
                pos: Point3::new(-4., 1., 0.),
                radius: 1.,
                material: LambertianMaterial {
                    albedo: [0.7, 0.6, 0.5].into(),
                }
                .into(),
            }
            .into(),
        );
        objects.push(
            Sphere {
                pos: Point3::new(4., 1., 0.),
                radius: 1.,
                material: MetalMaterial {
                    albedo: [0.7, 0.6, 0.5].into(),
                    fuzz: 0.,
                }
                .into(),
            }
            .into(),
        );

        Scene {
            camera,
            objects,
            skybox: SkyboxType::default(),
        }
    }
}
