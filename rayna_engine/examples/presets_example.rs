use nonzero::nonzero;
use rayna_engine::render::render_opts::RenderMode;
use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::render::renderer::Renderer;

pub fn main() {
    let all_scenes = rayna_engine::scene::preset::ALL();

    println!("Available scenes are:");
    all_scenes
        .iter()
        .enumerate()
        .for_each(|(i, s)| println!("{i}:\t{name}", name = s.name));
    let sel = loop {
        println!("Choose a scene to render: ");
        let mut str = String::new();
        std::io::stdin().read_line(&mut str).expect("failed to read stdin");
        let Ok(i) = str.trim().parse::<usize>() else {
            println!("Invalid input: Could not parse ({str})");
            continue;
        };
        match all_scenes.get(i) {
            Some(scene) => {
                println!("Chose scene {}", scene.name);
                break scene.clone();
            }
            None => {
                println!("Invalid input: Out of range ({i})")
            }
        }
    };

    let mut renderer = Renderer::<rand::rngs::SmallRng>::new_from(
        sel.scene,
        sel.camera,
        RenderOpts {
            width: nonzero!(1920_usize),
            height: nonzero!(1080_usize),
            samples: nonzero!(10_usize),
            mode: RenderMode::PBR,
            ray_depth: 10,
            ray_branching: nonzero!(1_usize),
        },
        0,
    )
    .expect("failed to create renderer");

    println!("Rendering...");
    let render = renderer.render();
    println!("Rendering Done");

    let mut img = image::RgbImage::new(render.img.width() as u32, render.img.height() as u32);

    // Loop over each pixel, converting and storing into the output
    render
        .img
        .indexed_iter()
        .for_each(|((x, y), col)| img[(x as u32, y as u32)] = image::Rgb(col.0.map(|c| (c * 255.0) as u8)));

    // Save to disk and open
    let output_dir = tempfile::tempdir().unwrap();
    let path = output_dir.path().join("accum.png");
    img.save(&path).expect("failed to save image");
    opener::open(&path).ok();
    println!("Press enter to exit");
    std::io::stdin().lines().next();
}
