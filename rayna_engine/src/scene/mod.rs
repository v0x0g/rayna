use crate::object::ObjectType;
use crate::shared::camera::Camera;
use crate::skybox::SkyboxType;

#[macro_export]
#[rustfmt::skip] // rustfmt is shit with macros
macro_rules! scene {
    {
        camera: $cam:expr,
        objects: [ $(
                $value:expr
        ),* $(,)? ]
    } => {
            $crate::scene::Scene {
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

pub mod stored {
    use super::Scene;
    use crate::material::dielectric::DielectricMaterial;
    use crate::material::lambertian::LambertianMaterial;
    use crate::material::metal::MetalMaterial;
    use crate::material::MaterialType;
    use crate::object::sphere::SphereBuilder;
    use crate::object::ObjectType;
    use crate::shared::camera::Camera;
    use crate::shared::rng;
    use crate::skybox::SkyboxType;
    use image::Pixel as _;
    use rand::{thread_rng, Rng};
    use rayna_shared::def::types::{Angle, Number, Pixel, Point3, Vector3};
    use static_init::*;

    #[dynamic]
    pub static SIMPLE: Scene = {
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
                SphereBuilder { // Small, top
                    pos: Point3::new(0., 0., 1.),
                    radius: 0.5,
                    material: MetalMaterial {
                        albedo: Pixel::from([0.8; 3]),
                        fuzz: 1.
                    }.into()
                },
                SphereBuilder { // Ground
                    pos: Point3::new(0., -100.5, -1.),
                    radius: 100.,
                    material: LambertianMaterial {
                        albedo: Pixel::from([0.5;3]),
                    }.into()
                }
            ]
        }
    };

    #[dynamic]
    pub static TRIO: Scene = {
        let material: MaterialType = LambertianMaterial {
            albedo: Pixel::from([1.; 3]),
        }
        .into();
        #[rustfmt::skip]
        scene! {
            camera: Camera {
                pos: Point3::new(0., 0., -3.),
                fwd: Vector3::Z,
                v_fov: Angle::from_degrees(45.),
                focus_dist: 3.,
                defocus_angle: Angle::from_degrees(3.)
            },
            objects: [
                SphereBuilder { // Left, big
                    pos: Point3::new(-0.2, 0., 0.),
                    radius: 0.25,
                    material: material.clone()
                },
                SphereBuilder { // Right, mid
                    pos: Point3::new(0.2, 0., 0.),
                    radius: 0.15,
                    material: material.clone()
                },
                SphereBuilder { // Small, top
                    pos: Point3::new(0., 0.5, 0.),
                    radius: 0.1,
                    material: material.clone()
                },
                SphereBuilder { // Ground
                    pos: Point3::new(0., -100.5, -1.),
                    radius: 100.,
                    material: material.clone()
                }
            ]
        }
    };

    #[dynamic]
    pub static GLASS: Scene = {
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
            SphereBuilder {
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
            SphereBuilder {
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
            SphereBuilder {
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
            SphereBuilder {
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
    };

    //noinspection SpellCheckingInspection
    #[dynamic]
    pub static BALLZ: Scene = {
        let camera = Camera {
            pos: Point3::new(13., 2., 3.),
            fwd: Vector3::new(-13., -2., -3.).normalize(),
            v_fov: Angle::from_degrees(20.),
            focus_dist: 10.,
            defocus_angle: Angle::from_degrees(0.6),
        };

        let mut objects = Vec::<ObjectType>::new();

        let grid_dims = -15..=15;
        let rng = &mut thread_rng();
        for a in grid_dims.clone() {
            for b in grid_dims.clone() {
                let (a, b) = (a as Number, b as Number);

                let material_choice = rng.gen::<Number>();

                let centre =
                    Point3::new(a, 0.2, b) + (Vector3::new(rng.gen(), 0., rng.gen()) * 0.9);
                const BIG_BALL_CENTRE: Point3 = Point3 {
                    x: 4.,
                    y: 0.2,
                    z: 0.,
                };

                if (centre - BIG_BALL_CENTRE).length() <= 0.9 {
                    continue;
                }

                let material: MaterialType = if material_choice < 0.7 {
                    LambertianMaterial {
                        albedo: Pixel::map2(
                            &rng::colour_rgb(rng),
                            &rng::colour_rgb(rng),
                            |a, b| a * b,
                        ),
                    }
                    .into()
                } else if material_choice <= 0.9 {
                    MetalMaterial {
                        albedo: rng::colour_rgb_range(rng, 0.5..=1.0),
                        fuzz: rng.gen_range(0.0..=0.5),
                    }
                    .into()
                } else {
                    DielectricMaterial {
                        albedo: [1.; 3].into(),
                        refractive_index: rng.gen_range(1.0..=10.0),
                    }
                    .into()
                };
                objects.push(
                    SphereBuilder {
                        pos: centre,
                        material,
                        radius: 0.2,
                    }
                    .into(),
                );
            }
        }

        objects.push(
            SphereBuilder {
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
            SphereBuilder {
                pos: Point3::new(-4., 1., 0.),
                radius: 1.,
                material: LambertianMaterial {
                    albedo: [0.4, 0.2, 0.1].into(),
                }
                .into(),
            }
            .into(),
        );
        objects.push(
            SphereBuilder {
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

        objects.push(
            SphereBuilder {
                pos: Point3::new(0., -1000., 0.),
                radius: 1000.,
                material: LambertianMaterial {
                    albedo: [0.5, 0.5, 0.5].into(),
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
    };
}
