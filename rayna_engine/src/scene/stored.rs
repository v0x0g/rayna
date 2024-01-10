//! This module is a repository of all builtin scenes in the engine
//!
//! There is no significance to them, apart from not having to manually create scenes by hand.
//!
//! There are some common ones [CORNELL] and [RTIAW_DEMO], that should be well known.

use image::Pixel as _;
use noise::*;
use rand::{thread_rng, Rng};
use static_init::*;
use std::sync::Arc;

use rayna_shared::def::types::{Angle, Number, Pixel, Point3, Size2, Vector2, Vector3};

use crate::material::dielectric::DielectricMaterial;
use crate::material::lambertian::LambertianMaterial;
use crate::material::metal::MetalMaterial;
use crate::material::MaterialInstance;
use crate::object::axis_box::*;
use crate::object::parallelogram::*;
use crate::object::sphere::*;
use crate::object::ObjectInstance;
use crate::shared::camera::Camera;
use crate::shared::rng;
use crate::skybox::SkyboxInstance;
use crate::texture::image::ImageTexture;
use crate::texture::noise::{ColourSource, LocalNoiseTexture, UvNoiseTexture};
use crate::texture::TextureInstance;

use super::Scene;

/// Super simple scene, just a ground sphere and a small sphere
#[dynamic]
pub static SIMPLE: Scene = {
    Scene {
        camera: Camera {
            pos: Point3::new(0., 0.5, -3.),
            fwd: Vector3::Z,
            v_fov: Angle::from_degrees(45.),
            focus_dist: 3.,
            defocus_angle: Angle::from_degrees(0.),
        },
        objects: vec![
            SphereBuilder {
                // Small, top
                pos: Point3::new(0., 0., 1.),
                radius: 0.5,
                material: MetalMaterial {
                    albedo: Pixel::from([0.8; 3]),
                    fuzz: 1.,
                }
                .into(),
            },
            SphereBuilder {
                // Ground
                pos: Point3::new(0., -100.5, -1.),
                radius: 100.,
                material: LambertianMaterial {
                    albedo: [0.5; 3].into(),
                    emissive: Default::default(),
                }
                .into(),
            },
        ]
        .into(),
        skybox: SkyboxInstance::default(),
    }
};

#[dynamic]
pub static TESTING: Scene = {
    let mut objects = Vec::<ObjectInstance>::new();
    objects.push(
        SphereObject::from(SphereBuilder {
            // Small, top
            pos: Point3::new(0., 2., 0.),
            radius: 0.5,
            material: MetalMaterial {
                albedo: [0.8; 3].into(),
                fuzz: 1.,
            }
            .into(),
        })
        .into(),
    );
    // objects.push(
    //     AxisBoxObject::from(AxisBoxBuilder {
    //         material: LambertianMaterial {
    //             albedo: [1.; 3].into(),
    //         }
    //         .into(),
    //         corner_1: Point3::new(-1., 0., -1.),
    //         corner_2: Point3::new(1., 1., 1.),
    //     })
    //     .into(),
    // );
    objects.push(
        ParallelogramObject::from(ParallelogramBuilder {
            material: LambertianMaterial {
                albedo: [1.; 3].into(),
                emissive: Default::default(),
            }
            .into(),
            corner_origin: Point3::new(0., 0., 0.),
            corner_upper: Point3::new(-1., 1., 0.),
            corner_right: Point3::new(1., 0.5, 0.),
        })
        .into(),
    );
    // objects.push(
    //     SphereObject::from(SphereBuilder {
    //         // Ground
    //         pos: Point3::new(0., -100.5, -1.),
    //         radius: 100.,
    //         material: LambertianMaterial {
    //             albedo: Pixel::from([0.5; 3]),
    //         }
    //         .into(),
    //     })
    //     .into(),
    // );
    Scene {
        camera: Camera {
            pos: Point3::new(0., 0.5, -3.),
            fwd: Vector3::Z,
            v_fov: Angle::from_degrees(45.),
            focus_dist: 3.,
            defocus_angle: Angle::from_degrees(0.),
        },
        objects: objects.into(),
        skybox: SkyboxInstance::default(),
    }
};

#[dynamic]
pub static TRIO: Scene = {
    let material: MaterialInstance = LambertianMaterial {
        albedo: [1.; 3].into(),
        emissive: Default::default(),
    }
    .into();
    Scene {
        camera: Camera {
            pos: Point3::new(0., 0., -3.),
            fwd: Vector3::Z,
            v_fov: Angle::from_degrees(45.),
            focus_dist: 3.,
            defocus_angle: Angle::from_degrees(3.),
        },
        objects: vec![
            SphereBuilder {
                // Left, big
                pos: Point3::new(-0.2, 0., 0.),
                radius: 0.25,
                material: material.clone(),
            },
            SphereBuilder {
                // Right, mid
                pos: Point3::new(0.2, 0., 0.),
                radius: 0.15,
                material: material.clone(),
            },
            SphereBuilder {
                // Small, top
                pos: Point3::new(0., 0.5, 0.),
                radius: 0.1,
                material: material.clone(),
            },
            SphereBuilder {
                // Ground
                pos: Point3::new(0., -100.5, -1.),
                radius: 100.,
                material: material.clone(),
            },
        ]
        .into(),
        skybox: SkyboxInstance::default(),
    }
};

//noinspection SpellCheckingInspection

/// From **RayTracing in A Weekend**, the demo scene at the end of the chapter (extended of course)
#[dynamic]
pub static RTIAW_DEMO: Scene = {
    let camera = Camera {
        pos: Point3::new(13., 2., 3.),
        fwd: Vector3::new(-13., -2., -3.).normalize(),
        v_fov: Angle::from_degrees(20.),
        focus_dist: 10.,
        defocus_angle: Angle::from_degrees(0.6),
    };

    let mut objects = Vec::<ObjectInstance>::new();

    let grid_dims = -15..=15;
    let rng = &mut thread_rng();
    for a in grid_dims.clone() {
        for b in grid_dims.clone() {
            let (a, b) = (a as Number, b as Number);

            let material_choice = rng.gen::<Number>();

            let centre = Point3::new(a, 0.2, b) + (Vector3::new(rng.gen(), 0., rng.gen()) * 0.9);
            const BIG_BALL_CENTRE: Point3 = Point3 { x: 4., y: 0.2, z: 0. };

            if (centre - BIG_BALL_CENTRE).length() <= 0.9 {
                continue;
            }

            let material: MaterialInstance = if material_choice < 0.7 {
                LambertianMaterial {
                    albedo: Pixel::map2(&rng::colour_rgb(rng), &rng::colour_rgb(rng), |a, b| a * b).into(),
                    emissive: Default::default(),
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
                    albedo: rng::colour_rgb_range(rng, 0.5..1.0),
                    refractive_index: rng.gen_range(1.0..=10.0),
                }
                .into()
            };

            let obj_choice = rng.gen::<Number>();
            let obj = if obj_choice < 0.7 {
                SphereBuilder {
                    pos: centre,
                    material,
                    radius: 0.2,
                }
                .into()
            } else {
                AxisBoxBuilder::new_centred(centre, rng::vector_in_unit_cube_01(rng) * 0.8, material).into()
            };
            objects.push(obj);
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
                emissive: Default::default(),
            }
            .into(),
        }
        .into(),
    );
    // objects.push(
    //     SphereBuilder {
    //         pos: Point3::new(4., 1., 0.),
    //         radius: 1.,
    //         material: MetalMaterial {
    //             albedo: [0.7, 0.6, 0.5].into(),
    //             fuzz: 0.,
    //         }
    //         .into(),
    //     }
    //     .into(),
    // );
    objects.push(
        SphereBuilder {
            pos: Point3::new(4., 1., 0.),
            radius: 1.,
            material: LambertianMaterial {
                albedo: ImageTexture {
                    scale: Size2::splat(1.),
                    offset: Vector2::ZERO,
                    image: Arc::new(
                        image::open("./media/textures/earthmap.jpg")
                            .expect("open")
                            .into_rgb32f(),
                    ),
                }
                .into(),
                emissive: Default::default(),
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
                albedo: LocalNoiseTexture {
                    func: ColourSource::Greyscale(ScalePoint::new(Perlin::new(69u32)).set_scale(10000.)).as_dyn_box(),
                }
                .into(),
                emissive: Default::default(),
            }
            .into(),
        }
        .into(),
    );

    Scene {
        camera,
        objects: objects.into(),
        skybox: SkyboxInstance::default(),
    }
};

/// The classic cornell box scene
#[dynamic]
pub static CORNELL: Scene = {
    let camera = Camera {
        pos: Point3::new(0., 0., 4.),
        fwd: Vector3::new(0., 0., -1.).normalize(),
        v_fov: Angle::from_degrees(45.),
        focus_dist: 3.,
        defocus_angle: Angle::from_degrees(3.),
    };

    let mut objects = Vec::<ObjectInstance>::new();

    fn quad(
        objs: &mut Vec<ObjectInstance>,
        q: impl Into<Point3>,
        a: impl Into<Point3>,
        b: impl Into<Point3>,
        albedo: impl Into<TextureInstance>,
        emissive: impl Into<TextureInstance>,
    ) {
        let quad = ParallelogramBuilder {
            corner_origin: q.into(),
            corner_upper: b.into(),
            corner_right: a.into(),
            material: LambertianMaterial {
                albedo: albedo.into(),
                emissive: emissive.into(),
            }
            .into(),
        };
        objs.push(quad.into());
    }

    {
        let red = [0.65, 0.05, 0.05];
        let green = [0.12, 0.45, 0.15];
        let white = [0.73; 3];
        let light = [15.; 3];
        let black = [0.; 3];
        let o = &mut objects;
        quad(o, [555., 0., 0.], [0., 555., 0.], [0., 0., 555.], green, black);
        quad(o, [0., 0., 0.], [0., 555., 0.], [0., 0., 555.], red, black);
        quad(o, [343., 554., 332.], [-130., 0., 0.], [0., 0., -105.], black, light);
        quad(o, [0., 0., 0.], [555., 0., 0.], [0., 0., 555.], white, black);
        quad(o, [555., 555., 555.], [-555., 0., 0.], [0., 0., -555.], white, black);
        quad(o, [0., 0., 555.], [555., 0., 0.], [0., 555., 0.], white, black);
    }

    Scene {
        camera,
        objects: objects.into(),
        skybox: SkyboxInstance::default(),
    }
};
