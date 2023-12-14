use crate::render::render_opts::RenderOpts;
use crate::shared::ray::Ray;
use num_traits::FloatConst;
use rayna_shared::def::types::{Number, Vector};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use valuable::Valuable;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// Position the camera is located at
    pub look_from: Vector,
    ///  A point the camera should look towards - this will be the focus of the camera
    pub look_towards: Vector,
    /// Vector direction the camera considers 'upwards'.
    /// Use this to rotate the camera around the central view ray (`look_from` -> `look_towards`)
    /// - inverting this is like rotating the camera upside-down
    pub up_vector: Vector,
    /// Angle in degrees for the vertical field of view
    pub vertical_fov: Number,
    /// Radius of the simulated lens. Larger values increase blur
    pub lens_radius: Number,
    /// Distance from the camera at which rays are perfectly in focus
    pub focus_dist: Number,
}

#[derive(Error, Copy, Clone, Debug, Valuable)]
pub enum CamInvalidError {
    /// The provided `up_vector` was too close to zero, and so vector normalisation failed
    #[error("the provided `up_vector` couldn't be normalised (too small)")]
    UpVectorTooSmall,
    /// The calculated look direction was not valid (likely too close to zero).
    /// Probably due to `look_from` and `look_towards` being too close to each other
    #[error("the calculated look direction was invalid (likely close to zero)")]
    LookDirectionInvalid,
}

impl Camera {
    /// A method for creating a camera
    ///
    /// # Return
    /// Returns a viewport with values according to the current camera state,
    /// unless the camera is currently in an invalid state
    ///
    /// # Errors
    /// This will return [`Option::Err`] if the `up_vector` points in the same direction as
    /// the forward vector (`look_from -> look_towards`),
    /// equivalent to the case where `cross(look_direction, up_vector) == Vec3::Zero`
    pub fn calculate_viewport(&self, render_opts: RenderOpts) -> Result<Viewport, CamInvalidError> {
        // must be normalised
        let up_vector = self
            .up_vector
            .try_normalize()
            .ok_or(CamInvalidError::UpVectorTooSmall)?;

        let theta = self.vertical_fov * (Number::PI() / 180.);
        let h = Number::tan(theta / 2.);
        let viewport_height = 2. * h;
        let aspect_ratio = render_opts.aspect_ratio();
        let viewport_width = aspect_ratio * viewport_height;

        //Magic that lets us position and rotate the camera
        let look_dir = (self.look_from - self.look_towards)
            .try_normalize()
            .ok_or(CamInvalidError::LookDirectionInvalid)?;
        let Some(norm_cross_up_look) = Vector::cross(up_vector, look_dir).try_normalize() else {
            return Err(CamInvalidError::LookDirectionInvalid);
        };
        let u = norm_cross_up_look;
        let v = Vector::cross(look_dir, u);

        let horizontal = u * viewport_width * self.focus_dist;
        let vertical = v * viewport_height * self.focus_dist;
        let uv_origin =
            self.look_from - (horizontal / 2.) - (vertical / 2.) - (look_dir * self.focus_dist);

        // Extract out some computations from the ray calculations
        // To save a bit of perf
        let img_width = render_opts.width.get() as Number;
        let img_height = render_opts.width.get() as Number;
        let horizontal_norm = horizontal / img_width;
        let vertical_norm = horizontal / img_height;

        Ok(Viewport {
            u_dir: u,
            v_dir: v,
            lens_radius: self.lens_radius,
            look_from: self.look_from,
            uv_origin,
            horizontal_norm,
            vertical_norm,
            width: render_opts.width.get() as Number,
            height: render_opts.width.get() as Number,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    u_dir: Vector,
    v_dir: Vector,
    lens_radius: Number,
    look_from: Vector,
    /// The lower left corner of the viewport
    uv_origin: Vector,
    horizontal_norm: Vector,
    vertical_norm: Vector,
    width: Number,
    height: Number,
}

impl Viewport {
    /// Calculates the view ray for a given pixel at the coords `(p_x, p_y)`
    /// (screen-space, top-left to bot-right)
    pub fn calc_ray(&self, p_x: usize, p_y: usize) -> Ray {
        /*
           How this works is all pixels have their rays originating at the same point
           `look_from` (with a slight jitter from the randomness for DOF),
           and their direction depends on the pixel's position on screen (its UV coords)
        */

        // Don't need to normalise since we divided by `img_width`/`image_height` in the ctor for the viewport
        let u = p_x as Number / self.width;
        let v = 1.0 - (p_y as Number / self.height); // Flip Y for img coords -> world coords

        let pos = Vector::new((u - 0.5) * 4., (v - 0.5) * 4., -1.) - self.look_from;
        let dir = Vector::new(0., 0., 1.);

        return Ray::new(pos, dir);

        let rand = Vector::ZERO * self.lens_radius; // Random offset to simulate DOF
        let offset = (self.u_dir * rand.x) + (self.v_dir * rand.y); // Shift pixel origin slightly
        let origin = self.look_from + offset;
        let direction =
            (self.uv_origin + (self.horizontal_norm * u) + (self.vertical_norm * v)) - origin;
        return Ray::new(origin, direction);
    }
}
