//! This module is a repository of all builtin scenes in the engine
//!
//! There is no significance to them, apart from not having to manually create scenes by hand.
//!
//! There are some common ones that should be well known (such as the infamous [cornell box](`self::CORNELL`)).

#![allow(non_snake_case)]
#![allow(unused)]

use crate::core::rng;
use crate::core::types::{Angle, Channel, Colour, Image, Number, Point3, Size3, Transform3, Vector3};
use crate::math::num::Lerp;
use crate::{
    material::{
        dielectric::DielectricMaterial, isotropic::IsotropicMaterial, lambertian::LambertianMaterial,
        light::LightMaterial, metal::MetalMaterial, MaterialInstance,
    },
    mesh::{
        axis_box::AxisBoxMesh,
        cylinder::CylinderMesh,
        list::ListMesh,
        planar::{InfinitePlaneMesh, ParallelogramMesh, Plane, UvWrappingMode},
        polygonised::PolygonisedIsosurfaceMesh,
        raymarched::RaymarchedIsosurfaceMesh,
        sphere::SphereMesh,
        MeshInstance,
    },
    noise::boxed::BoxedNoise,
    object::{simple::SimpleObject, transform::ObjectTransform, volumetric::VolumetricObject},
    scene::{camera::Camera, Scene},
    skybox::{
        none::NoSkybox,
        simple::{SimpleSkybox, WhiteSkybox},
    },
    texture::{
        checker::{UvCheckerTexture, WorldCheckerTexture},
        image::ImageTexture,
        noise::{NoiseSource, NoiseTexture},
        solid::SolidTexture,
        TextureToken,
    },
};
use rand::Rng as _;

/// Holds a preset scene that is pre-made, so that scenes can easily be loaded
/// without having to recreate them each time
#[derive(Debug, Clone)]
pub struct PresetScene {
    pub name: &'static str,
    pub camera: Camera,
    pub scene: Scene,
}

// FIXME: Calling these presets is extremely slow.
//  `RTTNW_DEMO()` takes ~1.4 sec, `ALL()` takes ~4.1 sec

// /// All the preset scenes.
// ///
// /// # Warning
// /// Currently all scenes are re-created each time this is called.
// /// You will want to cache this value somewhere
pub fn ALL() -> [PresetScene; 5] {
    [TESTING(), RTIAW_DEMO(), RTIAW_DEMO_DARK(), RTTNW_DEMO(), CORNELL()]
}

/// A testing scene used only during development
pub fn TESTING() -> PresetScene {
    let mut scene = Scene::new();
    scene.set_skybox(WhiteSkybox);

    // The lack of two-phase borrows and  E0499 are the bane of this function's existence

    let glass_tex = scene.add_tex([0.28, 0.53, 0.7]);
    let glass_mat = scene.add_mat(DielectricMaterial {
        albedo: glass_tex,
        density: 1.0,
        refractive_index: 1.335,
    });
    let drop_mesh = PolygonisedIsosurfaceMesh::new_in(&mut scene, 64, |p_raw| {
        let [x, y, z] = p_raw.into();

        // NOTE: Point is given to us inside range `0.0..=1.0`
        //  So map it to the appropriate range for our shape
        let [x, y, z] = [
            Lerp::lerp(-0.5, 0.5, x),
            Lerp::lerp(1.0, 0.0, y),
            Lerp::lerp(-0.5, 0.5, z),
        ];

        const A: Number = 11.0;
        const B: Number = 0.6;
        x.powi(2) + z.powi(2) + y.powf(A + (B)) - y.powf(A)
    });
    let drop_mesh = scene.add_mesh(drop_mesh);
    scene.add_obj(SimpleObject::new_from(&scene, drop_mesh, glass_mat, None));
    let sphere_mesh = scene.add_mesh(SphereMesh::new((0., -0.3, 0.), 0.1));
    scene.add_obj(SimpleObject::new_from(&scene, sphere_mesh, glass_mat, None));

    PresetScene {
        name: "Test",
        camera: Camera {
            pos: Point3::new(0.5, 0.1, 0.7),
            fwd: Vector3::new(0., 0., -1.).normalize(),
            v_fov: Angle::from_degrees(40.),
            focus_dist: 1.,
            defocus_angle: Angle::from_degrees(0.),
        },
        scene,
    }
}

/// From **RayTracing in A Weekend**, the demo scene at the end of the chapter (extended of course)
pub fn RTIAW_DEMO() -> PresetScene {
    let mut scene = Scene::new();
    scene.set_skybox(SimpleSkybox);

    let grid_dims = -15..=15;
    let rng = &mut rand::thread_rng();
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
                    albedo: scene.add_tex(rng::colour_rgb(rng) * rng::colour_rgb(rng)),
                }
                .into()
            } else if material_choice <= 0.9 {
                MetalMaterial {
                    albedo: scene.add_tex(rng::colour_rgb_range(rng, 0.5..=1.0)),
                    fuzz: rng.gen_range(0.0..=0.5),
                }
                .into()
            } else {
                DielectricMaterial {
                    albedo: scene.add_tex(rng::colour_rgb_range(rng, 0.5..1.0)),
                    refractive_index: rng.gen_range(1.0..=10.0),
                    density: 69.0,
                }
                .into()
            };

            let mesh_choice = rng.gen::<Number>();
            let mesh: MeshInstance = if mesh_choice < 0.7 {
                SphereMesh::new(centre, 0.2).into()
            } else {
                AxisBoxMesh::new_centred(centre, rng::vector_in_unit_cube_01(rng) * 0.8).into()
            };
            let obj = SimpleObject::new_in(&mut scene, mesh, material, None);
            scene.add_obj(obj);
        }
    }

    let mesh = scene.add_mesh(SphereMesh::new((0., 1., 0.), 1.));
    let tex = scene.add_tex([1.; 3]);
    let mat = scene.add_mat(DielectricMaterial {
        refractive_index: 1.5,
        density: 69.0,
        albedo: tex,
    });
    scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, None));
    let mesh = scene.add_mesh(SphereMesh::new((-4., 1., 0.), 1.));
    let tex = scene.add_tex([0.4, 0.2, 0.1]);
    let mat = scene.add_mat(LambertianMaterial { albedo: tex });
    scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, None));
    let mesh = scene.add_mesh(SphereMesh::new((4., 1., 0.), 1.));
    let tex = scene.add_tex([0.7, 0.6, 0.5]);
    let mat = scene.add_mat(MetalMaterial { albedo: tex, fuzz: 0. });
    scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, None));

    let mesh = scene.add_mesh(SphereMesh::new((0., -1000., 0.), 1000.));
    let noise = scene.add_noise3(BoxedNoise::from(
        noise::ScalePoint::new(noise::Perlin::new(69u32)).set_scale(10000.),
    ));
    let tex = scene.add_tex(NoiseTexture::Monochrome {
        noise: NoiseSource::LocalPos(noise),
        colour: Colour::WHITE,
    });
    let mat = scene.add_mat(LambertianMaterial { albedo: tex });
    scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, None));

    PresetScene {
        name: "RTIAW Demo",
        camera: Camera {
            pos: Point3::new(13., 2., 3.),
            fwd: Vector3::new(-13., -2., -3.).normalize(),
            v_fov: Angle::from_degrees(20.),
            focus_dist: 10.,
            defocus_angle: Angle::from_degrees(0.6),
        },
        scene,
    }
}

/// From **RayTracing in A Weekend**, the demo scene at the end of the chapter (night edition)
pub fn RTIAW_DEMO_DARK() -> PresetScene {
    let mut scene = Scene::new();
    scene.set_skybox(NoSkybox);

    let grid_dims = -15..=15;
    let rng = &mut rand::thread_rng();

    // Objects
    for a in grid_dims.clone() {
        for b in grid_dims.clone() {
            let (a, b) = (a as Number, b as Number);

            let centre = Point3::new(a, 0.2, b) + (Vector3::new(rng.gen(), 0., rng.gen()) * 0.9);
            const BIG_BALL_CENTRE: Point3 = Point3 { x: 4., y: 0.2, z: 0. };

            if (centre - BIG_BALL_CENTRE).length() <= 1.0 {
                continue;
            }

            let material_choice = rng.gen::<Number>();
            let material: MaterialInstance = if material_choice < 0.6 {
                LambertianMaterial {
                    albedo: scene.add_tex(rng::colour_rgb(rng) * rng::colour_rgb(rng)),
                }
                .into()
            } else if material_choice <= 0.8 {
                MetalMaterial {
                    albedo: scene.add_tex(rng::colour_rgb_range(rng, 0.5..=1.0)),
                    fuzz: rng.gen_range(0.0..=0.5),
                }
                .into()
            } else if material_choice <= 0.95 {
                DielectricMaterial {
                    albedo: scene.add_tex(rng::colour_rgb_range(rng, 0.5..1.0)),
                    refractive_index: rng.gen_range(1.0..=10.0),
                    density: 69.0,
                }
                .into()
            } else {
                LightMaterial {
                    emissive: scene.add_tex(rng::colour_rgb_range(rng, 0.0..0.8)),
                }
                .into()
            };

            let obj_choice = rng.gen::<Number>();
            let obj: MeshInstance = if obj_choice < 0.7 {
                SphereMesh::new(centre, 0.2).into()
            } else {
                AxisBoxMesh::new_centred(centre, rng::vector_in_unit_cube_01(rng) * 0.8).into()
            };
            let obj = SimpleObject::new_in(&mut scene, obj, material, None);
            scene.add_obj(obj);
        }
    }

    // Lights
    for a in grid_dims.clone() {
        for b in grid_dims.clone() {
            let (a, b) = (a as Number, b as Number);

            let centre = Point3::new(a, 3.2, b) + (Vector3::new(rng.gen(), 0., rng.gen()) * 0.9);
            const BIG_BALL_CENTRE: Point3 = Point3 { x: 4., y: 0.2, z: 0. };

            if (centre - BIG_BALL_CENTRE).length() <= 1.8 {
                continue;
            }

            let material: MaterialInstance = LightMaterial {
                emissive: scene.add_tex(rng::colour_rgb_range(rng, 10.0..50.0)),
            }
            .into();

            let obj_choice = rng.gen::<Number>();
            let obj: MeshInstance = if obj_choice <= 0.99 {
                continue;
            } else if obj_choice <= 0.995 {
                SphereMesh::new(centre, 0.2).into()
            } else {
                AxisBoxMesh::new_centred(centre, rng::vector_in_unit_cube_01(rng) * 0.8).into()
            };
            let obj = SimpleObject::new_in(&mut scene, obj, material, None);
            scene.add_obj(obj);
        }
    }

    let mesh = scene.add_mesh(SphereMesh::new((0., 1., 0.), 1.));
    let tex = scene.add_tex([1.; 3]);
    let mat = scene.add_mat(DielectricMaterial {
        refractive_index: 1.5,
        density: 69.0,
        albedo: tex,
    });
    let obj = SimpleObject::new_from(&scene, mesh, mat, None);
    scene.add_obj(obj);
    let mesh = scene.add_mesh(SphereMesh::new((-4., 1., 0.), 1.));
    let tex = scene.add_tex([0.4, 0.2, 0.1]);
    let mat = scene.add_mat(LambertianMaterial { albedo: tex });
    let obj = SimpleObject::new_from(&scene, mesh, mat, None);
    scene.add_obj(obj);
    let mesh = scene.add_mesh(SphereMesh::new((4., 1., 0.), 1.));
    let tex = scene.add_tex([0.7, 0.6, 0.5]);
    let mat = scene.add_mat(MetalMaterial { albedo: tex, fuzz: 0. });
    let obj = SimpleObject::new_from(&scene, mesh, mat, None);
    scene.add_obj(obj);

    let mesh = scene.add_mesh(InfinitePlaneMesh::new(
        Plane::new(Point3::ZERO, Vector3::X, Vector3::Z),
        UvWrappingMode::Wrap,
    ));
    let noise = scene.add_noise3(BoxedNoise::from(
        noise::ScalePoint::new(noise::Perlin::new(69u32)).set_scale(10000.),
    ));
    let tex = scene.add_tex(NoiseTexture::Monochrome {
        noise: NoiseSource::LocalPos(noise),
        colour: Colour::WHITE,
    });
    let mat = scene.add_mat(LambertianMaterial { albedo: tex });
    let obj = SimpleObject::new_from(&scene, mesh, mat, None);
    scene.add_obj(obj);

    PresetScene {
        name: "RTIAW Demo (Night)",
        camera: Camera {
            pos: Point3::new(13., 2., 3.),
            fwd: Vector3::new(-13., -2., -3.).normalize(),
            v_fov: Angle::from_degrees(20.),
            focus_dist: 10.,
            defocus_angle: Angle::from_degrees(0.6),
        },
        scene,
    }
}

/// From **RayTracing The Next Week**, the demo scene at the end of the chapter (extended of course)
pub fn RTTNW_DEMO() -> PresetScene {
    let mut scene = Scene::new();
    scene.set_skybox(None);
    let rng = &mut rand::thread_rng();

    {
        // BOXES (FLOOR)
        const COUNT: usize = 20;
        const HALF_COUNT: Number = COUNT as Number / 2.;
        const WIDTH: Number = 1.;

        // NOTE: Only need to use one material, shared across all boxes
        let tex = scene.add_tex([0.48, 0.83, 0.53]);
        let mat = scene.add_mat(LambertianMaterial { albedo: tex });

        let mut floor = vec![];
        for i in 0..COUNT {
            for j in 0..COUNT {
                let low = Point3::new(-HALF_COUNT * WIDTH, 0., -HALF_COUNT * WIDTH)
                    + Vector3::new(i as Number * WIDTH, 0., j as Number * WIDTH);
                let high = low + Vector3::new(WIDTH, rng.gen_range(0.0..=1.0), WIDTH);

                floor.push((AxisBoxMesh::new(low, high)));
            }
        }

        let mesh = ListMesh::new_in(&mut scene, floor);
        let mesh = scene.add_mesh(mesh);
        scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, None));
    }

    {
        // LIGHT
        let mesh = scene.add_mesh(ParallelogramMesh::new(Plane::new(
            (1.23, 5.54, 1.47),
            (3., 0., 0.),
            (0., 0., 2.65),
        )));
        let tex = scene.add_tex([7.; 3]);
        let mat = scene.add_mat(LightMaterial { emissive: tex });
        scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, None));
    }

    {
        // BROWN SPHERE
        let mesh = scene.add_mesh(SphereMesh::new((4., 4., 2.), 0.5));
        let tex = scene.add_tex([0.7, 0.3, 0.1]);
        let mat = scene.add_mat(LambertianMaterial { albedo: tex });
        scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, None));

        // GLASS SPHERE
        let mesh = scene.add_mesh(SphereMesh::new((2.6, 1.5, 0.45), 0.5));
        let tex = scene.add_tex([1.; 3]);
        let mat = scene.add_mat(DielectricMaterial {
            albedo: tex,
            density: 1.0,
            refractive_index: 1.5,
        });
        scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, None));

        // METAL SPHERE (RIGHT)
        let mesh = scene.add_mesh(SphereMesh::new((0., 1.5, 1.45), 0.5));
        let tex = scene.add_tex([0.8, 0.8, 0.9]);
        let mat = scene.add_mat(MetalMaterial { albedo: tex, fuzz: 1. });
        scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, None));

        // SUBSURFACE SCATTER BLUE SPHERE (LEFT)
        let mesh = scene.add_mesh(SphereMesh::new((3.6, 1.5, 1.45), 0.7));
        let tex = scene.add_tex([1.; 3]);
        let mat = scene.add_mat(DielectricMaterial {
            albedo: tex,
            refractive_index: 1.5,
            density: 0.0,
        });
        scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, None));
        // BLUE HAZE INSIDE
        let mesh = scene.add_mesh(SphereMesh::new((3.6, 1.5, 1.45), 0.6999));
        let tex = scene.add_tex([0.2, 0.4, 0.9]);
        let mat = scene.add_mat(IsotropicMaterial {
            albedo: tex,
            density: 0.3,
        });
        scene.add_obj(VolumetricObject::new_from(&scene, mesh, mat, 2.0, None));

        // EARTH SPHERE
        let mesh = scene.add_mesh(SphereMesh::new((4., 2., 4.), 1.0));
        // let tex = scene.add_tex(ImageTexture::from(Image::from(
        //     image::load_from_memory(include_bytes!("../../../media/texture/nasa-earthmap/5400x2700.jpg"))
        //         .expect("compile-time image resource should be valid"),
        // )));
        let tex = scene.add_tex(ImageTexture::from(Image::from_fn(2, 2, |x, y| {
            [[(0., 1., 0.), (0., 0., 1.)], [(1., 1., 1.), (0., 0., 0.)]][x][y].into()
        })));
        let mat = scene.add_mat(LambertianMaterial { albedo: tex });
        scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, None));

        // NOISE SPHERE
        let noise = scene.add_noise3(BoxedNoise::from(
            noise::ScalePoint::new(noise::Perlin::new(69)).set_scale(4.),
        ));
        let tex = scene.add_tex(NoiseTexture::Monochrome {
            noise: NoiseSource::WorldPos(noise),
            colour: Colour::WHITE,
        });
        let mat = scene.add_mat(LambertianMaterial { albedo: tex });
        let mesh = scene.add_mesh(SphereMesh::new((2.2, 2.8, 3.0), 0.8));
        scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, None));
    }

    {
        // CUBE OF BALLS

        const COUNT: usize = 1000;
        const SPREAD: Number = 0.825;
        const SIZE: Number = 0.1;

        let balls = (0..COUNT)
            .into_iter()
            .map(|_| SphereMesh::new((rng::vector_in_unit_cube(rng) * SPREAD).to_point(), SIZE));

        let tex = scene.add_tex([0.85; 3]);
        let mat = scene.add_mat(LambertianMaterial { albedo: tex });
        let mesh = ListMesh::new_in(&mut scene, balls);
        let mesh = scene.add_mesh(mesh);
        let trans = Transform3::from_scale_rotation_translation(
            Vector3::ONE,
            Vector3::Y,
            Angle::from_degrees(15.),
            // The original cube was not centred at middle, but "centred" at the corner
            Vector3::new(-1.0, 2.7, 3.95) + Vector3::splat(SPREAD),
        );
        scene.add_obj(SimpleObject::new_from(&scene, mesh, mat, trans));
    }

    {
        // HAZE

        let mesh = scene.add_mesh(SphereMesh::new(Point3::ZERO, 50.));
        let tex = scene.add_tex([1.; 3]);
        let mat = scene.add_mat(IsotropicMaterial {
            albedo: tex,
            density: 0.003,
        });
        scene.add_obj(VolumetricObject::new_from(&scene, mesh, mat, 0.003, None));
    }

    PresetScene {
        name: "RTTNW Demo",
        camera: Camera {
            pos: Point3::new(4.78, 2.78, -6.0),
            fwd: Vector3::new(-1., 0., 3.).normalize(),
            v_fov: Angle::from_degrees(40.),
            focus_dist: 1.,
            defocus_angle: Angle::from_degrees(0.0),
        },
        scene,
    }
}

/// The classic cornell box scene
pub fn CORNELL() -> PresetScene {
    let mut scene = Scene::new();
    scene.set_skybox(None);

    let red = scene.add_tex([0.65, 0.05, 0.05]);
    let green = scene.add_tex([0.12, 0.45, 0.15]);
    let warm_grey = scene.add_tex([0.85, 0.74, 0.55]);
    let light = scene.add_tex([15.; 3]);

    {
        // WALLS

        fn quad(
            scene: &mut Scene,
            p: impl Into<Point3>,
            u: impl Into<Vector3>,
            v: impl Into<Vector3>,
            albedo: TextureToken,
        ) {
            let obj = SimpleObject::new_in(
                scene,
                ParallelogramMesh::new(Plane::new(p, u, v)),
                LambertianMaterial { albedo },
                None,
            );
            scene.add_obj(obj);
        }
        quad(&mut scene, (0., 0., 0.), Vector3::Y, Vector3::Z, red); // Left
        quad(&mut scene, (0., 0., 0.), Vector3::X, Vector3::Y, warm_grey); // Back
        quad(&mut scene, (0., 0., 0.), Vector3::Z, Vector3::X, warm_grey); // Floor
        quad(&mut scene, (1., 0., 0.), Vector3::Z, Vector3::Y, green); // Right
        quad(&mut scene, (0., 1., 0.), Vector3::X, Vector3::Z, warm_grey); // Ceiling
    }

    {
        // LIGHT
        let mesh = ParallelogramMesh::new(Plane::new((0.4, 0.9999, 0.4), (0.2, 0., 0.), (0., 0., 0.2)));
        let mat = LightMaterial { emissive: light };
        let obj = SimpleObject::new_in(&mut scene, mesh, mat, None);
        scene.add_obj(obj);
    }

    {
        // INNER BOXES

        // Big
        let obj = SimpleObject::new_in(
            &mut scene,
            AxisBoxMesh::new((0.231, 0., 0.117), (0.531, 0.595, 0.414)),
            LambertianMaterial { albedo: warm_grey },
            Transform3::from_axis_angle(Vector3::Y, Angle::from_degrees(15.)),
        );
        scene.add_obj(obj);
        // Small
        let obj = SimpleObject::new_in(
            &mut scene,
            AxisBoxMesh::new((0.477, 0., 0.531), (0.774, 0.297, 0.829)),
            LambertianMaterial { albedo: warm_grey },
            Transform3::from_axis_angle(Vector3::Y, Angle::from_degrees(-18.)),
        );
        scene.add_obj(obj);
    }

    PresetScene {
        name: "Cornell Box",
        camera: Camera {
            pos: Point3::new(0.5, 0.5, 2.3),
            fwd: Vector3::new(0., 0., -1.).normalize(),
            v_fov: Angle::from_degrees(40.),
            focus_dist: 1.,
            defocus_angle: Angle::from_degrees(0.),
        },
        scene,
    }
}
