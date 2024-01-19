use crate::material::Material;
use crate::object::Object;
use crate::render::render::{Render, RenderStats};
use crate::render::render_opts::{RenderMode, RenderOpts};
use crate::scene::Scene;
use crate::shared::bounds::Bounds;
use crate::shared::camera::Viewport;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;
use crate::shared::{math, validate};
use crate::skybox::Skybox;
use average::{Estimate, Variance};
use derivative::Derivative;
use image::Pixel as _;
use itertools::Itertools;
use puffin::profile_function;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::rngs::SmallRng;
use rand::Rng;
use rand_core::{RngCore, SeedableRng};
use rayna_shared::def::targets::*;
use rayna_shared::def::types::{Channel, ImgBuf, Number, Pixel};
use rayna_shared::profiler;
use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder};
use std::array;
use std::marker::PhantomData;
use std::ops::{Add, DerefMut};
use std::sync::OnceLock;
use std::time::Duration;
use thiserror::Error;
use tracing::{error, trace};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Renderer<Obj: Object + Clone, Sky: Skybox + Clone> {
    /// A thread pool used to distribute the workload
    thread_pool: ThreadPool,
    #[derivative(Debug = "ignore")]
    data_pool: opool::Pool<PooledDataAllocator, PooledData>,
    phantom: PhantomData<Scene<Obj, Sky>>,
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

// region Construction

impl<Obj: Object + Clone, Sky: Skybox + Clone> Renderer<Obj, Sky> {
    /// Creates a new renderer instance
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
        let data_pool = opool::Pool::new_prefilled(256, PooledDataAllocator);

        Ok(Self {
            thread_pool,
            data_pool,
            phantom: PhantomData {},
        })
    }
}

/// Clone Renderer
impl<Obj: Object + Clone, Sky: Skybox + Clone> Clone for Renderer<Obj, Sky> {
    fn clone(&self) -> Self { Self::new().expect("could not clone: couldn't create renderer") }
}

// endregion Construction

// region Pooled/Cached Data

/// A helper struct that holds data we want to be pooled
#[derive(Clone, Debug)]
struct PooledData<R: RngCore + SeedableRng = SmallRng> {
    /// PRNG's
    pub rngs: [R; 2],
    /// Buffer of [Pixel] values
    pub samples: Vec<Pixel>,
    /// The [Uniform] number distribution for creating MSAA values
    pub msaa_distr: Uniform<Number>,
}

#[derive(Copy, Clone, Debug, Default)]
struct PooledDataAllocator;
impl<R: SeedableRng + RngCore> opool::PoolAllocator<PooledData<R>> for PooledDataAllocator {
    fn allocate(&self) -> PooledData<R> {
        PooledData {
            // I will admit I have no idea how to fill an array from a closure lol
            rngs: array::from_fn(|_| R::from_entropy()),
            samples: vec![],
            msaa_distr: Uniform::new_inclusive(-0.5, 0.5),
        }
    }
}

// endregion Pooled/Cached Data

// region High-level Rendering

impl<Obj: Object + Clone, Sky: Skybox + Clone> Renderer<Obj, Sky> {
    // TODO: Should `render()` be fallible?
    pub fn render(&mut self, scene: &Scene<Obj, Sky>, render_opts: &RenderOpts) -> Render<ImgBuf> {
        profile_function!();

        // Render image, and collect stats

        let start = puffin::now_ns();
        let num_threads = self.thread_pool.current_num_threads();

        let image = match scene.camera.calculate_viewport(render_opts) {
            Err(err) => {
                trace!(target: RENDERER, ?err, "couldn't calculate viewport");
                let [w, h] = render_opts.dims_u32_slice();
                Self::render_failed(w, h)
            }
            Ok(viewport) => {
                let bounds = Bounds::from(1e-3..Number::MAX);
                self.render_actual(scene, render_opts, &viewport, &bounds)
            }
        };

        let end = puffin::now_ns();
        let duration = Duration::from_nanos(end.abs_diff(start));

        Render {
            img: image,
            stats: RenderStats {
                duration,
                num_threads,
                opts: *render_opts,
            },
        }
    }

    /// Helper function for returning a render in case of a failure
    /// (and so we can't make an actual render)
    /// Probably only called if the viewport couldn't be calculated
    fn render_failed(w: u32, h: u32) -> ImgBuf {
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

        // log2 so the pixels are still clearly visible even when the dimensions are huge
        let img = internal(w.ilog2(), h.ilog2());

        return img;
    }

    /// Does the actual rendering
    ///
    /// This is only called when the viewport is valid, and therefore an image can be rendered
    fn render_actual(
        &mut self,
        scene: &Scene<Obj, Sky>,
        render_opts: &RenderOpts,
        viewport: &Viewport,
        bounds: &Bounds<Number>,
    ) -> ImgBuf {
        profile_function!();

        let [w, h] = render_opts.dims_u32_slice();

        let mut img = ImgBuf::new(w, h);

        self.thread_pool.install(|| {
            let data_pool = &self.data_pool;

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
                    // Can't use puffin's macro because of macro hygiene :(
                    let profiler_scope = if puffin::are_scopes_on() {
                        static LOCATION: OnceLock<String> = OnceLock::new();
                        let location = LOCATION.get_or_init(|| format!("{}:{}", puffin::current_file_name!(), line!()));
                        Some(puffin::ProfilerScope::new("inner", location, ""))
                    } else {
                        None
                    };

                    // Pull values from our thread pool
                    // We hold them for the duration of each work segment, so we don't pull/push each pixel
                    (profiler_scope, data_pool.get())
                },
                // Process each pixel
                |(_scope, pooled), (x, y, p)| {
                    *p = Self::render_px(scene, render_opts, viewport, bounds, x, y, pooled.deref_mut());
                },
            );
        });

        return img;
    }
}

// endregion High-level Rendering

// region Low-level Rendering

impl<Obj: Object + Clone, Sky: Skybox + Clone> Renderer<Obj, Sky> {
    /// Renders a single pixel in the scene, and returns the colour
    ///
    /// Takes into account [`RenderOpts::msaa`]
    fn render_px(
        scene: &Scene<Obj, Sky>,
        opts: &RenderOpts,
        viewport: &Viewport,
        bounds: &Bounds<Number>,
        x: usize,
        y: usize,
        pooled_data: &mut PooledData,
    ) -> Pixel {
        let px = x as Number;
        let py = y as Number;
        let target_sample_count = opts.samples.get();

        let PooledData {
            samples,
            msaa_distr,
            rngs: [rng1, rng2],
        } = pooled_data;

        samples.clear();

        // NOTE: This is "Smart MSAA". It optimises MSAA by doing slightly fewer than required samples for most pixels,
        //  but sampling again if the samples have a lot of variance
        let samples_per_iteration = (target_sample_count / 4).max(target_sample_count);
        let samples_max = target_sample_count * 2;
        debug_assert!(samples_per_iteration >= 1 && samples_max >= target_sample_count);

        let overall_colour = loop {
            // Do a bunch of samples
            std::iter::repeat_with(|| [px + msaa_distr.sample(rng1), py + msaa_distr.sample(rng1)])
                .take(samples_per_iteration)
                .map(|[x, y]| Self::render_px_once(scene, viewport, opts, bounds, x, y, rng2))
                .inspect(|p| validate::colour(p))
                .collect_into(samples);

            // Find mean of all samples. Don't have anything better than this ATM
            let ests = samples
                .iter()
                .map(|p| p.0)
                .fold(array::from_fn(|_| Variance::new()), |mut est, col| {
                    est.iter_mut().zip_eq(col).for_each(|(e, c)| e.add(c as f64));
                    est
                });

            let means = ests.clone().map(|e| e.mean() as Channel);
            let vars = ests.map(|e| e.sample_variance());

            // Are the samples generally consistent with one another, or is there a lot of noise?
            let samples_are_noisy = vars.iter().sum::<f64>() > 0.5;

            // Enough good samples, or we've sampled up to threshold
            if !samples_are_noisy || samples.len() > samples_max {
                break Pixel::from(means);
            }
        };

        validate::colour(overall_colour);
        overall_colour
    }

    /// Renders a given pixel a single time
    ///
    /// This handles the switching between render modes
    fn render_px_once(
        scene: &Scene<Obj, Sky>,
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
        scene: &'o Scene<Obj, Sky>,
        ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Obj::Mat>> {
        scene.objects.full_intersect(ray, bounds, rng)
    }

    fn ray_colour_recursive(
        scene: &Scene<Obj, Sky>,
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
}

// endregion Low-level Rendering
