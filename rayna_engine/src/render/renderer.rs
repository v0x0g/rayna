use crate::core::profiler;
use crate::core::targets::*;
use crate::core::types::{Channel, Colour, Image, Number, Vector2};
use crate::material::Material;
use crate::object::Object;
use crate::render::render::{Render, RenderStats};
use crate::render::render_opts::{RenderMode, RenderOpts};
use crate::scene::camera::Camera;
use crate::scene::camera::Viewport;
use crate::scene::Scene;
use crate::shared::intersect::ObjectIntersection;
use crate::shared::interval::Interval;
use crate::shared::math::Lerp;
use crate::shared::ray::Ray;
use crate::shared::validate;
use crate::skybox::Skybox;
use ndarray::Zip;
use num_integer::Roots as _;
use puffin::profile_function;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand_core::{RngCore, SeedableRng};
use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder};
use smallvec::SmallVec;
use std::ops::DerefMut as _;
use std::time::Duration;
use thiserror::Error;
use tracing::{error, trace};

use super::accum_buffer::AccumulationBuffer;

/// The main struct that does the rendering of scenes
///
///
#[derive(derivative::Derivative, getset::Getters, getset::Setters)]
#[derivative(Debug)]
pub struct Renderer<Obj, Sky, Rng> {
    /// A thread pool used to distribute the workload
    thread_pool: ThreadPool,
    data_pool: opool::Pool<PooledDataAllocator, PooledData<Rng>>,
    /// Accumulation buffer storing the [accumulated] result of previous renders.
    accum_buffer: AccumulationBuffer,
    // Purposefully storing these in the render (though not really required)
    // for future compatibility with GPU renderer
    #[getset(get = "pub")]
    scene: Scene<Obj, Sky>,
    #[getset(get = "pub")]
    camera: Camera,
    #[getset(get = "pub")]
    options: RenderOpts,
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

impl<Obj, Sky, Rng> Renderer<Obj, Sky, Rng> {
    /// Creates a new renderer instance, using default values for the scene, camera, and render options
    pub fn new_default() -> Result<Self, RendererCreateError>
    where
        Obj: Default,
        Sky: Default,
        Rng: SeedableRng,
    {
        Self::new_from(
            Scene {
                objects: Obj::default(),
                skybox: Sky::default(),
            },
            Camera::default(),
            RenderOpts::default(),
            0,
        )
    }

    /// Creates a new renderer instance, from the given scene, camera, and render options
    pub fn new_from(
        scene: Scene<Obj, Sky>,
        camera: Camera,
        options: RenderOpts,
        num_threads: usize,
    ) -> Result<Self, RendererCreateError>
    where
        Rng: SeedableRng,
    {
        let thread_pool = Self::create_thread_pool(num_threads).map_err(RendererCreateError::from)?;
        let data_pool = Self::create_data_pool();
        let accum_buffer = AccumulationBuffer::default();

        Ok(Self {
            thread_pool,
            data_pool,
            accum_buffer,
            scene,
            camera,
            options,
        })
    }

    /// Helper method to create the thread pool
    fn create_thread_pool(num_threads: usize) -> Result<ThreadPool, ThreadPoolBuildError> {
        ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .thread_name(|id| format!("Renderer::worker_{id}"))
            .start_handler(|id| {
                trace!(target: RENDERER, "renderer worker {id} start");
                profiler::renderer::init_thread();
            })
            .exit_handler(|id| trace!(target: RENDERER, "renderer worker {id} exit"))
            .build()
    }

    /// Helper method to create the data pool
    fn create_data_pool() -> opool::Pool<PooledDataAllocator, PooledData<Rng>>
    where
        Rng: SeedableRng,
    {
        // Create a pool that should have enough RNGs stored for all of our threads
        // We pool randoms so we don't have to init/create them in hot paths
        opool::Pool::new_prefilled(256, PooledDataAllocator)
    }
}

/// Clone Renderer
impl<Obj: Clone, Sky: Clone, Rng: SeedableRng> Clone for Renderer<Obj, Sky, Rng> {
    fn clone(&self) -> Self {
        // No good way to clone thread pool or data pool
        Self::new_from(
            self.scene.clone(),
            self.camera.clone(),
            self.options.clone(),
            self.thread_pool.current_num_threads(),
        )
        .expect("could not clone: couldn't create renderer")
    }
}

// endregion Construction

// region Properties

impl<Obj, Sky, Rng> Renderer<Obj, Sky, Rng> {
    /// Clears the accumulation buffer, removing all previous renderer frames
    pub fn clear_accumulation(&mut self) { self.accum_buffer.clear(); }

    /// Sets the camera.
    ///
    /// Also clears the accumulation buffer
    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
        self.clear_accumulation();
    }
    /// Sets the scene to be rendered.
    ///
    /// Also clears the accumulation buffer
    pub fn set_scene(&mut self, scene: Scene<Obj, Sky>) {
        self.scene = scene;
        self.clear_accumulation();
    }

    /// Sets the render options.
    ///
    /// Also clears the accumulation buffer
    pub fn set_options(&mut self, options: RenderOpts) {
        self.options = options;
        self.clear_accumulation();
    }

    /// Changes the number of threads used for rendering
    pub fn set_thread_count(&mut self, num_threads: usize) -> Result<(), ThreadPoolBuildError> {
        self.thread_pool = Self::create_thread_pool(num_threads)?;
        Ok(())
    }
}

// endregion Properties

// region Pooled/Cached Data

/// A helper struct that holds data we want to be pooled
#[derive(Clone, Debug)]
struct PooledData<Rng> {
    /// PRNG's
    pub rngs: [Rng; 2],
    /// Buffer of [Vector2] values
    pub px_coords: Vec<Vector2>,
    /// Buffer of [Colour] values
    pub px_samples: Vec<Colour>,
    /// The [Uniform] number distribution for creating MSAA values
    pub msaa_distr: Uniform<Number>,
}

#[derive(Copy, Clone, Debug, Default)]
struct PooledDataAllocator;
impl<Rng: SeedableRng> opool::PoolAllocator<PooledData<Rng>> for PooledDataAllocator {
    fn allocate(&self) -> PooledData<Rng> {
        // I will admit I have no idea if you can fill an array from a function like this
        let rngs = [(); 2].map(|()| Rng::from_entropy());
        let msaa_dist = Uniform::new_inclusive(-0.5, 0.5);
        PooledData {
            rngs,
            px_coords: vec![],
            px_samples: vec![],
            msaa_distr: msaa_dist,
        }
    }
}

// endregion Pooled/Cached Data

// region High-level Rendering

impl<Obj: Object, Sky: Skybox, Rng: RngCore + Send + SeedableRng> Renderer<Obj, Sky, Rng> {
    // TODO: Should `render()` be fallible?
    pub fn render(&mut self) -> Render<Image> {
        profile_function!();

        // Render image, and collect stats

        let start = puffin::now_ns();
        let num_threads = self.thread_pool.current_num_threads();

        let image = match self.camera.calculate_viewport() {
            Err(err) => {
                trace!(target: RENDERER, ?err, "couldn't calculate viewport");
                let [w, h] = self.options.dims();
                Self::render_failed(w, h)
            }
            Ok(viewport) => {
                let interval = Interval::from(1e-3..Number::MAX);
                Self::render_actual(
                    &self.thread_pool,
                    &self.data_pool,
                    &mut self.accum_buffer,
                    &self.scene,
                    &self.options,
                    &viewport,
                    &interval,
                )
            }
        };

        let end = puffin::now_ns();
        let duration = Duration::from_nanos(end.abs_diff(start));

        Render {
            img: image,
            stats: RenderStats {
                duration,
                num_threads,
                opts: self.options,
                accum_frames: self.accum_buffer.frame_count(),
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
        thread_pool: &ThreadPool,
        data_pool: &opool::Pool<PooledDataAllocator, PooledData<Rng>>,
        accum_buffer: &mut AccumulationBuffer,
        scene: &Scene<Obj, Sky>,
        render_opts: &RenderOpts,
        viewport: &Viewport,
        interval: &Interval<Number>,
    ) -> Image {
        profile_function!();

        let [w, h] = render_opts.dims();

        let mut dest_img = Image::new_blank(w, h); // Output image
        let accum = accum_buffer.new_frame([w, h]);

        thread_pool.install(|| {
            let pixels = Zip::indexed(accum.deref_mut())
                .and(dest_img.deref_mut())
                .into_par_iter()
                // Return on panic as fast as possible; don't keep processing all the pixels on panic
                // Otherwise we get (literally) millions of panics (1 per pixel) which just hangs the renderer as it prints
                .panic_fuse();

            pixels.for_each_init(
                || {
                    let profiler_scope = puffin::profile_scope_custom!("inner");

                    // Pull values from our thread pool
                    // We hold them for the duration of each work segment, so we don't pull/push each pixel
                    (profiler_scope, data_pool.get())
                },
                // Process each pixel
                |(_scope, pooled), ((x, y), accum, dest)| {
                    let sample = Self::render_px_msaa(scene, render_opts, viewport, interval, x, y, pooled.deref_mut());
                    accum.insert_sample(sample);
                    *dest = accum.get();
                },
            );
        });

        return dest_img;
    }
}

// endregion High-level Rendering

// region Low-level Rendering

impl<Obj: Object, Sky: Skybox, Rng: RngCore> Renderer<Obj, Sky, Rng> {
    /// Renders a single pixel in the scene, and returns the colour
    ///
    /// Takes into account [`RenderOpts::msaa`]
    fn render_px_msaa(
        scene: &Scene<Obj, Sky>,
        opts: &RenderOpts,
        viewport: &Viewport,
        interval: &Interval<Number>,
        x: usize,
        y: usize,
        pooled_data: &mut PooledData<Rng>,
    ) -> Colour {
        let sample_count = opts.samples.get();

        let PooledData {
            px_coords: sample_coords,
            px_samples: samples,
            msaa_distr,
            rngs: [rng_sample, rng_render],
        } = pooled_data;

        // Samples are chosen stratified within the area of the pixel.
        // To keep things O(Samples) not O(Samples^2), we might have to skip stratifying some samples
        sample_coords.resize(sample_count, Vector2::ZERO);
        let px_centre = Vector2::new(x as Number, y as Number);

        let stratify_dim = sample_count.sqrt();
        let stratify_dim_inv = 1.0 / stratify_dim as Number;
        for i in 0..stratify_dim {
            for j in 0..stratify_dim {
                let rand: Vector2 = [msaa_distr.sample(rng_sample), msaa_distr.sample(rng_sample)].into();
                let stratify_coord: Vector2 = [i as Number, j as Number].into();
                // Make sure to divide `randomness` and `stratify_coord`
                // so that it doesn't spill out across the stratified sub-pixels
                let coord: Vector2 = px_centre + (rand * stratify_dim_inv) + (stratify_coord * stratify_dim_inv);
                sample_coords[i + (stratify_dim * j)] = coord;
            }
        }
        // The remainder are fully random
        for i in (stratify_dim * stratify_dim)..sample_count {
            sample_coords[i] =
                px_centre + Vector2::from([msaa_distr.sample(rng_sample), msaa_distr.sample(rng_sample)]);
        }

        samples.clear();
        sample_coords
            .iter()
            .map(|&Vector2 { x, y }| Self::render_px_once(scene, viewport, opts, interval, x, y, rng_render))
            .inspect(|p| validate::colour(p))
            .collect_into(samples);

        let overall_colour = {
            let accum: Colour = samples.iter().copied().sum();
            let count = samples.len() as Channel;
            accum / count // Mean
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
        interval: &Interval<Number>,
        x: Number,
        y: Number,
        rng: &mut Rng,
    ) -> Colour {
        let ray = viewport.calc_ray(x, y, opts.width.get() as Number, opts.height.get() as Number, rng);
        validate::ray(ray);
        let mode = opts.mode;

        if mode == RenderMode::PBR {
            return Self::ray_colour_recursive(scene, &ray, opts, interval, 0, rng);
        }

        let Some(ObjectIntersection {
            intersection: intersect,
            material,
        }) = Self::calculate_intersection(scene, &ray, interval, rng)
        else {
            return scene.skybox.sky_colour(&ray);
        };
        validate::intersection(ray, &intersect, interval);

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
            RenderMode::FrontFace => COLOURS[intersect.front_face as usize],
            RenderMode::Side => {
                // TODO: Make `Object: Hash`
                let hash = intersect.side % (N_COL - 1) + 1;
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
                Colour::lerp(a, b, frac)
            }
        };
    }

    /// Calculates the nearest intersection in the scene for the given ray
    fn calculate_intersection<'o>(
        scene: &'o Scene<Obj, Sky>,
        ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut Rng,
    ) -> Option<ObjectIntersection<'o, Obj::Mat>> {
        scene.objects.full_intersect(ray, interval, rng)
    }

    /// Recursive function that calculates the colour in the scene for a given ray.
    ///
    /// # Recursion
    /// This will recurse each time the ray scatters off an object in the scene, up to a limit imposed by [RenderOpts::bounces].
    /// It should be fine for all *reasonable* bounce limits (~200), but will most likely overflow the stack past that.
    fn ray_colour_recursive(
        scene: &Scene<Obj, Sky>,
        in_ray: &Ray,
        opts: &RenderOpts,
        interval: &Interval<Number>,
        depth: usize,
        rng: &mut Rng,
    ) -> Colour {
        if depth > opts.ray_depth {
            return Colour::from([0.; 3]);
        }

        // Intersect
        let Some(ObjectIntersection { intersection, material }) =
            Self::calculate_intersection(scene, in_ray, interval, rng)
        else {
            return scene.skybox.sky_colour(in_ray);
        };
        validate::intersection(in_ray, &intersection, interval);

        let col_emitted = {
            let col = material.emitted_light(in_ray, &intersection, rng);
            validate::colour(&col);
            col
        };

        // PERF: Chose num samples as a tradeoff between not allocating on heap, and wasting stack space
        //  If we go above 8 branches, the sheer amount of intersections will have a much bigger perf impact
        //  than any heap allocations. Also we want to make sure we don't overflow the stack with high depths
        let mut scatter_samples = SmallVec::<[Colour; 8]>::new();

        // NOTE: The number of rays increases almost exponentially, with the number of branches and bounce depth
        //  For a given `d: depth, b: branches`, we check `(b^(d+1) - 1) / (b - 1)` rays, per pixel
        //  Normally, any more than 4 branches is visually indistinguishable, as well as crazy slow

        // Calculate the lighting samples for the scattered ray
        for _ in 0..opts.ray_branching.get() {
            let scatter_ray = {
                let Some(future_ray_dir) = material.scatter(in_ray, &intersection, rng) else {
                    scatter_samples.push(Colour::BLACK);
                    continue;
                };
                validate::normal3(&future_ray_dir);
                let future_ray = Ray::new(intersection.pos_w, future_ray_dir);
                validate::ray(future_ray);
                future_ray
            };

            // Follow ray and calculate future bounces
            let scatter_col = {
                let col_future = Self::ray_colour_recursive(scene, &scatter_ray, opts, interval, depth + 1, rng);
                validate::colour(&col_future);
                let col_scattered = material.reflected_light(in_ray, &intersection, &scatter_ray, &col_future, rng);
                validate::colour(&col_scattered);
                col_scattered
            };

            scatter_samples.push(scatter_col);
        }

        let col_scatter_sum = scatter_samples.iter().copied().sum::<Colour>();
        let col_scattered = col_scatter_sum / scatter_samples.len() as Channel;

        col_emitted + col_scattered
    }
}

// endregion Low-level Rendering
