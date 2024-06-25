use crate::core::types::{Angle, Number, Point3, Transform3, Vector3};
use crate::shared::ray::Ray;
use crate::shared::{rng, validate};
use puffin::profile_function;
use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use valuable::Valuable;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// Position the camera is located at
    pub pos: Point3,
    /// Vertical FOV
    pub v_fov: Angle,
    /// Direction the camera is looking in
    // TODO: Refactor this to store a quaternion for the rotation instead,
    //  and calculate fwd/up/right by multiplying basis vectors by rotation
    pub fwd: Vector3,
    /// Distance at which the camera is focused at
    pub focus_dist: Number,
    /// How large the defocus cone for each ray is.
    ///
    /// Larger angles increase defocus blur, zero gives perfect focus.
    pub defocus_angle: Angle,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Point3::ZERO,
            v_fov: Angle::from_degrees(45.0),
            fwd: Vector3::Z,
            focus_dist: 1.0,
            defocus_angle: Angle::from_degrees(0.0),
        }
    }
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
    /// Helper function to calculate the right vector
    fn right_dir(&self) -> Result<Vector3, CamInvalidError> {
        Vector3::cross(self.fwd, Vector3::Y)
            .try_normalize()
            .ok_or(CamInvalidError::ForwardVectorInvalid)
    }

    /// Applies a change in position to the camera
    ///
    /// Positive deltas imply a 'forwards' motion along the axis, negatives imply the opposite.
    /// E.g. `up_down = -2.0` is a downward motion of 2 units
    pub fn apply_pos_delta(
        &mut self,
        fwd_back: Number,
        right_left: Number,
        up_down: Number,
    ) -> Result<(), CamInvalidError> {
        let right_dir = Vector3::cross(self.fwd, Vector3::Y)
            .try_normalize()
            .ok_or(CamInvalidError::ForwardVectorInvalid)?;

        self.pos += Vector3::Y * up_down;
        self.pos += self.fwd * fwd_back;
        self.pos += right_dir * right_left;

        Ok(())
    }

    /// Applies rotation to the camera
    ///
    /// # Note
    /// Currently, `roll` is not implemented, and rotations around that axis will be silently ignored
    pub fn apply_rot_delta(&mut self, yaw: Angle, pitch: Angle, _roll: Angle) -> Result<(), CamInvalidError> {
        profile_function!();

        let right_dir = self.right_dir()?;

        let yaw_quat = Transform3::from_axis_angle(Vector3::Y, yaw);
        let pitch_quat = Transform3::from_axis_angle(right_dir, pitch);
        // TODO: Implement roll (rotation around `fwd` axis)
        self.fwd = (yaw_quat * pitch_quat)
            .map_vector(self.fwd)
            .try_normalize()
            .ok_or(CamInvalidError::ForwardVectorInvalid)?;

        Ok(())
    }

    /// A method for calculating the viewport from a camera
    ///
    /// # Return
    /// Returns a viewport with values according to the current camera state,
    /// unless the camera is currently in an invalid state.
    ///
    /// # Errors
    /// This will return a [`CamInvalidError`] if any of the settings of the camera are not valid, and so
    /// the viewport couldn't be calculated. This might happen if the FOV is zero ([`CamInvalidError::FovInvalid`]).
    pub fn calculate_viewport(&self) -> Result<Viewport, CamInvalidError> {
        profile_function!();

        // Not normally same in real cameras, but in our fake cam it is
        // Also seems to always be off by one
        let focal_length = self.focus_dist;

        if self.v_fov.radians == 0. {
            return Err(CamInvalidError::FovInvalid);
        }
        if focal_length == 0. {
            return Err(CamInvalidError::FocalLengthInvalid);
        }

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = -self.fwd.try_normalize().ok_or(CamInvalidError::ForwardVectorInvalid)?;
        let u = Vector3::cross(Vector3::Y, w)
            .try_normalize()
            .ok_or(CamInvalidError::ForwardVectorInvalid)?;
        let v = Vector3::cross(w, u);

        let pos = self.pos;

        // Calculate the location of the central pixel
        let pixel_center = pos - (w * focal_length);

        // Calculate the camera defocus disk basis vectors.
        let defocus_radius = focal_length * (self.defocus_angle / 2.).tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        let theta = self.v_fov;
        let h = (theta / 2.).tan();
        let viewport_size = 2. * h * focal_length;
        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = u * viewport_size; // Vector across viewport horizontal edge
        let viewport_v = -v * viewport_size; // Vector down viewport vertical edge

        validate::point3(pos);
        validate::point3(pixel_center);
        validate::vector3(viewport_u);
        validate::vector3(viewport_v);
        validate::vector3(defocus_disk_u);
        validate::vector3(defocus_disk_v);

        Ok(Viewport {
            pos,
            pixel_center,
            viewport_u,
            viewport_v,
            defocus_disk_u,
            defocus_disk_v,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub pos: Point3,
    pub pixel_center: Point3,
    pub viewport_u: Vector3,
    pub viewport_v: Vector3,
    pub defocus_disk_u: Vector3,
    pub defocus_disk_v: Vector3,
}

impl Viewport {
    /// Calculates the view ray for a given pixel at the coords `(px, py)`
    /// (screen-space, top-left to bot-right)
    ///
    /// # Parameters
    /// - `px`, `py`: Normalised pixel coordinates
    /// - `defocus_rng`: RNG to generate a random sample in the focus disk with
    ///
    /// # Note
    /// The values `px` and `py` should already have an appropriate pixel shift (+-0.5) applied,
    /// if MSAA is desired.
    pub fn calc_ray(&self, px: Number, py: Number, w: Number, h: Number, defocus_rng: &mut impl Rng) -> Ray {
        // FIXME: This function is a rendering hotspot

        // Normalise over the size of one dimension, so aspect is preserved
        // One dimension will be `-0.5..0.5`, other will have different magnitude
        // Also shift so `(0, 0)` is center

        // I chose height here to preserve the FOV (it's vertical FOV)
        // But another good option is the smaller dimension: `Number::min(w, h)`
        let norm_dim = h;
        let u = (px - (w / 2.)) / norm_dim;
        let v = (py - (h / 2.)) / norm_dim;

        // Pixel position
        let pixel_sample = self.pixel_center + (self.viewport_u * u) + (self.viewport_v * v);

        // Ray starts off on the focus disk, and then goes through the pixel position
        let defocus_rand = rng::vector_in_unit_circle(defocus_rng);
        let ray_pos = self.pos + (self.defocus_disk_u * defocus_rand.x) + (self.defocus_disk_v * defocus_rand.y);
        let ray_dir = pixel_sample - ray_pos;

        return Ray::new(ray_pos, ray_dir);
    }
}
