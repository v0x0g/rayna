use crate::render::render_opts::RenderOpts;
use crate::shared::ray::Ray;
use rayna_shared::def::types::{Number, Vector};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use valuable::Valuable;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// Position the camera is located at
    pub look_from: Vector,
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
        let img_width = render_opts.width.get() as Number;
        let img_height = render_opts.height.get() as Number;
        let aspect_ratio = img_width / img_height;

        // Determine viewport dimensions.
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * (img_width / img_height);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = Vector::new(viewport_width, 0., 0.);
        let viewport_v = Vector::new(0., -viewport_height, 0.);

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u / img_width;
        let pixel_delta_v = viewport_v / img_height;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left =
            self.look_from - Vector::new(0., 0., focal_length) - viewport_u / 2. - viewport_v / 2.;
        let uv_origin = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

        Ok(Viewport {
            centre: self.look_from,
            pixel_delta_u,
            pixel_delta_v,
            width: img_width,
            height: img_height,
            uv_origin,
            aspect_ratio,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    centre: Vector,
    uv_origin: Vector,
    pixel_delta_u: Vector,
    pixel_delta_v: Vector,
    width: Number,
    height: Number,
    aspect_ratio: Number,
}

impl Viewport {
    /// Calculates the view ray for a given pixel at the coords `(px, py)`
    /// (screen-space, top-left to bot-right)
    pub fn calc_ray(&self, px: Number, py: Number) -> Ray {
        let pixel_center = self.uv_origin + (self.pixel_delta_u * px) + (self.pixel_delta_v * py);
        let ray_direction = pixel_center - self.centre;
        Ray::new(pixel_center, ray_direction)
    }
}
