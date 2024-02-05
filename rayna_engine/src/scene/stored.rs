//! This module is a repository of all builtin scenes in the engine
//!
//! There is no significance to them, apart from not having to manually create scenes by hand.
//!
//! There are some common ones [CORNELL] and [RTIAW_DEMO], that should be well known.
#![allow(unused)]
use crate::core::types::{Angle, Channel, Colour, Image, Number, Point3, Size3, Transform3, Vector3};
use crate::object::simple::SimpleObject;
use noise::*;
use rand::{thread_rng, Rng};
use static_init::*;

use crate::material::dielectric::DielectricMaterial;
use crate::material::isotropic::IsotropicMaterial;
use crate::material::lambertian::LambertianMaterial;
use crate::material::light::LightMaterial;
use crate::material::metal::MetalMaterial;
use crate::material::MaterialInstance;
use crate::mesh::advanced::bvh::BvhMesh;
use crate::mesh::isosurface::polygonised::PolygonisedIsosurfaceMesh;
use crate::mesh::isosurface::raymarched::RaymarchedIsosurfaceMesh;
use crate::mesh::planar::infinite_plane::{InfinitePlaneMesh, UvWrappingMode};
use crate::mesh::planar::parallelogram::ParallelogramMesh;
use crate::mesh::planar::Planar;
use crate::mesh::primitive::axis_box::AxisBoxMesh;
use crate::mesh::primitive::sphere::SphereMesh;
use crate::mesh::MeshInstance;
use crate::object::volumetric::VolumetricObject;
use crate::object::ObjectInstance;
use crate::scene::camera::Camera;
use crate::shared::math::Lerp;
use crate::shared::rng;
use crate::skybox::SkyboxInstance;
use crate::texture::image::ImageTexture;
use crate::texture::noise::{ColourSource, LocalNoiseTexture, WorldNoiseTexture};
use crate::texture::solid::SolidTexture;
use crate::texture::TextureInstance;

use super::Scene;

// /// Super simple scene, just a ground sphere and a small sphere
// #[dynamic]
// pub static SIMPLE: Scene = {
//     Scene {
//         camera: Camera {
//             pos: Point3::new(0., 0.5, -3.),
//             fwd: Vector3::Z,
//             v_fov: Angle::from_degrees(45.),
//             focus_dist: 3.,
//             defocus_angle: Angle::from_degrees(0.),
//         },
//         objects: [
//             SimpleObject::new(
//                 // Small, top
//                 SphereMesh::new((0., 0., 1.), 0.5),
//                 MetalMaterial {
//                     albedo: [0.8; 3].into(),
//                     fuzz: 1.,
//                 },
//                 None,
//             ),
//             SimpleObject::new(
//                 // Ground
//                 SphereMesh::new((0., -100.5, -1.), 100.),
//                 LambertianMaterial {
//                     albedo: [0.5; 3].into(),
//                 },
//                 None,
//             ),
//         ]
//         .into(),
//         skybox: SkyboxInstance::default(),
//     }
// };

#[dynamic]
pub static TESTING: Scene = {
    let camera = Camera {
        pos: Point3::new(0.5, 0.2, 0.9),
        fwd: Vector3::new(0., 0., -1.).normalize(),
        v_fov: Angle::from_degrees(75.),
        focus_dist: 1.,
        defocus_angle: Angle::from_degrees(0.),
    };

    let mut objects = Vec::new();

    {
        objects.push(SimpleObject::new(
            PolygonisedIsosurfaceMesh::new(64, |p_raw| {
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
            }),
            // DielectricMaterial {
            //     albedo: [0.68, 0.73, 0.8].into(),
            //     density: 4.0,
            //     refractive_index: 1.335,
            // },
            LambertianMaterial::default(),
            None,
        ));

        // let v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0]].map(Vector3::from);
        // let [vr, vg, vb, vw] = v;
        // let [vrg, vgb, vbr] = [(vr + vg) / 2., (vg + vb) / 2., (vb + vr) / 2.];
        // let [r, g, b, w] = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.], [1., 1., 1.]].map(Colour::from);
        // let [rg, gb, br] = [(r + g) / 2., (g + b) / 2., (b + r) / 2.];
        // let radius = 0.05;
        //
        // let mut sphere = |p: Vector3, c: Colour| {
        //     objects.push(SimpleObject::new(
        //         SphereMesh::new(p, radius),
        //         LambertianMaterial::from(TextureInstance::from(c)),
        //         None,
        //     ));
        // };
        //
        // sphere(Vector3::ZERO, Colour::BLACK);
        //
        // sphere(vr, r);
        // sphere(vg, g);
        // sphere(vb, b);
        //
        // sphere(vr / 2., r);
        // sphere(vg / 2., g);
        // sphere(vb / 2., b);
        //
        // sphere(vrg, rg);
        // sphere(vgb, gb);
        // sphere(vbr, br);
        //
        // sphere(vw, w);
    }

    Scene {
        camera,
        objects: objects.into(),
        skybox: Default::default(),
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
            let material: MaterialInstance<TextureInstance> = if material_choice < 0.7 {
                LambertianMaterial {
                    albedo: (rng::colour_rgb(rng) * rng::colour_rgb(rng)).into(),
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
                    density: 69.0,
                }
                .into()
            };

            let obj_choice = rng.gen::<Number>();
            let obj: MeshInstance = if obj_choice < 0.7 {
                SphereMesh::new(centre, 0.2).into()
            } else {
                AxisBoxMesh::new_centred(centre, rng::vector_in_unit_cube_01(rng) * 0.8).into()
            };
            objects.push(SimpleObject::new(obj, material, None));
        }
    }

    objects.push(SimpleObject::new(
        SphereMesh::new((0., 1., 0.), 1.),
        DielectricMaterial {
            refractive_index: 1.5,
            density: 69.0,
            albedo: [1.; 3].into(),
        },
        None,
    ));
    objects.push(SimpleObject::new(
        SphereMesh::new((-4., 1., 0.), 1.),
        LambertianMaterial {
            albedo: [0.4, 0.2, 0.1].into(),
        },
        None,
    ));
    objects.push(SimpleObject::new(
        SphereMesh::new((4., 1., 0.), 1.),
        MetalMaterial {
            albedo: [0.7, 0.6, 0.5].into(),
            fuzz: 0.,
        },
        None,
    ));

    objects.push(SimpleObject::new(
        SphereMesh::new((0., -1000., 0.), 1000.),
        LambertianMaterial {
            albedo: LocalNoiseTexture {
                source: ColourSource::Greyscale(ScalePoint::new(Perlin::new(69u32)).set_scale(10000.)).to_dyn_box(),
            }
            .into(),
        },
        None,
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
    }
};

//noinspection SpellCheckingInspection
/// From **RayTracing The Next Week**, the demo scene at the end of the chapter (extended of course)
#[dynamic]
pub static RTTNW_DEMO: Scene = {
    let mut objects: Vec<ObjectInstance<MeshInstance, MaterialInstance<TextureInstance>>> = Vec::new();
    let rng = &mut thread_rng();

    // Const trait impls aren't yet stabilised, so we can't call TextureInstance::From(Pixel) yet
    const fn solid_texture(albedo: [Channel; 3]) -> TextureInstance {
        let pixel = Colour { 0: albedo };
        let solid = SolidTexture { albedo: pixel };
        let texture = TextureInstance::SolidTexture(solid);
        texture
    }

    {
        // BOXES (FLOOR)
        const COUNT: usize = 20;
        const HALF_COUNT: Number = COUNT as Number / 2.;
        const WIDTH: Number = 1.;

        let mut floor: Vec<MeshInstance> = vec![];
        for i in 0..COUNT {
            for j in 0..COUNT {
                let low = Point3::new(-HALF_COUNT * WIDTH, 0., -HALF_COUNT * WIDTH)
                    + Vector3::new(i as Number * WIDTH, 0., j as Number * WIDTH);
                let high = low + Vector3::new(WIDTH, rng.gen_range(0.0..=1.0), WIDTH);

                floor.push(AxisBoxMesh::new(low, high).into());
            }
        }

        objects.push(
            SimpleObject::new(
                BvhMesh::new(floor),
                LambertianMaterial {
                    albedo: solid_texture([0.48, 0.83, 0.53]),
                },
                None,
            )
            .into(),
        );
    }

    {
        // LIGHT
        objects.push(
            SimpleObject::new(
                ParallelogramMesh::new(Planar::new((1.23, 5.54, 1.47), (3., 0., 0.), (0., 0., 2.65))),
                LightMaterial {
                    emissive: solid_texture([7.; 3]),
                },
                None,
            )
            .into(),
        );
    }

    {
        // BROWN SPHERE
        objects.push(
            SimpleObject::new(
                SphereMesh::new((4., 4., 2.), 0.5),
                LambertianMaterial {
                    albedo: solid_texture([0.7, 0.3, 0.1]),
                },
                None,
            )
            .into(),
        );

        // GLASS SPHERE
        objects.push(
            SimpleObject::new(
                SphereMesh::new((2.6, 1.5, 0.45), 0.5),
                DielectricMaterial {
                    albedo: [1.; 3].into(),
                    density: 1.0,
                    refractive_index: 1.5,
                },
                None,
            )
            .into(),
        );

        // METAL SPHERE (RIGHT)
        objects.push(
            SimpleObject::new(
                SphereMesh::new((0., 1.5, 1.45), 0.5),
                MetalMaterial {
                    albedo: [0.8, 0.8, 0.9].into(),
                    fuzz: 1.,
                },
                None,
            )
            .into(),
        );

        // SUBSURFACE SCATTER BLUE SPHERE (LEFT)
        objects.push(
            SimpleObject::new(
                SphereMesh::new((3.6, 1.5, 1.45), 0.7),
                DielectricMaterial {
                    albedo: [1.; 3].into(),
                    refractive_index: 1.5,
                    density: 0.0,
                },
                None,
            )
            .into(),
        );
        objects.push(
            VolumetricObject::new(
                // BLUE HAZE INSIDE
                SphereMesh::new((3.6, 1.5, 1.45), 0.6),
                IsotropicMaterial {
                    albedo: [0.2, 0.4, 0.9].into(),
                },
                1.0,
                None,
            )
            .into(),
        );

        // EARTH SPHERE
        objects.push(
            SimpleObject::new(
                SphereMesh::new((4., 2., 4.), 1.0),
                LambertianMaterial {
                    albedo: ImageTexture::from(Image::from(
                        image::load_from_memory(include_bytes!("../../../media/texture/nasa-earthmap/5400x2700.jpg"))
                            .expect("compile-time image resource should be valid"),
                    ))
                    .into(),
                },
                None,
            )
            .into(),
        );

        // NOISE SPHERE
        objects.push(
            SimpleObject::new(
                SphereMesh::new((2.2, 2.8, 3.0), 0.8),
                LambertianMaterial {
                    albedo: WorldNoiseTexture {
                        source: ColourSource::Greyscale(ScalePoint::new(Perlin::new(69)).set_scale(4.)).to_dyn_box(),
                    }
                    .into(),
                },
                None,
            )
            .into(),
        );
    }

    {
        // CUBE OF BALLS

        const COUNT: usize = 1000;
        const SPREAD: Number = 0.825;

        let balls = (0..COUNT)
            .into_iter()
            .map(|_| SphereMesh::new((rng::vector_in_unit_cube(rng) * SPREAD).to_point(), 0.1).into())
            .collect();

        objects.push(
            SimpleObject::new(
                BvhMesh::new(balls),
                LambertianMaterial {
                    albedo: solid_texture([0.85; 3]),
                },
                Transform3::from_scale_rotation_translation(
                    Vector3::ONE,
                    Vector3::Y,
                    Angle::from_degrees(15.),
                    // The original cube was not centred at middle, but "centred" at the corner
                    Vector3::new(-1.0, 2.7, 3.95) + Vector3::splat(SPREAD),
                ),
            )
            .into(),
        );
    }

    {
        // HAZE

        objects.push(
            VolumetricObject::new(
                SphereMesh::new(Point3::ZERO, 50.),
                IsotropicMaterial { albedo: [1.; 3].into() },
                0.003,
                None,
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
        objs: &mut Vec<SimpleObject<MeshInstance, MaterialInstance<TextureInstance>>>,
        p: impl Into<Point3>,
        u: impl Into<Vector3>,
        v: impl Into<Vector3>,
        albedo: impl Into<TextureInstance>,
    ) {
        objs.push(SimpleObject::new(
            ParallelogramMesh::new(Planar::new(p, u, v)),
            LambertianMaterial { albedo: albedo.into() },
            None,
        ));
    }

    let red = [0.65, 0.05, 0.05];
    let green = [0.12, 0.45, 0.15];
    let warm_grey = [0.85, 0.74, 0.55];
    let light = [15.; 3];

    let o = &mut objects;

    {
        // WALLS

        quad(o, (0., 0., 0.), Vector3::Y, Vector3::Z, red); // Left
        quad(o, (0., 0., 0.), Vector3::X, Vector3::Y, warm_grey); // Back
        quad(o, (0., 0., 0.), Vector3::Z, Vector3::X, warm_grey); // Floor
        quad(o, (1., 0., 0.), Vector3::Z, Vector3::Y, green); // Right
        quad(o, (0., 1., 0.), Vector3::X, Vector3::Z, warm_grey); // Ceiling

        o.push(SimpleObject::new(
            ParallelogramMesh::new(Planar::new((0.4, 0.9999, 0.4), (0.2, 0., 0.), (0., 0., 0.2))),
            LightMaterial { emissive: light.into() },
            None,
        ));
    }

    {
        // INNER BOXES

        // Big
        o.push(SimpleObject::new(
            AxisBoxMesh::new((0.231, 0., 0.117), (0.531, 0.595, 0.414)),
            LambertianMaterial {
                albedo: warm_grey.into(),
            },
            Transform3::from_axis_angle(Vector3::Y, Angle::from_degrees(15.)),
        ));
        // Small
        o.push(SimpleObject::new(
            AxisBoxMesh::new((0.477, 0., 0.531), (0.774, 0.297, 0.829)),
            LambertianMaterial {
                albedo: warm_grey.into(),
            },
            Transform3::from_axis_angle(Vector3::Y, Angle::from_degrees(-18.)),
        ));
    }

    Scene {
        camera,
        objects: objects.into(),
        skybox: None.into(),
        // skybox: SkyboxInstance::default(),
    }
};
