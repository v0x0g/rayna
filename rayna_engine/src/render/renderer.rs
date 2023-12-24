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
use derivative::Derivative;
use image::Pixel as _;
use puffin::{profile_function, profile_scope};
use rand::{Rng, SeedableRng};
use rayna_shared::def::targets::*;
use rayna_shared::def::types::{Channel, ImgBuf, Number, Pixel};
use rayna_shared::profiler;
use rayon::{ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder};
use smallvec::SmallVec;
use std::ops::Add;
use std::time::Duration;
use thiserror::Error;
use tracing::trace;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Renderer {
    /// A thread pool used to distribute the workload
    thread_pool: ThreadPool,
    #[derivative(Debug = "ignore")]
    seed_rng: SeedRng,
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

/// Tge RNG that we use to seed our rendering PRNGs
type SeedRng = rand::rngs::OsRng;
/// Type alias for what PRNG the renderer uses
type MyRng = rand_pcg::Lcg128Xsl64;

impl Renderer {
    pub fn new() -> Result<Self, RendererCreateError> {
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(10)
            .thread_name(|id| format!("Renderer::worker_{id}"))
            .start_handler(|id| {
                trace!(target: RENDERER, "renderer worker {id} start");
                profiler::worker_profiler_init();
            })
            .exit_handler(|id| trace!(target: RENDERER, "renderer worker {id} exit"))
            .build()
            .map_err(RendererCreateError::from)?;

        let rng = SeedRng::default();

        Ok(Self {
            thread_pool,
            seed_rng: rng,
        })
    }

    // TODO: Should `render()` be fallible?
    pub fn render(&mut self, scene: &Scene, render_opts: &RenderOpts) -> Render<ImgBuf> {
        profile_function!();

        let viewport = match scene.camera.calculate_viewport(render_opts) {
            Ok(v) => v,
            Err(err) => {
                trace!(target: RENDERER, ?err, "couldn't calculate viewport");
                let [w, h] = render_opts.dims_u32_slice();
                return Self::render_failed(w, h);
            }
        };
        let bounds = Bounds::from(1e-3..Number::MAX);

        self.render_actual(scene, render_opts, &viewport, &bounds)
    }

    /// Helper function for returning a render in case of a failure
    /// (and so we can't make an actual render)
    /// Probably only called if the viewport couldn't be calculated
    fn render_failed(w: u32, h: u32) -> Render<ImgBuf> {
        profile_function!();

        #[memoize::memoize(Capacity: 8)] // Keep cap small since images can be huge
        #[cold]
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
        &mut self,
        scene: &Scene,
        render_opts: &RenderOpts,
        viewport: &Viewport,
        bounds: &Bounds<Number>,
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
                    // Cache randoms so we don't `clone()` in hot paths
                    let mut rng_1 =
                        MyRng::from_rng(self.seed_rng).expect("failed init rng from seed_rng");
                    let mut rng_2 =
                        MyRng::from_rng(self.seed_rng).expect("failed init rng from seed_rng");
                    scope.spawn(move |_| {
                        profile_scope!("inner");

                        for (x, y, pix) in row {
                            *pix = Self::render_px(
                                scene,
                                render_opts,
                                viewport,
                                bounds,
                                x as usize,
                                y as usize,
                                &mut rng_1,
                                &mut rng_2,
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
        bounds: &Bounds<Number>,
        x: usize,
        y: usize,
        rng_1: &mut impl Rng,
        rng_2: &mut impl Rng,
    ) -> Pixel {
        let px = x as Number;
        let py = y as Number;
        let sample_count = opts.msaa.get();

        let samples: SmallVec<[Pixel; 32]> = (0..sample_count)
            .into_iter()
            .map(|_s| Self::apply_msaa_shift(px, py, rng_1))
            .map(|[px, py]| Self::render_px_once(scene, viewport, opts, bounds, px, py, rng_2))
            .inspect(|p| validate::colour(p))
            .collect();

        // Pixel doesn't implement [core::ops::Add], so have to manually do it with slices
        // TODO: Implement something better than just averaging
        let accum = samples
            .iter()
            .copied()
            .reduce(|a, b| Pixel::map2(&a, &b, Channel::add))
            .unwrap_or_else(|| [0.; 3].into());

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
        bounds: &Bounds<Number>,
        x: Number,
        y: Number,
        rng: &mut impl Rng,
    ) -> Pixel {
        let ray = viewport.calc_ray(x, y, rng);
        validate::ray(ray);
        let mode = opts.mode;

        if mode == RenderMode::PBR {
            return Self::ray_colour_recursive(scene, &ray, opts, bounds, 0, rng);
        }

        let Some(intersect) = Self::calculate_intersection(scene, &ray, bounds) else {
            return scene.skybox.sky_colour(&ray);
        };
        validate::intersection(ray, &intersect, bounds);

        return match mode {
            RenderMode::OutwardNormal => {
                Pixel::from(intersect.normal.as_array().map(|f| (f / 2.) as f32 + 0.5))
            }
            RenderMode::RayNormal => Pixel::from(
                intersect
                    .ray_normal
                    .as_array()
                    .map(|f| (f / 2.) as f32 + 0.5),
            ),
            RenderMode::PBR => unreachable!("mode == RenderMode::PBR already checked"),
            RenderMode::Scatter => Pixel::from(
                intersect
                    .material
                    .scatter(&ray, &intersect, rng)
                    .unwrap_or_default()
                    .as_array()
                    .map(|f| (f / 2.) as f32 + 0.5),
            ),
        };
    }

    /// Calculates the nearest intersection in the scene for the given ray
    fn calculate_intersection(
        scene: &Scene,
        ray: &Ray,
        bounds: &Bounds<Number>,
    ) -> Option<Intersection> {
        scene
            .objects
            .iter()
            // Intersect all and only include hits not misses
            .filter_map(|obj| obj.intersect(ray, bounds))
            .inspect(|i| validate::intersection(ray, i, bounds))
            // Choose closest intersect
            .min_by(|a, b| Number::total_cmp(&a.dist, &b.dist))
    }

    fn ray_colour_recursive(
        scene: &Scene,
        ray: &Ray,
        opts: &RenderOpts,
        bounds: &Bounds<Number>,
        depth: usize,
        rng: &mut impl Rng,
    ) -> Pixel {
        if depth > opts.bounces {
            return Pixel::from([0.; 3]);
        }

        let Some(intersect) = Self::calculate_intersection(scene, ray, bounds) else {
            return scene.skybox.sky_colour(ray);
        };
        validate::intersection(ray, &intersect, bounds);

        let Some(scatter_dir) = intersect.material.scatter(ray, &intersect, rng) else {
            // No scatter (material absorbed ray)
            return Pixel::from([0.; 3]);
        };
        validate::normal3(&scatter_dir);
        let future_ray = Ray::new(intersect.pos, scatter_dir);
        validate::ray(future_ray);

        let future_col =
            Self::ray_colour_recursive(scene, &future_ray, opts, bounds, depth + 1, rng);
        validate::colour(&future_col);

        return intersect
            .material
            .calculate_colour(&ray, &intersect, &future_ray, &future_col);
    }

    /// Calculates a random pixel shift (for MSAA), and applies it to the (pixel) coordinates
    fn apply_msaa_shift(px: Number, py: Number, rng: &mut impl Rng) -> [Number; 2] {
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
