use crate::def::types::{ImgBuf, Pix};
use crate::render::render_opts::RenderOpts;
use crate::shared::camera::Viewport;
use crate::shared::math;
use crate::shared::scene::Scene;

pub fn render(scene: &Scene, render_opts: RenderOpts) -> ImgBuf {
    let [w, h] = render_opts.dims_u32_slice();

    let mut img = ImgBuf::new(w, h);

    let viewport = scene.camera.calculate_viewport(render_opts);

    img.enumerate_pixels_mut()
        .for_each(|(x, y, p)| *p = render_pixel_once(scene, viewport, x as usize, y as usize));

    img
}

/// Renders a single pixel in the scene, and returns the colour
fn render_pixel_once(scene: &Scene, viewport: Viewport, x: usize, y: usize) -> Pix {
    let ray = viewport.calculate_pixel_ray(x, y);

    let a = 0.5 * ray.dir().y + 0.5;

    let white = Pix::from([1.; 3]);
    let blue = Pix::from([0., 0., 1.]);

    math::lerp(white, blue, a)
}
