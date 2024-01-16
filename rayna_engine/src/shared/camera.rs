use crate::render::render_opts::RenderOpts;
use crate::shared::ray::Ray;
use crate::shared::{rng, validate};
use puffin::profile_function;
use rand::Rng;
use rayna_shared::def::types::{Angle, Number, Point3, Transform3, Vector3};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use valuable::Valuable;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// Position the camera is located at
    pub pos: Point3,
    /// Vertical FOV
    pub v_fov: Angle,
    // TODO: DOF
    pub fwd: Vector3,
    pub focus_dist: Number,
    pub defocus_angle: Angle,
}

#[derive(Error, Copy, Clone, Debug, Valuable)]
pub enum CamInvalidError {
    /// The provided `up_vector` was too close to zero, and so vector normalisation failed
    #[error("the provided `up` vector couldn't be normalised (too small)")]
    UpVectorInvalid,
    /// The calculated look direction (forward vector) was not valid.
    #[error("the provided `fwd` vector couldn't be normalised (too small)")]
    ForwardVectorInvalid,
    /// The calculated field-of-view was not valid.
    #[error("the provided FOV was not valid")]
    FovInvalid,
    /// The calculated focal length was not valid. Try checking the focus distance is `> 0`
    #[error("the provided focal length was not valid")]
    FocalLengthInvalid,
}

impl Camera {
    pub fn apply_motion(&mut self, position: Vector3, rotate: Vector3, fov: Number) {
        profile_function!();

        let right_dir = Vector3::cross(self.fwd, Vector3::Y).normalize();

        self.pos += Vector3::Y * position.y;
        self.pos += self.fwd * position.z;
        self.pos += right_dir * position.x;

        let yaw_quat = Transform3::from_axis_angle(Vector3::Y, Angle::from_degrees(-rotate.x));
        let pitch_quat = Transform3::from_axis_angle(right_dir, Angle::from_degrees(-rotate.y));
        // TODO: Implement roll (rotation around `fwd` axis)
        self.fwd = (yaw_quat * pitch_quat).map_vector(self.fwd).normalize();

        self.v_fov += Angle::from_degrees(fov);
    }

    /// A method for creating a camera
    ///
    /// # Return
    /// Returns a viewport with values according to the current camera state,
    /// unless the camera is currently in an invalid state.
    ///
    /// # Note
    /// Once created, the viewport should be used for a single frame only, and is only valid given that the
    /// state of the renderer system does not mutate.
    /// This is because it depends on variables such as rendering image dimensions (e.g. [RenderOpts::width])
    ///
    /// # Errors
    /// This will return [Option::None] if the `up_vector` points in the same direction as
    /// the forward vector (`look_from -> look_towards`),
    /// equivalent to the case where `cross(look_direction, up_vector) == Vec3::Zero`
    pub fn calculate_viewport(&self, render_opts: &RenderOpts) -> Result<Viewport, CamInvalidError> {
        profile_function!();

        let img_width = render_opts.width.get() as Number;
        let img_height = render_opts.height.get() as Number;
        let aspect_ratio = img_width / img_height;
        // Not normally same in real cameras, but in our fake cam it is
        // Also seems to always be off by one
        let focal_length = self.focus_dist;

        if self.v_fov.radians == 0. {
            return Err(CamInvalidError::FovInvalid);
        }
        if focal_length == 0. {
            return Err(CamInvalidError::FocalLengthInvalid);
        }

        let theta = self.v_fov;
        let h = (theta / 2.).tan();
        let viewport_height = 2. * h * focal_length;
        let viewport_width = viewport_height * aspect_ratio;

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = -self.fwd.try_normalize().ok_or(CamInvalidError::ForwardVectorInvalid)?;
        let u = Vector3::cross(Vector3::Y, w)
            .try_normalize()
            .ok_or(CamInvalidError::ForwardVectorInvalid)?;
        let v = Vector3::cross(w, u);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = u * viewport_width; // Vector across viewport horizontal edge
        let viewport_v = -v * viewport_height; // Vector down viewport vertical edge

        // Calculate the horizontal and vertical delta vectors to the next pixel.
        let pixel_delta_u = viewport_u / img_width;
        let pixel_delta_v = viewport_v / img_height;

        let pos = self.pos;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left = pos - (w * focal_length) - (viewport_u / 2.) - (viewport_v / 2.);
        let pixel_0_0_pos = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

        // Calculate the camera defocus disk basis vectors.
        let defocus_radius = focal_length * (self.defocus_angle / 2.).tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        validate::point3(pos);
        validate::point3(pixel_0_0_pos);
        validate::vector3(pixel_delta_u);
        validate::vector3(pixel_delta_v);
        validate::vector3(defocus_disk_u);
        validate::vector3(defocus_disk_v);

        Ok(Viewport {
            pos,
            pixel_0_0_pos,
            pixel_delta_u,
            pixel_delta_v,
            defocus_disk_u,
            defocus_disk_v,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub pos: Point3,
    pub pixel_0_0_pos: Point3,
    pub pixel_delta_u: Vector3,
    pub pixel_delta_v: Vector3,
    pub defocus_disk_u: Vector3,
    pub defocus_disk_v: Vector3,
}

impl Viewport {
    /// Calculates the view ray for a given pixel at the coords `(px, py)`
    /// (screen-space, top-left to bot-right)
    ///
    /// # Note
    /// The values `px` and `py` should already have an appropriate pixel shift (+-0.5) applied,
    /// if MSAA is desired

    // PERF: This function is a rendering hotspot
    pub fn calc_ray(&self, px: Number, py: Number, rng: &mut impl Rng) -> Ray {
        // Pixel position
        let pixel_sample = self.pixel_0_0_pos + (self.pixel_delta_u * px) + (self.pixel_delta_v * py);

        // Ray starts off on the focus disk, and then goes through the pixel position
        let defocus_rand = rng::vector_in_unit_circle(rng);
        let ray_pos = self.pos + (self.defocus_disk_u * defocus_rand.x) + (self.defocus_disk_v * defocus_rand.y);
        let ray_dir = pixel_sample - ray_pos;

        return Ray::new(ray_pos, ray_dir);
    }
}
