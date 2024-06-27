use nonzero::nonzero;
use rayna_engine::core::types::*;
use rayna_engine::object::Object;
use rayna_engine::render::{
    render_opts::{RenderMode, RenderOpts},
    renderer::Renderer,
};
use rayna_engine::scene::{camera::Camera, Scene};
use rayna_engine::skybox::Skybox;

pub type Rng = rand::rngs::SmallRng;

pub const SIMPLE_RENDER_OPTIONS: RenderOpts = RenderOpts {
    width: nonzero!(320_usize),
    height: nonzero!(320_usize),
    samples: nonzero!(10_usize),
    mode: RenderMode::PBR,
    ray_depth: 5,
    ray_branching: nonzero!(1_usize),
};

pub const RENDERER_THREAD_COUNT: usize = 4;

/// Quick and dirty renders the scene
pub fn render_simple<Obj: Object, Sky: Skybox>(scene: Scene<Obj, Sky>, camera: Camera) -> Image {
    let mut rend = Renderer::<Obj, Sky, Rng>::new_from(scene, camera, SIMPLE_RENDER_OPTIONS, RENDERER_THREAD_COUNT)
        .expect("failed creating renderer");
    rend.render().img
}
