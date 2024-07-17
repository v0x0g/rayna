// Type aliases used everywhere in the engine. Always import this
use rayna_engine::core::types::*;

use rayna_engine::scene::camera::Camera;
/// Creates a camera object, that controls where the image is rendered from.
///
/// See [Camera] for documentation for the fields a camera has.
pub fn create_camera() -> Camera {
    // Position the camera up slightly and backwards, relative to the origin
    let pos: Point3 = Point3::new(0.0, 1.0, -3.0);
    // We want our camera to point towards the origin
    let target: Point3 = Point3::ZERO;
    // Use vector maths to calculate the vector pointing from
    // the camera position towards the target point
    let fwd: Vector3 = (target - pos).normalize();
    // Choose a reasonably narrow view angle
    let v_fov: Angle = Angle::from_degrees(25.0);
    // All points this distance away will be in perfect focus.
    // Use the distance to our target point, so the target is in focus
    let focus_dist: Number = (target - pos).length();
    // Add a large amount of defocus blur (a strong DOF)
    let defocus_angle: Angle = Angle::from_degrees(15.0);

    let camera = Camera {
        pos,
        v_fov,
        fwd,
        focus_dist,
        defocus_angle,
    };

    return camera;
}

// The scene struct is generic, use `StandardScene` most of the time
// as it's the easiest, and generally most performant
use rayna_engine::scene::StandardScene;
// Standard enum types, that wrap the other types into one, for static-dispatch.
// So you can accept only one type (`MeshInstance`), but it actually accepts
// spheres, boxes, triangles, etc
use rayna_engine::{
    material::MaterialInstance, mesh::MeshInstance, object::ObjectInstance, skybox::SkyboxInstance,
    texture::TextureInstance,
};

/// Creates the scene that will be rendered
///
/// The scene contains a list of all of the objects that will be rendered, as well
/// as the skybox.
pub fn create_scene() -> StandardScene {
    // Specific types we use in this example
    use rayna_engine::{
        material::lambertian::LambertianMaterial, object::simple::SimpleObject, object::transform::ObjectTransform,
        skybox::simple::SimpleSkybox, texture::solid::SolidTexture,
    };

    // Choose the default skybox, looks reasonably good
    let skybox = SimpleSkybox;

    // Create a vector to hold our scene objects. We will convert this into a
    // proper type later.
    // Note all the specified types here, normally this is implicit via the compiler
    let mut objects = Vec::<ObjectInstance<MeshInstance, MaterialInstance<TextureInstance>>>::new();

    // Create an object and add it to the scene
    objects.push(
        SimpleObject::new(
            // Create a sphere for the mesh (shape) of the object
            // NOTE: Using a tuple for sphere pos, since it's `impl Into<Point3>`
            SphereMesh::new((1., 0., 0.), 0.95),
            // A lambertian material is a fancy way of saying a diffuse material,
            // such as a painted wall, or concrete. It wants a texture, which we can
            // create by converting a colour, which turns into a uniform texture
            LambertianMaterial {
                albedo: Colour::RED.into(),
            },
            // The identity transform does nothing, so it leaves the object as-is.
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
                // NOTE: Explicitly doing the `Colour -> TextureInstance` conversion here
                albedo: TextureInstance::from(SolidTexture::from(Colour::WHITE * 0.5)),
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
        // Convert our vec of objects into an `ObjectInstance`
        objects: ObjectInstance::from(objects), // or `objects.into()`
        skybox: SkyboxInstance::from(skybox),   // or `skybox.into()`
    };

    return scene;
}

// `Renderer` is the main struct that does the rendering
use rayna_engine::render::renderer::Renderer;
// These two control how the image is rendered
use rand::rngs::SmallRng;
// Specific types we use in this example
use rayna_engine::mesh::sphere::SphereMesh;
use rayna_engine::render::render_opts::{RenderMode, RenderOpts};

/// Here we create the renderer, using the scene and camera we created earlier.
/// Due to future-compatibility reasons, the renderer takes ownership of them.
///
/// # Note
/// Type parameters provided here, but normally are inferred by the compiler.
/// We use the [SmallRng] as the RNG source, although any RNG will work as long as it's
/// seedable ([rand::SeedableRng]) and thread-safe ([std::marker::Send])
pub fn create_renderer(
    scene: StandardScene,
    camera: Camera,
) -> Renderer<ObjectInstance<MeshInstance, MaterialInstance<TextureInstance>>, SkyboxInstance, SmallRng> {
    let render_options = RenderOpts {
        width: nonzero::nonzero!(200_usize),       // Image Dimensions
        height: nonzero::nonzero!(200_usize),      // Image Dimensions
        samples: nonzero::nonzero!(1_usize),       // Sample each pixel multiple times
        mode: RenderMode::PBR,                     // Make normal renders
        ray_depth: 3,                              // Bounce three times
        ray_branching: nonzero::nonzero!(1_usize), // Ignore this; advanced and probably useless
    };
    return Renderer::new_from(scene, camera, render_options, 2).unwrap();
}

/// Here we use the renderer to actually render the images
///
/// We don't need to take in the scene or camera, since they're already stored inside
/// the renderer. All we need to do is call [Renderer::render()]!!
pub fn do_renders<Obj, Sky, Rng>(mut renderer: Renderer<Obj, Sky, Rng>) -> (Image, Image)
where
    Obj: rayna_engine::object::Object,
    Sky: rayna_engine::skybox::Skybox,
    Rng: rand::RngCore + std::marker::Send + rand::SeedableRng,
{
    // Render a single image, without accumulation (since it's the first render)
    print!("rendering a single image...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let render_single = renderer.render();
    println!("done");

    // Mark it as dirty so that it resets accumulation
    // This is automatically called when changing settings or scenes
    // This is only for demonstration purposes - normally you want
    // as many frames accumulated as possible
    renderer.clear_accumulation();
    // Accumulate multiple frames
    print!("rendering an accumulated image...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let render_accum = (0..50).into_iter().map(|_| renderer.render()).last().unwrap();
    println!("done");

    // The render contains both the image and the stats for the render
    // Currently, stats are only for the last frame though, not accumulated duration
    let _ = render_single.stats;
    let _ = render_accum.stats;
    let image_single = render_single.img;
    let image_accum = render_accum.img;

    return (image_single, image_accum);
}

/// Takes in the renderer images, and shows them to the user
pub fn save_and_show_images((image_single, image_accum): (Image, Image)) {
    // NOTE: Compatibility thing because of version differences
    //  between `image` that the engine uses and that `viuer` uses.
    //  Only an issue for this example
    extern crate image_viuer_compat as image;

    // If you want to use the renders, you have to convert into that format
    // Here, we use the `image` crate's format, so we can save them to disk
    // and then display them

    let mut output_single = image::RgbImage::new(image_single.width() as u32, image_single.height() as u32);
    let mut output_accum = image::RgbImage::new(image_accum.width() as u32, image_accum.height() as u32);

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

    // Also print to the console, using ANSI code fanciness
    println!("press enter to print the images to the terminal...");
    std::io::stdin().lines().next();
    viuer::print(&image::DynamicImage::from(output_single), &Default::default()).expect("failed to print image");
    viuer::print(&image::DynamicImage::from(output_accum), &Default::default()).expect("failed to print image");
    std::thread::sleep(std::time::Duration::from_secs(2)); // Sleep to allow time to print
}

pub fn main() {
    // See each of the functions for how this works
    let scene: StandardScene = create_scene();
    let camera: Camera = create_camera();
    let renderer = create_renderer(scene, camera);
    let (image_single, image_accum) = do_renders(renderer);
    save_and_show_images((image_single, image_accum));
}
