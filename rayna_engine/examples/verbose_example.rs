use std::io::Write;

// NOTE: Compatibility thing because of version differences
//   between `image` that the engine uses and that `viuer` uses
extern crate image_viuer_compat as image;

pub fn main() {
    // Type aliases used everywhere in the engine
    use rayna_engine::core::types::*;

    // region CREATING THE CAMERA

    // Self-explanatory
    use rayna_engine::scene::camera::Camera;

    let camera_pos = Point3::new(0.0, 1.0, -3.0); // Position the camera up slightly, and back a bit
    let camera_target = Point3::ZERO; // Point it towards the origin
    let camera = Camera {
        pos: camera_pos,
        v_fov: Angle::from_degrees(25.0), // Realtively standard FOV
        fwd: (camera_target - camera_pos).normalize(),
        focus_dist: (camera_target - camera_pos).length(),
        defocus_angle: Angle::from_degrees(15.0), // Add a large amount of defocus blur
    };

    // endregion CREATING THE CAMERA

    // region CREATING THE SCENE

    // The scene struct is generic, use `StandardScene` most of the time
    // as it's the easiest, and generally most performant
    use rayna_engine::scene::StandardScene;
    // Standard enum types, that wrap the other types into one, for static-dispatch.
    // So you can accept only one type `MeshInstance`, but it actually accepts
    // spheres, boxes, triangles, etc
    use rayna_engine::{
        material::MaterialInstance, mesh::MeshInstance, object::ObjectInstance, skybox::SkyboxInstance,
        texture::TextureInstance,
    };
    // Types we use in this example
    use rayna_engine::{
        material::lambertian::LambertianMaterial, mesh::primitive::sphere::SphereMesh, object::simple::SimpleObject,
        object::transform::ObjectTransform,
    };

    let skybox = rayna_engine::skybox::simple::WhiteSkybox;

    // Note all the specified types here, normally this is implicit via the compiler
    let mut objects = Vec::<ObjectInstance<MeshInstance, MaterialInstance<TextureInstance>>>::new();

    objects.push(
        SimpleObject::new(
            // NOTE: Using a tuple for sphere pos, since it's `impl Into<Point3>`
            SphereMesh::new((1., 0., 0.), 0.95),
            LambertianMaterial {
                albedo: Colour::RED.into(),
            },
            ObjectTransform::IDENTITY,
        )
        // NOTE: the call to `.into()` at the end, to convert it to `ObjectInstance`
        .into(),
    );
    objects.push(
        SimpleObject::new(
            // NOTE: Using an array for sphere pos, since it's `impl Into<Point3>`
            SphereMesh::new([-1., 0., 0.], 0.95),
            LambertianMaterial {
                albedo: TextureInstance::from(Colour::WHITE * 0.5),
            },
            // NOTE: Can use `Option::None` for the identity transform
            // `Option::Some(transform)` also works
            None,
        )
        .into(),
    );
    objects.push(
        SimpleObject::new(
            // NOTE: Using an array for sphere pos, since it's `impl Into<Point3>`
            SphereMesh::new([0., -0.2, -0.6], 0.2),
            LambertianMaterial {
                albedo: TextureInstance::from([0.2, 0.4, 1.0]),
            },
            None,
        )
        .into(),
    );

    let scene = StandardScene {
        objects: ObjectInstance::from(objects), // or `objects.into()`
        skybox: SkyboxInstance::from(skybox),   // or `skybox.into()`
    };

    // endregion CREATING THE SCENE

    // region RENDERING

    // `Renderer` is the main struct that does the rendering
    use rayna_engine::render::renderer::Renderer;
    // These two control how the image is rendered
    use rayna_engine::render::render_opts::{RenderMode, RenderOpts};

    let render_options = RenderOpts {
        width: nonzero::nonzero!(400_usize),       // Image Dimensions
        height: nonzero::nonzero!(400_usize),      // Image Dimensions
        samples: nonzero::nonzero!(1_usize),       // Sample each pixel multiple times
        mode: RenderMode::PBR,                     // Make normal renders
        ray_depth: 3,                              // Bounce three times
        ray_branching: nonzero::nonzero!(1_usize), // Ignore this; advanced and probably useless
    };
    // Have to provide the renderer with an RNG source, which must be seedable and thread-safe
    type Rng = rand::rngs::SmallRng;
    let mut renderer = Renderer::<_, _, Rng>::new_from(scene, camera, render_options, 2).unwrap();

    // Render a single image, without accumulation (since it's the first render)
    print!("rendering a single image...");
    std::io::stdout().flush().unwrap();
    let render_single = renderer.render();
    println!("done");

    // Mark it as dirty so that it resets accumulation
    // This is automatically called when changing settings or scenes
    renderer.clear_accumulation();
    // Accumulate multiple frames
    print!("rendering an accumulated image...");
    std::io::stdout().flush().unwrap();
    let render_accum = (0..50).into_iter().map(|_| renderer.render()).last().unwrap();
    println!("done");

    // The render contains both the image and the stats for the render
    // Currently, stats are only for the last frame though, not accumulated duration
    println!("single render stats:      {:?}", render_single.stats);
    println!("accumulated render stats: {:?}", render_accum.stats);
    let image_single = render_single.img;
    let image_accum = render_accum.img;

    // endregion RENDERING

    // region SAVING IMAGES

    // If you want to use the renders, you have to convert into that format
    // Here, we use the `image` crate's format, so we can save them to disk
    // and then display them

    let mut output_single = image::RgbImage::new(render_options.width.get() as u32, render_options.height.get() as u32);
    let mut output_accum = image::RgbImage::new(render_options.width.get() as u32, render_options.height.get() as u32);

    // Loop over each pixel, converting and storing into the output
    image_single
        .indexed_iter()
        .for_each(|((x, y), col)| output_single[(x as u32, y as u32)] = image::Rgb(col.0.map(|c| (c * 255.0) as u8)));

    image_accum
        .indexed_iter()
        .for_each(|((x, y), col)| output_accum[(x as u32, y as u32)] = image::Rgb(col.0.map(|c| (c * 255.0) as u8)));

    println!("note how the accumulated image is much less noisy than the single frame");
    println!("note how there is defocus blur applied to the blue ball");

    // Save to disk and open
    let output_dir = tempfile::tempdir().unwrap();
    let path_single = output_dir.path().join("single.png");
    let path_accum = output_dir.path().join("accum.png");
    output_single.save(&path_single).expect("failed to save image");
    output_accum.save(&path_accum).expect("failed to save image");
    opener::open(&path_single).ok();
    opener::open(&path_accum).ok();

    println!("press enter to print the images to the terminal...");
    std::io::stdin().lines().next();
    viuer::print(&image::DynamicImage::from(output_single), &Default::default()).expect("failed to print image");
    viuer::print(&image::DynamicImage::from(output_accum), &Default::default()).expect("failed to print image");

    std::thread::sleep(std::time::Duration::from_secs(2));
    // endregion SAVING IMAGES
}
