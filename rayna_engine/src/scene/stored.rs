//! This module is a repository of all builtin scenes in the engine
//!
//! There is no significance to them, apart from not having to manually create scenes by hand.
//!
//! There are some common ones [CORNELL] and [RTIAW_DEMO], that should be well known.

use crate::object::simple::SimpleObject;
use image::Pixel as _;
use noise::*;
use rand::{thread_rng, Rng};
use rayna_shared::def::types::{Angle, Channel, Number, Pixel, Point3, Transform3, Vector3};
use static_init::*;
use std::marker::PhantomData;

use crate::material::dielectric::DielectricMaterial;
use crate::material::isotropic::IsotropicMaterial;
use crate::material::lambertian::LambertianMaterial;
use crate::material::light::LightMaterial;
use crate::material::metal::MetalMaterial;
use crate::material::MaterialInstance;
use crate::mesh::axis_box::*;
use crate::mesh::homogenous_volume::HomogeneousVolumeBuilder;
use crate::mesh::infinite_plane::{InfinitePlaneBuilder, UvWrappingMode};
use crate::mesh::parallelogram::*;
use crate::mesh::planar::PlanarBuilder;
use crate::mesh::sphere::*;
use crate::mesh::MeshInstance;
use crate::object::list::ObjectList;
use crate::object::ObjectInstance;
use crate::shared::camera::Camera;
use crate::shared::rng;
use crate::skybox::SkyboxInstance;
use crate::texture::image::ImageTexture;
use crate::texture::noise::{ColourSource, LocalNoiseTexture, WorldNoiseTexture};
use crate::texture::solid::SolidTexture;
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
        objects: [
            SimpleObject::new(
                SphereBuilder {
                    // Small, top
                    pos: Point3::new(0., 0., 1.),
                    radius: 0.5,
                },
                MetalMaterial {
                    albedo: [0.8; 3].into(),
                    fuzz: 1.,
                },
            ),
            SimpleObject::new(
                SphereBuilder {
                    // Ground
                    pos: Point3::new(0., -100.5, -1.),
                    radius: 100.,
                },
                LambertianMaterial {
                    albedo: [0.5; 3].into(),
                    emissive: Default::default(),
                },
            ),
        ]
        .into(),
        skybox: SkyboxInstance::default(),
        phantom_mat: PhantomData,
        phantom_mesh: PhantomData,
    }
};

#[dynamic]
pub static TESTING: Scene = {
    let mut objects = Vec::new();

    objects.push(SimpleObject::new(
        // Ground
        InfinitePlaneBuilder {
            plane: PlanarBuilder::Vectors {
                p: Point3::ZERO,
                u: Vector3::X,
                v: Vector3::Z,
            },
            uv_wrap: UvWrappingMode::Wrap,
        },
        LambertianMaterial {
            albedo: [0.5; 3].into(),
            emissive: [0.; 3].into(),
        },
    ));
    objects.push(SimpleObject::new(
        // Slope
        InfinitePlaneBuilder {
            plane: PlanarBuilder::Vectors {
                p: (0., -0.1, 0.).into(),
                u: (1., 0.1, 0.).into(),
                v: Vector3::Z,
            },
            uv_wrap: UvWrappingMode::Wrap,
        },
        LambertianMaterial {
            albedo: [0.2; 3].into(),
            emissive: [0.1; 3].into(),
        },
    ));
    objects.push(SimpleObject::new(
        // Ball
        HomogeneousVolumeBuilder::<SphereMesh> {
            mesh: SphereBuilder {
                pos: (0., 1., 0.).into(),
                radius: 1.,
            }
            .into(),
            density: 1.,
        },
        IsotropicMaterial {
            albedo: [0.5; 3].into(),
        },
    ));
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
        phantom_mat: PhantomData,
        phantom_mesh: PhantomData,
    }
};

/// From **RayTracing in A Weekend**, the demo scene at the end of the chapter (extended of course)
#[dynamic]
pub static RTIAW_DEMO: Scene = {
    let mut objects = Vec::new();

    let grid_dims = -15..=15;
    let rng = &mut thread_rng();
    for a in grid_dims.clone() {
        for b in grid_dims.clone() {
            let (a, b) = (a as Number, b as Number);

            let centre = Point3::new(a, 0.2, b) + (Vector3::new(rng.gen(), 0., rng.gen()) * 0.9);
            const BIG_BALL_CENTRE: Point3 = Point3 { x: 4., y: 0.2, z: 0. };

            if (centre - BIG_BALL_CENTRE).length() <= 0.9 {
                continue;
            }

            let material_choice = rng.gen::<Number>();
            let material: MaterialInstance = if material_choice < 0.7 {
                LambertianMaterial {
                    albedo: Pixel::map2(&rng::colour_rgb(rng), &rng::colour_rgb(rng), |a, b| a * b).into(),
                    emissive: Default::default(),
                }
                .into()
            } else if material_choice <= 0.9 {
                MetalMaterial {
                    albedo: rng::colour_rgb_range(rng, 0.5..=1.0).into(),
                    fuzz: rng.gen_range(0.0..=0.5),
                }
                .into()
            } else {
                DielectricMaterial {
                    albedo: rng::colour_rgb_range(rng, 0.5..1.0).into(),
                    refractive_index: rng.gen_range(1.0..=10.0),
                }
                .into()
            };

            let obj_choice = rng.gen::<Number>();
            let obj: MeshInstance = if obj_choice < 0.7 {
                SphereBuilder {
                    pos: centre,
                    radius: 0.2,
                }
                .into()
            } else {
                AxisBoxBuilder::new_centred(centre, rng::vector_in_unit_cube_01(rng) * 0.8).into()
            };
            objects.push(SimpleObject::new(obj, material));
        }
    }

    objects.push(SimpleObject::new(
        SphereBuilder {
            pos: Point3::new(0., 1., 0.),
            radius: 1.,
        },
        DielectricMaterial {
            refractive_index: 1.5,
            albedo: [1.; 3].into(),
        },
    ));
    objects.push(SimpleObject::new(
        SphereBuilder {
            pos: Point3::new(-4., 1., 0.),
            radius: 1.,
        },
        LambertianMaterial {
            albedo: [0.4, 0.2, 0.1].into(),
            emissive: Default::default(),
        },
    ));
    objects.push(SimpleObject::new(
        SphereBuilder {
            pos: Point3::new(4., 1., 0.),
            radius: 1.,
        },
        MetalMaterial {
            albedo: [0.7, 0.6, 0.5].into(),
            fuzz: 0.,
        },
    ));

    objects.push(SimpleObject::new(
        SphereBuilder {
            pos: Point3::new(0., -1000., 0.),
            radius: 1000.,
        },
        LambertianMaterial {
            albedo: LocalNoiseTexture {
                source: ColourSource::Greyscale(ScalePoint::new(Perlin::new(69u32)).set_scale(10000.)).to_dyn_box(),
            }
            .into(),
            emissive: Default::default(),
        },
    ));

    Scene {
        camera: Camera {
            pos: Point3::new(13., 2., 3.),
            fwd: Vector3::new(-13., -2., -3.).normalize(),
            v_fov: Angle::from_degrees(20.),
            focus_dist: 10.,
            defocus_angle: Angle::from_degrees(0.6),
        },
        objects: objects.into(),
        skybox: SkyboxInstance::default(),
        phantom_mat: PhantomData,
        phantom_mesh: PhantomData,
    }
};

//noinspection SpellCheckingInspection
/// From **RayTracing The Next Week**, the demo scene at the end of the chapter (extended of course)
#[dynamic]
pub static RTTNW_DEMO: Scene = {
    let mut objects: Vec<ObjectInstance<MeshInstance, MaterialInstance>> = Vec::new();
    let rng = &mut thread_rng();

    // Const trait impls aren't yet stabilised, so we can't call TextureInstance::From(Pixel) yet
    const fn solid_texture(albedo: [Channel; 3]) -> TextureInstance {
        let pixel = Pixel { 0: albedo };
        let solid = SolidTexture { albedo: pixel };
        let texture = TextureInstance::SolidTexture(solid);
        texture
    }

    const BLACK_TEX: TextureInstance = solid_texture([0., 0., 0.]);

    {
        // BOXES (FLOOR)
        const COUNT: usize = 20;
        const HALF_COUNT: Number = COUNT as Number / 2.;
        const WIDTH: Number = 1.;
        const GROUND_MATERIAL: LambertianMaterial = LambertianMaterial {
            albedo: solid_texture([0.48, 0.83, 0.53]),
            emissive: BLACK_TEX,
        };

        let mut floor = vec![];
        for i in 0..COUNT {
            for j in 0..COUNT {
                let low = Point3::new(-HALF_COUNT * WIDTH, 0., -HALF_COUNT * WIDTH)
                    + Vector3::new(i as Number * WIDTH, 0., j as Number * WIDTH);
                let high = low + Vector3::new(WIDTH, rng.gen_range(0.0..=1.0), WIDTH);

                floor.push(
                    SimpleObject::new(
                        AxisBoxBuilder {
                            corner_1: low,
                            corner_2: high,
                        },
                        GROUND_MATERIAL,
                    )
                    .into(),
                );
            }
        }

        objects.push(ObjectList::new(floor).into());
    }

    {
        // LIGHT
        objects.push(
            SimpleObject::new(
                ParallelogramBuilder {
                    plane: PlanarBuilder::Vectors {
                        p: (1.23, 5.54, 1.47).into(),
                        u: (3., 0., 0.).into(),
                        v: (0., 0., 2.65).into(),
                    },
                },
                LightMaterial {
                    emissive: solid_texture([7.; 3]),
                },
            )
            .into(),
        );
    }

    {
        // BROWN SPHERE
        objects.push(
            SimpleObject::new(
                SphereBuilder {
                    pos: (4., 4., 2.).into(),
                    radius: 0.5,
                },
                LambertianMaterial {
                    albedo: solid_texture([0.7, 0.3, 0.1]),
                    emissive: BLACK_TEX,
                },
            )
            .into(),
        );

        // GLASS SPHERE
        objects.push(
            SimpleObject::new(
                SphereBuilder {
                    pos: (2.6, 1.5, 0.45).into(),
                    radius: 0.5,
                },
                DielectricMaterial {
                    albedo: [1.; 3].into(),
                    refractive_index: 1.5,
                },
            )
            .into(),
        );

        // METAL SPHERE (RIGHT)
        objects.push(
            SimpleObject::new(
                SphereBuilder {
                    pos: (0., 1.5, 1.45).into(),
                    radius: 0.5,
                },
                MetalMaterial {
                    albedo: [0.8, 0.8, 0.9].into(),
                    fuzz: 1.,
                },
            )
            .into(),
        );

        // SUBSURFACE SCATTER BLUE SPHERE (LEFT)
        objects.push(
            SimpleObject::new(
                SphereBuilder {
                    pos: (3.6, 1.5, 1.45).into(),
                    radius: 0.7,
                },
                DielectricMaterial {
                    albedo: [1.; 3].into(),
                    refractive_index: 1.5,
                },
            )
            .into(),
        );
        objects.push(
            SimpleObject::new(
                // BLUE HAZE INSIDE
                HomogeneousVolumeBuilder::<SphereMesh> {
                    mesh: SphereBuilder {
                        pos: (3.6, 1.5, 1.45).into(),
                        radius: 0.6,
                    }
                    .into(),
                    density: 80.,
                },
                IsotropicMaterial {
                    albedo: [0.01, 0.02, 0.6].into(),
                },
            )
            .into(),
        );

        // EARTH SPHERE
        objects.push(
            SimpleObject::new(
                SphereBuilder {
                    pos: (4., 2., 4.).into(),
                    radius: 1.0,
                },
                LambertianMaterial {
                    albedo: ImageTexture::from(
                        image::load_from_memory(include_bytes!("../../../media/textures/earthmap.jpg"))
                            .expect("compile-time image resource should be valid")
                            .into_rgb32f(),
                    )
                    .into(),
                    emissive: BLACK_TEX,
                },
            )
            .into(),
        );

        // NOISE SPHERE
        objects.push(
            SimpleObject::new(
                SphereBuilder {
                    pos: (2.2, 2.8, 3.0).into(),
                    radius: 0.8,
                },
                LambertianMaterial {
                    albedo: WorldNoiseTexture {
                        source: ColourSource::Greyscale(ScalePoint::new(Perlin::new(69)).set_scale(4.)).to_dyn_box(),
                    }
                    .into(),
                    emissive: BLACK_TEX,
                },
            )
            .into(),
        );
    }

    {
        // CUBE OF BALLS

        const COUNT: usize = 1000;
        const SPREAD: Number = 0.825;
        let white = LambertianMaterial {
            albedo: solid_texture([0.85; 3]),
            emissive: BLACK_TEX,
        };

        let balls_iter = (0..COUNT).into_iter().map(|_| {
            SimpleObject::new(
                SphereBuilder {
                    pos: (rng::vector_in_unit_cube(rng) * SPREAD).to_point(),
                    radius: 0.1,
                },
                white.clone(),
            )
            .into()
        });

        let balls_list = ObjectList::new_without_correction(
            balls_iter,
            Transform3::from_scale_rotation_translation(
                Vector3::ONE,
                Vector3::Y,
                Angle::from_degrees(15.),
                // The original cube was not centred at middle, but "centred" at the corner
                Vector3::new(-1.0, 2.7, 3.95) + Vector3::splat(SPREAD),
            ),
        );
        objects.push(ObjectInstance::ObjectList(balls_list));
    }

    {
        // HAZE

        objects.push(
            SimpleObject::new(
                HomogeneousVolumeBuilder::<SphereMesh> {
                    mesh: SphereBuilder {
                        pos: Point3::ZERO,
                        radius: 50.,
                    }
                    .into(),
                    density: 0.8,
                },
                IsotropicMaterial { albedo: [1.; 3].into() },
            )
            .into(),
        );
    }

    Scene {
        camera: Camera {
            pos: Point3::new(4.78, 2.78, -6.0),
            fwd: Vector3::new(-1., 0., 3.).normalize(),
            v_fov: Angle::from_degrees(40.),
            focus_dist: 1.,
            defocus_angle: Angle::from_degrees(0.0),
        },
        objects: objects.into(),
        skybox: None.into(),
        phantom_mat: PhantomData,
        phantom_mesh: PhantomData,
    }
};

/// The classic cornell box scene
#[dynamic]
pub static CORNELL: Scene = {
    let camera = Camera {
        pos: Point3::new(0.5, 0.5, 2.3),
        fwd: Vector3::new(0., 0., -1.).normalize(),
        v_fov: Angle::from_degrees(40.),
        focus_dist: 1.,
        defocus_angle: Angle::from_degrees(0.),
    };

    let mut objects = Vec::new();

    fn quad(
        objs: &mut Vec<SimpleObject<MeshInstance, MaterialInstance>>,
        p: impl Into<Point3>,
        u: impl Into<Vector3>,
        v: impl Into<Vector3>,
        albedo: impl Into<TextureInstance>,
        emissive: impl Into<TextureInstance>,
    ) {
        let p = p.into();
        let u = u.into();
        let v = v.into();
        objs.push(SimpleObject::new(
            ParallelogramBuilder {
                plane: PlanarBuilder::Vectors { p, u, v },
            },
            LambertianMaterial {
                albedo: albedo.into(),
                emissive: emissive.into(),
            },
        ));
    }

    let red = [0.65, 0.05, 0.05];
    let green = [0.12, 0.45, 0.15];
    let warm_grey = [0.85, 0.74, 0.55];
    let light = [15.; 3];
    let black = [0.; 3];

    let o = &mut objects;

    {
        // WALLS

        quad(o, (0., 0., 0.), Vector3::Y, Vector3::Z, red, black); // Left
        quad(o, (0., 0., 0.), Vector3::X, Vector3::Y, warm_grey, black); // Back
        quad(o, (0., 0., 0.), Vector3::Z, Vector3::X, warm_grey, black); // Floor
        quad(o, (1., 0., 0.), Vector3::Z, Vector3::Y, green, black); // Right
        quad(o, (0., 1., 0.), Vector3::X, Vector3::Z, warm_grey, black); // Ceiling
        quad(o, (0.4, 0.9999, 0.4), (0.2, 0., 0.), (0., 0., 0.2), black, light);
    }

    {
        // INNER BOXES

        // Big
        o.push(SimpleObject::new_with_correction(
            AxisBoxBuilder {
                corner_1: (0.231, 0., 0.117).into(),
                corner_2: (0.531, 0.595, 0.414).into(),
            },
            MetalMaterial {
                albedo: warm_grey.into(),
                fuzz: 0.,
            },
            Transform3::from_axis_angle(Vector3::Y, Angle::from_degrees(15.)),
        ));
        // Small
        o.push(SimpleObject::new_with_correction(
            AxisBoxBuilder {
                corner_1: (0.477, 0., 0.531).into(),
                corner_2: (0.774, 0.297, 0.829).into(),
            },
            LambertianMaterial {
                albedo: warm_grey.into(),
                emissive: [0.; 3].into(),
            },
            Transform3::from_axis_angle(Vector3::Y, Angle::from_degrees(-18.)),
        ));

        // Ball on Small
        o.push(SimpleObject::new(
            HomogeneousVolumeBuilder::<SphereMesh> {
                mesh: SphereBuilder {
                    pos: (0.6255, 0.43, 0.680).into(),
                    radius: 0.1,
                }
                .into(),
                density: 8.,
            },
            IsotropicMaterial {
                albedo: [0.3; 3].into(),
            },
        ));
    }

    Scene {
        camera,
        objects: objects.into(),
        skybox: None.into(),
        // skybox: SkyboxInstance::default(),
        phantom_mat: PhantomData,
        phantom_mesh: PhantomData,
    }
};
