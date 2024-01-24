use crate::core::profiler;
use crate::core::targets::*;
use crate::core::types::{Channel, Colour, Image, Number, Point3, Vector2};
use crate::material::Material;
use crate::object::Object;
use crate::render::render::{Render, RenderStats};
use crate::render::render_opts::{RenderMode, RenderOpts};
use crate::scene::Scene;
use crate::shared::bounds::Bounds;
use crate::shared::camera::Viewport;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;
use crate::shared::{math, rng, validate};
use crate::skybox::Skybox;
use derivative::Derivative;
use num_integer::Roots;
use puffin::profile_function;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::rngs::SmallRng;
use rand::Rng;
use rand_core::{RngCore, SeedableRng};
use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder};
use std::marker::PhantomData;
use std::ops::DerefMut;
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
                profiler::renderer_profiler_init();
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
    /// Buffer of [Vector2] values
    pub px_coords: Vec<Vector2>,
    /// Buffer of [Colour] values
    pub samples: Vec<Colour>,
    /// The [Uniform] number distribution for creating MSAA values
    pub msaa_distr: Uniform<Number>,
}

#[derive(Copy, Clone, Debug, Default)]
struct PooledDataAllocator;
impl<R: SeedableRng + RngCore> opool::PoolAllocator<PooledData<R>> for PooledDataAllocator {
    fn allocate(&self) -> PooledData<R> {
        // I will admit I have no idea if you can fill an array from a function like this
        let rngs = [(); 2].map(|()| R::from_entropy());
        let vec2 = vec![];
        let colours = vec![];
        let msaa_dist = Uniform::new_inclusive(-0.5, 0.5);
        PooledData {
            rngs,
            px_coords: vec2,
            samples: colours,
            msaa_distr: msaa_dist,
        }
    }
}

// endregion Pooled/Cached Data

// region High-level Rendering

impl<Obj: Object + Clone, Sky: Skybox + Clone> Renderer<Obj, Sky> {
    // TODO: Should `render()` be fallible?
    pub fn render(&mut self, scene: &Scene<Obj, Sky>, render_opts: &RenderOpts) -> Render<Image> {
        profile_function!();

        // Render image, and collect stats

        let start = puffin::now_ns();
        let num_threads = self.thread_pool.current_num_threads();

        let image = match scene.camera.calculate_viewport(render_opts) {
            Err(err) => {
                trace!(target: RENDERER, ?err, "couldn't calculate viewport");
                let [w, h] = render_opts.dims();
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
    fn render_failed(w: usize, h: usize) -> Image {
        profile_function!();

        #[memoize::memoize(Capacity: 8)] // Keep cap small since images can be huge
        #[cold]
        fn internal(w: usize, h: usize) -> Image {
            profile_function!();

            Image::from_fn(w, h, |x, y| {
                Colour::from({
                    if (x + y) % 2 == 0 {
                        [0., 0., 0.]
                    } else {
                        [1., 0., 1.]
                    }
                })
            })
        }

        // log2 so the pixels are still clearly visible even when the dimensions are huge
        let img = internal(w.ilog2() as usize, h.ilog2() as usize);

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
    ) -> Image {
        profile_function!();

        let [w, h] = render_opts.dims();

        let mut img = Image::new_blank(w, h);

        self.thread_pool.install(|| {
            let data_pool = &self.data_pool;

            let pixels = img
                .deref_mut()
                .into_par_iter()
                .enumerate()
                .map(|(idx, p)| {
                    let (y, x) = num_integer::Integer::div_rem(&idx, &w);
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
    ) -> Colour {
        let px = x as Number;
        let py = y as Number;
        let sample_count = opts.samples.get();

        let PooledData {
            px_coords: sample_coords,
            samples,
            msaa_distr,
            rngs: [rng1, rng2],
        } = pooled_data;

        // Samples are chosen stratified within the area of the pixel.
        // To keep things O(Samples) not O(Samples^2), we might have to skip stratifying some samples
        sample_coords.resize(sample_count, Vector2::ZERO);
        let px_centre: Vector2 = [px, py].into();

        let stratify_dim = sample_count.sqrt();
        let stratify_dim_f = stratify_dim as Number;
        for i in 0..stratify_dim {
            for j in 0..stratify_dim {
                let rand: Vector2 = [msaa_distr.sample(rng1), msaa_distr.sample(rng1)].into();
                let stratify_coord: Vector2 = [i as Number, j as Number].into();
                // Make sure to divide `randomness` and `stratify_coord`
                // so that it doesn't spill out across the stratified sub-pixels
                let coord: Vector2 = px_centre + (rand / stratify_dim_f) + (stratify_coord / stratify_dim_f);
                sample_coords[i + (stratify_dim * j)] = coord;
            }
        }
        // The remainder are fully random
        for i in (stratify_dim * stratify_dim)..sample_count {
            sample_coords[i] = px_centre + Vector2::from([msaa_distr.sample(rng1), msaa_distr.sample(rng1)]);
        }

        samples.clear();
        sample_coords
            .into_iter()
            .map(|&mut Vector2 { x, y }| Self::render_px_once(scene, viewport, opts, bounds, x, y, rng2))
            .inspect(|p| validate::colour(p))
            .collect_into(samples);

        let overall_colour = {
            // Find mean of all samples
            let accum: Colour = samples.iter().copied().sum();

            let mean = accum.map(|c| c / (samples.len() as Channel));
            let pix = Colour::from(mean);
            pix
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
    ) -> Colour {
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
        const COLOURS: [Colour; N_COL] = [
            Colour::new([1.0, 1.0, 1.0]),
            Colour::new([1.0, 0.0, 0.0]),
            Colour::new([1.0, 0.5, 0.0]),
            Colour::new([1.0, 1.0, 0.0]),
            Colour::new([0.5, 1.0, 0.0]),
            Colour::new([0.0, 1.0, 0.0]),
            Colour::new([0.0, 1.0, 0.5]),
            Colour::new([0.0, 1.0, 1.0]),
            Colour::new([0.0, 0.5, 1.0]),
            Colour::new([0.0, 0.0, 1.0]),
            Colour::new([0.5, 0.0, 1.0]),
            Colour::new([1.0, 0.0, 1.0]),
            Colour::new([0.0, 0.0, 0.0]),
        ];

        return match mode {
            RenderMode::PBR => unreachable!("mode == RenderMode::PBR already checked"),
            RenderMode::OutwardNormal => Colour::from(intersect.normal.as_array().map(|f| (f / 2.) as Channel + 0.5)),
            RenderMode::RayNormal => Colour::from(intersect.ray_normal.as_array().map(|f| (f / 2.) as Channel + 0.5)),
            RenderMode::Scatter => Colour::from(
                material
                    .scatter(&ray, &intersect, rng)
                    .unwrap_or_default()
                    .as_array()
                    .map(|f| (f / 2.) as Channel + 0.5),
            ),
            RenderMode::Uv => Colour::from([
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

    /// Recursive function that calculates the colour in the scene for a given ray.
    ///
    /// # Recursion
    /// This will recurse each time the ray scatters off an object in the scene, up to a limit imposed by [RenderOpts::bounces].
    /// It should be fine for all *reasonable* bounce limits (~200), but will most likely overflow the stack past that.
    fn ray_colour_recursive(
        scene: &Scene<Obj, Sky>,
        ray: &Ray,
        opts: &RenderOpts,
        bounds: &Bounds<Number>,
        depth: usize,
        rng: &mut impl Rng,
    ) -> Colour {
        if depth > opts.bounces {
            return Colour::from([0.; 3]);
        }

        // Intersect
        let Some(FullIntersection { intersection, material }) = Self::calculate_intersection(scene, ray, bounds, rng)
        else {
            return scene.skybox.sky_colour(ray);
        };
        validate::intersection(ray, &intersection, bounds);

        let col_emitted = {
            let col = material.emitted_light(ray, &intersection, rng);
            validate::colour(&col);
            col
        };

        // Calculate scatter
        let future_ray = {
            let Some(future_ray_dir) = material.scatter(ray, &intersection, rng) else {
                return col_emitted;
            };
            validate::normal3(&future_ray_dir);
            let future_ray = Ray::new(intersection.pos_w, future_ray_dir);
            validate::ray(future_ray);
            future_ray
        };

        // Follow ray and calculate future bounces
        let col_scattered = {
            let col_future = Self::ray_colour_recursive(scene, &future_ray, opts, bounds, depth + 1, rng);
            validate::colour(&col_future);
            let col_scattered = material.reflected_light(ray, &intersection, &future_ray, &col_future, rng);
            validate::colour(&col_scattered);
            col_scattered
        };
        let prob_scattered = material.scatter_probability(ray, &future_ray, &intersection);

        // // PDF value for the scattered ray
        // let prob_scatter = material.scatter_probability(ray, &future_ray, &intersection);
        //
        // // Try calculate a light ray
        // const N_LIGHTS: usize = 1;
        // //
        // let mut prob_light_sum = 1.;
        // let mut col_lights =
        // for i in 0..N_LIGHTS{
        //
        // }

        // Calculate the future colour, including from light sources

        let ray_light = Ray::new(
            intersection.pos_w,
            Point3::new(0.5, 1.0, 0.5) + rng::vector_in_unit_cube_01(rng) * 0.01 - intersection.pos_w,
        );
        // depth=opts.bounces so it don't recurse again
        let col_light = Self::ray_colour_recursive(scene, &ray_light, opts, bounds, opts.bounces, rng);
        let prob_light = material.scatter_probability(ray, &ray_light, &intersection);

        let mut col_accum = Colour::BLACK;
        let mut prob_accum = 0.;

        let samples = [(col_scattered, prob_scattered)];

        // Do a weighted average of each source of light.
        for (col, prob) in samples {
            col_accum += col * prob as Channel;
            prob_accum += prob;
        }

        // Normalise at the end by dividing by dividing by total probability
        let col_avg = col_accum / prob_accum as Channel;

        col_emitted + col_avg
    }
}

// endregion Low-level Rendering
