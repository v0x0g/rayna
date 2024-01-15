use crate::material::Material;
use crate::object::FullObject;
use crate::render::render::{Render, RenderStats};
use crate::render::render_opts::{RenderMode, RenderOpts};
use crate::scene::Scene;
use crate::shared::bounds::Bounds;
use crate::shared::camera::Viewport;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;
use crate::shared::rng::RngPoolAllocator;
use crate::shared::{math, validate};
use crate::skybox::Skybox;
use derivative::Derivative;
use image::Pixel as _;
use puffin::profile_function;
use rand::rngs::SmallRng;
use rand::Rng;
use rand_core::RngCore;
use rayna_shared::def::targets::*;
use rayna_shared::def::types::{Channel, ImgBuf, Number, Pixel};
use rayna_shared::profiler;
use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder};
use smallvec::SmallVec;
use std::ops::{Add, DerefMut};
use std::sync::OnceLock;
use std::time::Duration;
use thiserror::Error;
use tracing::{error, trace};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Renderer {
    /// A thread pool used to distribute the workload
    thread_pool: ThreadPool,
    #[derivative(Debug = "ignore")]
    rng_pool: opool::Pool<RngPoolAllocator, SmallRng>,
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

        // Create a pool that should have enough RNGs stored for all of our threads
        // We pool randoms so we don't have to init/create them in hot paths
        // `SmallRng` is the (slightly) fastest of all RNGs tested
        let rng_pool = opool::Pool::new_prefilled(256, RngPoolAllocator);

        Ok(Self { thread_pool, rng_pool })
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

            self.thread_pool.install(|| {
                let rng_pool = &self.rng_pool;

                let pixels = img
                    .deref_mut()
                    .par_chunks_exact_mut(3)
                    .enumerate()
                    .map(|(idx, chans)| {
                        let (y, x) = num_integer::Integer::div_rem(&idx, &(w as usize));
                        let p = Pixel::from_slice_mut(chans);
                        (x, y, p)
                    })
                    // Return on panic as fast as possible; don't keep processing all the pixels on panic
                    // Otherwise we get (literally) millions of panics (1 per pixel) which just hangs the renderer as it prints
                    .panic_fuse();

                pixels.for_each_init(
                    move || {
                        // Can't use macro because of macro hygiene :(
                        let profiler_scope = if puffin::are_scopes_on() {
                            static LOCATION: OnceLock<String> = OnceLock::new();
                            let location =
                                LOCATION.get_or_init(|| format!("{}:{}", puffin::current_file_name!(), line!()));
                            Some(puffin::ProfilerScope::new("inner", location, ""))
                        } else {
                            None
                        };
                        let rng_1 = rng_pool.get();
                        let rng_2 = rng_pool.get();
                        (rng_1, rng_2, profiler_scope)
                    },
                    // Process each pixel
                    |(rng_1, rng_2, _), (x, y, p)| {
                        *p = Self::render_px(
                            scene,
                            render_opts,
                            viewport,
                            bounds,
                            x,
                            y,
                            rng_1.deref_mut(),
                            rng_2.deref_mut(),
                        );
                    },
                );
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
    ///
    /// This handles the switching between render modes
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

        let Some(FullIntersection {
            intersection: intersect,
            material,
        }) = Self::calculate_intersection(scene, &ray, bounds, rng)
        else {
            return scene.skybox.sky_colour(&ray);
        };
        validate::intersection(ray, &intersect, bounds);

        // Some colours to help with visualisation
        const N_COL: usize = 13;
        const COLOURS: [Pixel; N_COL] = [
            Pixel { 0: [1.0, 1.0, 1.0] },
            Pixel { 0: [1.0, 0.0, 0.0] },
            Pixel { 0: [1.0, 0.5, 0.0] },
            Pixel { 0: [1.0, 1.0, 0.0] },
            Pixel { 0: [0.5, 1.0, 0.0] },
            Pixel { 0: [0.0, 1.0, 0.0] },
            Pixel { 0: [0.0, 1.0, 0.5] },
            Pixel { 0: [0.0, 1.0, 1.0] },
            Pixel { 0: [0.0, 0.5, 1.0] },
            Pixel { 0: [0.0, 0.0, 1.0] },
            Pixel { 0: [0.5, 0.0, 1.0] },
            Pixel { 0: [1.0, 0.0, 1.0] },
            Pixel { 0: [0.0, 0.0, 0.0] },
        ];

        return match mode {
            RenderMode::PBR => unreachable!("mode == RenderMode::PBR already checked"),
            RenderMode::OutwardNormal => Pixel::from(intersect.normal.as_array().map(|f| (f / 2.) as f32 + 0.5)),
            RenderMode::RayNormal => Pixel::from(intersect.ray_normal.as_array().map(|f| (f / 2.) as Channel + 0.5)),
            RenderMode::Scatter => Pixel::from(
                material
                    .scatter(&ray, &intersect, rng)
                    .unwrap_or_default()
                    .as_array()
                    .map(|f| (f / 2.) as Channel + 0.5),
            ),
            RenderMode::Uv => Pixel::from([
                (intersect.uv.x as Channel).clamp(0., 1.),
                (intersect.uv.y as Channel).clamp(0., 1.),
                0.,
            ]),
            RenderMode::Face => {
                // TODO: Make `Object: Hash`
                let hash = intersect.face % (N_COL - 1) + 1;
                COLOURS[hash]
            }
            RenderMode::Distance => {
                let dist = intersect.dist;
                // let val = (dist + 1.).log2();
                let val = 2. * dist.cbrt();

                let floor = val.floor().clamp(0., (N_COL - 1) as _);
                let ceil = val.ceil().clamp(0., (N_COL - 1) as _);
                let frac = val - floor;

                let a = COLOURS[floor as usize];
                let b = COLOURS[ceil as usize];
                let lerp = math::lerp_px(a, b, frac);
                lerp
            }
        };
    }

    /// Calculates the nearest intersection in the scene for the given ray
    fn calculate_intersection<'o>(
        scene: &'o Scene,
        ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o>> {
        scene.objects.full_intersect(ray, bounds, rng)
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

        let Some(FullIntersection { intersection, material }) = Self::calculate_intersection(scene, ray, bounds, rng)
        else {
            return scene.skybox.sky_colour(ray);
        };
        validate::intersection(ray, &intersection, bounds);

        return match material.scatter(ray, &intersection, rng) {
            Some(scatter_dir) => {
                validate::normal3(&scatter_dir);

                let future_ray = Ray::new(intersection.pos_w, scatter_dir);
                validate::ray(future_ray);

                let future_col = Self::ray_colour_recursive(scene, &future_ray, opts, bounds, depth + 1, rng);
                validate::colour(&future_col);

                let reflected_col = material.reflected_light(ray, &intersection, &future_ray, &future_col, rng);
                let emitted_col = material.emitted_light(ray, &intersection, rng);

                Pixel::map2(&reflected_col, &emitted_col, Channel::add)
            }
            // No scatter, so only emission
            None => {
                let emitted_col = material.emitted_light(ray, &intersection, rng);
                emitted_col
            }
        };
    }

    /// Calculates a random pixel shift (for MSAA), and applies it to the (pixel) coordinates

    #[inline /* Hot path */]
    fn apply_msaa_shift(px: Number, py: Number, rng: &mut impl Rng) -> [Number; 2] {
        [px + rng.gen_range(-0.5..=0.5), py + rng.gen_range(-0.5..=0.5)]
    }
}

impl Clone for Renderer {
    fn clone(&self) -> Self { Self::new().expect("could not clone: couldn't create renderer") }
}
