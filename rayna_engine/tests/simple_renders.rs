use approx::assert_relative_eq;
use rand::thread_rng;
use rayna_engine::core::colour::ColourRgb;
use rayna_engine::core::types::*;
use rayna_engine::material::lambertian::LambertianMaterial;
use rayna_engine::mesh::primitive::sphere::SphereMesh;
use rayna_engine::object::simple::SimpleObject;
use rayna_engine::scene::camera::Camera;
use rayna_engine::scene::StandardScene;
use rayna_engine::shared::rng;
use rayna_engine::skybox::simple::WhiteSkybox;

mod common;

/// This test makes some renders with spheres of different colours in a white background,
/// and counts the pixels of the rendered image.
///
/// It tests renders at multiple positions, where the approximate pixel colours should be known.
/// For example, inside the sphere should be all black, and pointing at the sky should be all white.
///
/// To ensure a bit more rigour, colours are randomly generated.
#[test]
pub fn sphere_colours() {
    const THRESHOLD: Channel = 0.1;
    const COUNT: usize = 10;
    // Generate mid-range colours, so they can't be confused with black or white
    let gen_colour = || rng::colour_rgb_range(&mut thread_rng(), (0.01 + THRESHOLD)..(0.99 - THRESHOLD));

    std::iter::repeat_with(gen_colour)
        .take(COUNT)
        .for_each(|col| sphere_colours_internal(col, THRESHOLD));
}

/// Internal implementation for [sphere_colours()]
fn sphere_colours_internal(target_col: ColourRgb, thresh: Channel) {
    let scene = StandardScene {
        objects: SimpleObject::new_uncorrected(
            SphereMesh::new(Point3::ZERO, 1.0),
            LambertianMaterial {
                albedo: target_col.into(),
            },
            None,
        )
        .into(),
        skybox: WhiteSkybox.into(),
    };
    let camera = Camera {
        pos: Point3::ZERO,
        v_fov: Angle::from_degrees(45.),
        fwd: Vector3::new(0., 0., 1.),
        focus_dist: 1.,
        defocus_angle: Angle::from_degrees(0.),
    };

    let colours_eq = |px: ColourRgb, target: ColourRgb, thresh: Channel| -> bool {
        let diff = (target - px).abs();
        return diff.into_iter().all(|c| c < thresh);
    };

    let count_colours = |img: &Image| -> [Number; 4] {
        // Count the number of pixels of each colour
        let mut colour = 0;
        let mut white = 0;
        let mut black = 0;
        let mut other = 0;
        let mut total = 0;

        for px in img.iter().copied() {
            if colours_eq(px, ColourRgb::WHITE, thresh) {
                white += 1;
            } else if colours_eq(px, ColourRgb::BLACK, thresh) {
                black += 1;
            } else if colours_eq(px, target_col, thresh) {
                colour += 1;
            } else {
                other += 1;
            }
            total += 1;
        }

        return [colour, white, black, other].map(|c| c as Number / total as Number);
    };

    println!("current colour is: {target_col:?}");

    println!("rendering inside sphere, expecting all black...");
    let img = common::render_simple(
        scene.clone(),
        Camera {
            pos: (0., 0., 0.).into(),
            ..camera
        },
    );
    let [colour, white, black, other] = count_colours(&img);
    assert_relative_eq!(colour, 0.);
    assert_relative_eq!(white, 0.);
    assert_relative_eq!(black, 1.);
    assert_relative_eq!(other, 0.);

    println!("rendering just next to sphere, expecting all colour...");
    let img = common::render_simple(
        scene.clone(),
        Camera {
            pos: (0., 0., -1.01).into(),
            ..camera
        },
    );

    let [colour, white, black, other] = count_colours(&img);
    assert_relative_eq!(colour, 1.);
    assert_relative_eq!(white, 0.);
    assert_relative_eq!(black, 0.);
    assert_relative_eq!(other, 0.);

    println!("rendering the sky, expecting all white...");
    let img = common::render_simple(
        scene.clone(),
        Camera {
            pos: (0., 0., 1.01).into(),
            ..camera
        },
    );
    let [colour, white, black, other] = count_colours(&img);
    assert_relative_eq!(colour, 0.);
    assert_relative_eq!(white, 1.);
    assert_relative_eq!(black, 0.);
    assert_relative_eq!(other, 0.);

    println!("rendering near sphere, expecting mostly colour...");
    let img = common::render_simple(
        scene.clone(),
        Camera {
            pos: (0., 0., -2.7).into(),
            ..camera
        },
    );
    let [colour, white, black, other] = count_colours(&img);
    assert!(colour >= 0.6, "{colour}");
    assert!(0.2 <= white && white <= 0.4, "{white}");
    assert!(other <= 0.05, "{other}");
    assert_relative_eq!(black, 0.);
}
