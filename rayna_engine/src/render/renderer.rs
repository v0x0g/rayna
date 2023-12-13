use crate::def::targets::*;
use crate::def::types::{ImgBuf, Pix};
use crate::render::render::{Render, RenderStats};
use crate::render::render_opts::RenderOpts;
use crate::shared::camera::Viewport;
use crate::shared::math;
use crate::shared::scene::Scene;
use puffin::profile_scope;
use rayon::{ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder};
use std::time::Duration;
use thiserror::Error;
use tracing::trace;

#[derive(Debug)]
pub struct Renderer {
    /// A thread pool used to distribute the workload
    thread_pool: ThreadPool,
}

#[derive(Error, Debug)]
pub enum RendererCreateError {
    #[error("failed to create worker thread pool")]
    ThreadPoolError {
        #[backtrace]
        #[from]
        source: ThreadPoolBuildError,
    },
}

impl Renderer {
    pub fn new() -> Result<Self, RendererCreateError> {
        let pool = ThreadPoolBuilder::new()
            .num_threads(0)
            .thread_name(|id| format!("Renderer::worker_{id}"))
            .build()
            .map_err(RendererCreateError::from)?;

        Ok(Self { thread_pool: pool })
    }

    // TODO: Should `render()` be fallible?
    pub fn render(&self, scene: &Scene, render_opts: RenderOpts) -> Render {
        puffin::profile_function!();

        let [w, h] = render_opts.dims_u32_slice();

        let mut img = ImgBuf::new(w, h);

        let viewport = match scene.camera.calculate_viewport(render_opts) {
            Ok(v) => v,
            Err(err) => {
                trace!(target: RENDERER, ?err, "couldn't calculate viewport");
                return Render {
                    img: Self::render_failed_image(w, h),
                    stats: RenderStats {
                        num_threads: 0,
                        duration: Duration::ZERO,
                        num_px: (w * h) as usize,
                    },
                };
            }
        };

        let duration;
        let num_threads;
        {
            profile_scope!("render_pixels");
            let start = puffin::now_ns();
            num_threads = self.thread_pool.current_num_threads();
            self.thread_pool.in_place_scope(|scope| {
                let rows = img.enumerate_rows_mut();
                for (_, row) in rows {
                    scope.spawn(|_| {
                        profile_scope!("inner");
                        for (x, y, pix) in row {
                            *pix = Self::render_px(scene, viewport, x as usize, y as usize);
                        }
                    });
                }
            });
            let end = puffin::now_ns();
            duration = Duration::from_nanos(end.abs_diff(start));
        }

        Render {
            img,
            stats: RenderStats {
                num_threads,
                duration,
                num_px: (w * h) as usize,
            },
        }
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

    /// Helper function to
    fn render_failed_image(w: u32, h: u32) -> ImgBuf {
        puffin::profile_function!();

        #[memoize::memoize(Capacity: 8)] // Keep cap small since images can be huge
        fn internal(w: u32, h: u32) -> ImgBuf {
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

        if w > 16 && h > 16 {
            internal(w / 4, h / 4)
        } else {
            internal(w, h)
        }
    }
}

impl Clone for Renderer {
    fn clone(&self) -> Self {
        Self::new().expect("could not clone: couldn't create renderer")
    }
}
