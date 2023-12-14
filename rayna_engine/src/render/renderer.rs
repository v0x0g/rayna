use crate::obj::sphere::Sphere;
use crate::obj::Object;
use crate::render::render::{Render, RenderStats};
use crate::render::render_opts::RenderOpts;
use crate::shared::bounds::Bounds;
use crate::shared::camera::Viewport;
use crate::shared::scene::Scene;
use image::Pixel;
use puffin::{profile_function, profile_scope};
use rayna_shared::def::targets::*;
use rayna_shared::def::types::{ImgBuf, Number, Pix, Vector};
use rayna_shared::profiler;
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
            .num_threads(1)
            .thread_name(|id| format!("Renderer::worker_{id}"))
            .start_handler(|_id| profiler::worker_profiler_init())
            .build()
            .map_err(RendererCreateError::from)?;

        Ok(Self { thread_pool: pool })
    }

    // TODO: Should `render()` be fallible?
    pub fn render(&self, scene: &Scene, render_opts: RenderOpts) -> Render<ImgBuf> {
        profile_function!();

        let viewport = match scene.camera.calculate_viewport(render_opts) {
            Ok(v) => v,
            Err(err) => {
                trace!(target: RENDERER, ?err, "couldn't calculate viewport");
                let [w, h] = render_opts.dims_u32_slice();
                return Self::render_failed(w, h);
            }
        };

        self.render_actual(scene, render_opts, viewport)
    }

    pub fn render_convert<T: Send + Sync>(
        &self,
        scene: &Scene,
        render_opts: RenderOpts,
        convert: impl FnOnce(ImgBuf) -> T,
    ) -> Render<T> {
        let render = self.render(scene, render_opts);

        Render {
            img: convert(render.img),
            stats: render.stats,
        }
    }

    /// Renders a single pixel in the scene, and returns the colour
    fn render_px(scene: &Scene, viewport: Viewport, x: usize, y: usize) -> Pix {
        let ray = viewport.calc_ray(x, y);
        let bounds = Bounds::from(0.0..Number::MAX);

        let intersect = scene
            .objects
            .iter()
            .filter_map(|obj| obj.intersect(ray, bounds.clone()))
            .min_by(|a, b| Number::total_cmp(&a.dist, &b.dist));

        intersect
            .map(|i| *Pix::from_slice(&i.normal.as_array().map(|f| (f / 2.) as f32 + 0.5)))
            .unwrap_or_else(|| scene.skybox.sky_colour(ray))
    }

    fn render_actual(
        &self,
        scene: &Scene,
        render_opts: RenderOpts,
        viewport: Viewport,
    ) -> Render<ImgBuf> {
        profile_function!();

        let [w, h] = render_opts.dims_u32_slice();

        let mut img = ImgBuf::new(w, h);

        let duration;
        let num_threads;
        {
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

    /// Helper function for returning a render in case of a failure
    /// (and so we can't make an actual render)
    fn render_failed(w: u32, h: u32) -> Render<ImgBuf> {
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

        let img = if w > 16 && h > 16 {
            internal(w / 4, h / 4)
        } else {
            internal(w, h)
        };

        Render {
            img,
            stats: RenderStats {
                num_threads: 0,
                duration: Duration::ZERO,
                num_px: (w * h) as usize,
            },
        }
    }
}

impl Clone for Renderer {
    fn clone(&self) -> Self {
        Self::new().expect("could not clone: couldn't create renderer")
    }
}
