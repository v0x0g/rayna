use crate::mat::Material;
use crate::render::render::{Render, RenderStats};
use crate::render::render_opts::{RenderMode, RenderOpts};
use crate::shared::bounds::Bounds;
use crate::shared::camera::Viewport;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::scene::Scene;
use crate::shared::validate;
use crate::skybox::Skybox;
use puffin::{profile_function, profile_scope};
use rand::{thread_rng, Rng};
use rayna_shared::def::targets::*;
use rayna_shared::def::types::{Channel, ImgBuf, Number, Pixel};
use rayna_shared::profiler;
use rayon::prelude::IntoParallelIterator;
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
    pub fn render(&self, scene: &Scene, render_opts: &RenderOpts) -> Render<ImgBuf> {
        profile_function!();

        let viewport = match scene.camera.calculate_viewport(render_opts) {
            Ok(v) => v,
            Err(err) => {
                trace!(target: RENDERER, ?err, "couldn't calculate viewport");
                let [w, h] = render_opts.dims_u32_slice();
                return Self::render_failed(w, h);
            }
        };

        self.render_actual(scene, render_opts, &viewport)
    }

    /// Helper function for returning a render in case of a failure
    /// (and so we can't make an actual render)
    /// Probably only called if the viewport couldn't be calculated
    fn render_failed(w: u32, h: u32) -> Render<ImgBuf> {
        profile_function!();

        #[memoize::memoize(Capacity: 8)] // Keep cap small since images can be huge
        fn internal(w: u32, h: u32) -> ImgBuf {
            profile_function!();

            ImgBuf::from_fn(w, h, |x, y| {
                Pixel::from({
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

    /// Does the actual rendering
    ///
    /// This is only called when the viewport is valid, and therefore an image can be rendered
    fn render_actual(
        &self,
        scene: &Scene,
        render_opts: &RenderOpts,
        viewport: &Viewport,
    ) -> Render<ImgBuf> {
        profile_function!();

        let [w, h] = render_opts.dims_u32_slice();

        let mut img = ImgBuf::new(w, h);

        let duration;
        let num_threads;
        {
            let start = puffin::now_ns();
            num_threads = self.thread_pool.current_num_threads();

            // Split each row into an operation for the thread pool
            self.thread_pool.in_place_scope(|scope| {
                let rows = img.enumerate_rows_mut();
                for (_, row) in rows {
                    scope.spawn(|_| {
                        profile_scope!("inner");
                        for (x, y, pix) in row {
                            *pix = Self::render_px(
                                scene,
                                render_opts,
                                viewport,
                                x as usize,
                                y as usize,
                            );
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
    ///
    /// Takes into account [`RenderOpts::msaa`]
    fn render_px(
        scene: &Scene,
        opts: &RenderOpts,
        viewport: &Viewport,
        x: usize,
        y: usize,
    ) -> Pixel {
        let px = x as Number;
        let py = y as Number;
        let sample_count = opts.msaa.get();
        let mut rng = thread_rng();

        let accum = (0..sample_count)
            .into_iter()
            .map(|_s| Self::apply_msaa_shift(px, py, &mut rng))
            .map(|[px, py]| Self::render_px_once(scene, viewport, opts, px, py))
            .inspect(|p| validate::colour(p))
            // Pixel doesn't implement [core::ops::Add], so have to manually do it with slices
            .map(|p| p.0)
            .fold([0.; 3], |[r1, g1, b1], [r2, g2, b2]| {
                [r1 + r2, g1 + g2, b1 + b2]
            });

        let mean = accum.map(|c| c / (sample_count as Channel));
        let pix = Pixel::from(mean);

        validate::colour(pix);
        pix
    }

    /// Renders a given pixel a single time
    fn render_px_once(
        scene: &Scene,
        viewport: &Viewport,
        opts: &RenderOpts,
        x: Number,
        y: Number,
    ) -> Pixel {
        let ray = viewport.calc_ray(x, y);
        validate::ray(ray);
        let bounds = Bounds::from(0.0..Number::MAX);

        let Some(intersect) = Self::calculate_intersection(scene, ray, &bounds) else {
            return scene.skybox.sky_colour(ray);
        };
        validate::intersection(&intersect, &bounds);

        return match opts.mode {
            RenderMode::OutwardNormal => {
                Pixel::from(intersect.normal.as_array().map(|f| (f / 2.) as f32 + 0.5))
            }
            RenderMode::RayNormal => Pixel::from(
                intersect
                    .ray_normal
                    .as_array()
                    .map(|f| (f / 2.) as f32 + 0.5),
            ),
            RenderMode::PBR => Pixel::from([1., 0., 0.]),
            RenderMode::Scatter => Pixel::from(
                intersect
                    .material
                    .scatter(&intersect)
                    .unwrap_or_default()
                    .as_array()
                    .map(|f| (f / 2.) as f32 + 0.5),
            ),
        };
    }

    /// Calculates the nearest intersection in the scene for the given ray
    fn calculate_intersection(
        scene: &Scene,
        ray: Ray,
        bounds: &Bounds<Number>,
    ) -> Option<Intersection> {
        scene
            .objects
            .iter()
            // Intersect all and only include hits not misses
            .filter_map(|obj| obj.intersect(ray, bounds.clone()))
            .inspect(|i| validate::intersection(i, bounds))
            // Choose closest intersect
            .min_by(|a, b| Number::total_cmp(&a.dist, &b.dist))
    }

    fn ray_colour_recursive(
        scene: &Scene,
        ray: Ray,
        opts: &RenderOpts,
        bounds: &Bounds<Number>,
        depth: usize,
    ) -> Pixel {
        if depth > opts.bounces {
            return Pixel::from([0.; 3]);
        }

        let Some(intersect) = Self::calculate_intersection(scene, ray, bounds) else {
            return scene.skybox.sky_colour(ray);
        };
        validate::intersection(&intersect, bounds);

        let Some(scatter_dir) = intersect.material.scatter(&intersect) else {
            // No scatter (material absorbed ray)
            return Pixel::from([0.; 3]);
        };
        validate::normal(&scatter_dir);
        let future_ray = Ray::new(intersect.pos, scatter_dir);
        validate::ray(&ray);

        let future_col = Self::ray_colour_recursive(scene, future_ray, opts, bounds, depth + 1);
        validate::colour(&future_col);

        return intersect
            .material
            .calculate_colour(&intersect, future_ray, future_col);
    }

    /// Calculates a random pixel shift (for MSAA), and applies it to the (pixel) coordinates
    fn apply_msaa_shift<R: Rng>(px: Number, py: Number, rng: &mut R) -> [Number; 2] {
        [
            px + rng.gen_range(-0.5..=0.5),
            py + rng.gen_range(-0.5..=0.5),
        ]
    }
}

impl Clone for Renderer {
    fn clone(&self) -> Self {
        Self::new().expect("could not clone: couldn't create renderer")
    }
}
