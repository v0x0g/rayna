use crate::def::targets::*;
use crate::def::types::{ImgBuf, Pix};
use crate::render::render_opts::RenderOpts;
use crate::shared::camera::Viewport;
use crate::shared::math;
use crate::shared::scene::Scene;
use tracing::trace;

fn render_failed_image(w: u32, h: u32) -> ImgBuf {
    ImgBuf::from_fn(w, h, |x, y| {
        Pix::from({
            if (x + y) % 2 == 0 {
                [0., 0., 0.]
            } else {
                [1., 0., 1.]
            }
        })
    })
}

pub fn render(scene: &Scene, render_opts: RenderOpts) -> ImgBuf {
    let [w, h] = render_opts.dims_u32_slice();

    let mut img = ImgBuf::new(w, h);

    let viewport = match scene.camera.calculate_viewport(render_opts) {
        Ok(v) => v,
        Err(err) => {
            trace!(target: RENDERER, ?err, "couldn't calculate viewport");
            return render_failed_image(w, h);
        }
    };

    img.enumerate_pixels_mut()
        .for_each(|(x, y, p)| *p = render_pixel_once(scene, viewport, x as usize, y as usize));

    img
}

/// Renders a single pixel in the scene, and returns the colour
fn render_pixel_once(scene: &Scene, viewport: Viewport, x: usize, y: usize) -> Pix {
    let ray = viewport.calculate_pixel_ray(x, y);

    let a = (0.5 * ray.dir().y) + 0.5;

    let white = Pix::from([1., 1., 1.]);
    let blue = Pix::from([0.5, 0.7, 1.]);

    math::lerp(white, blue, a)
}
