use crate::def::types::{ImgBuf, Pix};
use crate::render::render_opts::RenderOpts;
use crate::scene::Scene;

pub fn render(scene: &Scene, render_opts: RenderOpts) -> ImgBuf {
    let [w, h] = render_opts.dims_u32_slice();

    let mut img = ImgBuf::new(w, h);

    img.enumerate_pixels_mut().for_each(|(x, y, p)| {
        *p = if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
            Pix::from([1.0; 3])
        } else {
            Pix::from([0.0, 1.0, 0.0])
        }
    });

    img
}
