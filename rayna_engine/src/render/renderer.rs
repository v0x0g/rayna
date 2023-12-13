use crate::def::targets::*;
use crate::def::types::{ImgBuf, Pix};
use crate::render::render_opts::RenderOpts;
use crate::shared::camera::Viewport;
use crate::shared::math;
use crate::shared::scene::Scene;
use image::{FlatSamples, Pixel};
use puffin::profile_scope;
use rayon::prelude::*;
use tracing::trace;

#[memoize::memoize(Capacity: 8)] // Keep cap small since images can be huge
fn render_failed_image(w: u32, h: u32) -> ImgBuf {
    puffin::profile_function!();

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
    puffin::profile_function!();

    let [w, h] = render_opts.dims_u32_slice();

    let mut img = ImgBuf::new(w, h);

    let viewport = match scene.camera.calculate_viewport(render_opts) {
        Ok(v) => v,
        Err(err) => {
            trace!(target: RENDERER, ?err, "couldn't calculate viewport");
            return render_failed_image(w, h);
        }
    };

    {
        profile_scope!("flat_samples");
        let samples = img.as_flat_samples_mut();
        for y in 0..samples.layout.height as usize {
            for x in 0..samples.layout.width as usize {
                let w = x * samples.layout.width_stride;
                let h = y * samples.layout.height_stride;

                // Guaranteed channels are contiguous, so we can slice
                let p_mut = Pix::from_slice_mut(&mut samples.samples[w + h..w + h + 3]);
                *p_mut = render_px(scene, viewport, x, y);
            }
        }
    }
    // {
    //     profile_scope!("flat_samples_par");
    //     let FlatSamples {
    //         samples, layout, ..
    //     } = img.as_flat_samples_mut();
    //
    //     // SAFETY:
    //     // `img.as_flat_samples_mut()` guarantees that channels are neighbours:
    //     // > The returned buffer is guaranteed to be well formed in all cases.
    //     // > It is laid out by colors, width then height, meaning
    //     // > `channel_stride <= width_stride <= height_stride`
    //     // Therefore this is safe since each pixel is [f32; 3] internally
    //     let samples = unsafe { std::mem::transmute::<&mut [f32], &mut [Pix]>(samples) };
    //
    //     let w = layout.width as usize;
    //     let h = layout.height as usize;
    //
    //     let px_iter = (0..w * h)
    //         .into_par_iter()
    //         .map(|pos| (/*x*/ pos % w, /*y*/ pos / w));
    //     px_iter.for_each(|(x, y)| {
    //         // Guaranteed channels are contiguous, so we can slice
    //         let p_mut = &mut samples[x * layout.width_stride + y * layout.height_stride];
    //         *p_mut = render_px(scene, viewport, x, y);
    //     });
    // }

    {
        profile_scope!("flat_samples_par");
        let FlatSamples {
            samples, layout, ..
        } = img.as_flat_samples_mut();

        // SAFETY:
        // `img.as_flat_samples_mut()` guarantees that channels are neighbours:
        // > The returned buffer is guaranteed to be well formed in all cases.
        // > It is laid out by colors, width then height, meaning
        // > `channel_stride <= width_stride <= height_stride`
        // Therefore this is safe since each pixel is [f32; 3] internally
        // let samples = unsafe { std::mem::transmute::<&mut [f32], &mut [Pix]>(samples) };

        let w = layout.width as usize;
        let h = layout.height as usize;

        samples
            .par_chunks_exact_mut(layout.channels as usize)
            .enumerate()
            .map(|(pos, chan_slice)| (/*x*/ pos % w, /*y*/ pos / w, chan_slice))
            .for_each(|(x, y, chan_slice)| {
                // Guaranteed channels are contiguous, so we can slice
                let p_mut = Pix::from_slice_mut(chan_slice);
                *p_mut = render_px(scene, viewport, x, y);
            });
    }

    {
        profile_scope!("enumerate_par");
        img.enumerate_pixels_mut()
            .par_bridge()
            .for_each(|(x, y, p)| *p = render_px(scene, viewport, x as usize, y as usize));
    }
    {
        profile_scope!("enumerate");
        img.enumerate_pixels_mut()
            .for_each(|(x, y, p)| *p = render_px(scene, viewport, x as usize, y as usize));
    }

    img
}

/// Renders a single pixel in the scene, and returns the colour
fn render_px(scene: &Scene, viewport: Viewport, x: usize, y: usize) -> Pix {
    puffin::profile_function!();

    let ray = viewport.calc_ray(x, y);

    let a = (0.5 * ray.dir().y) + 0.5;

    let white = Pix::from([1., 1., 1.]);
    let blue = Pix::from([0.5, 0.7, 1.]);

    math::lerp(white, blue, a)
}
