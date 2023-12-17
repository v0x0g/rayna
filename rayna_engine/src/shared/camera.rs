use crate::render::render_opts::RenderOpts;
use crate::shared::ray::Ray;
use glam::{DQuat, Vec4Swizzles};
use glamour::{AsRaw, ToRaw};
use rayna_shared::def::types::{Angle, Matrix4, Number, Point2, Point3, Vector2, Vector3, Vector4};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use valuable::Valuable;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// Position the camera is located at
    pub pos: Point3,
    /// Vertical FOV
    pub v_fov: Angle,
    pub fwd: Vector3,
    pub up: Vector3,
}

#[derive(Error, Copy, Clone, Debug, Valuable)]
pub enum CamInvalidError {
    /// The provided `up_vector` was too close to zero, and so vector normalisation failed
    #[error("the provided `up` vector couldn't be normalised (too small)")]
    UpVectorInvalid,
    /// The calculated look direction (forward vector) was not valid.
    #[error("the provided `fwd` vector couldn't be normalised (too small)")]
    ForwardVectorInvalid,
}

impl Camera {
    pub fn apply_motion(&mut self, pos_move: Vector3, dir_move: Vector2) {
        let right_dir = Vector3::cross(self.fwd, self.up).normalize();

        self.pos += self.up * pos_move.y;
        self.pos += self.fwd * pos_move.z;
        self.pos += right_dir * pos_move.x;

        let pitch_delta = dir_move.y;
        let yaw_delta = dir_move.x;

        // let pitch_quat = Transform3::from_axis_angle(right_dir, Angle::from_degrees(-pitch_delta));
        // let yaw_quat = Transform3::from_axis_angle(self.up, Angle::from_degrees(-yaw_delta));
        // let rot = pitch_quat * yaw_quat;
        // self.fwd = (rot.map_vector(self.fwd)).normalize();

        let pitch_quat = DQuat::from_axis_angle(right_dir.to_raw(), -pitch_delta);
        let yaw_quat = DQuat::from_axis_angle(self.up.to_raw(), -yaw_delta);
        let rot = DQuat::normalize(pitch_quat * yaw_quat);
        self.fwd = (rot * self.fwd).normalize();
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
    /// This is because it depends on variables such as rendering image dimensions ([RenderOpts.width])
    ///
    /// # Errors
    /// This will return [`Option::Err`] if the `up_vector` points in the same direction as
    /// the forward vector (`look_from -> look_towards`),
    /// equivalent to the case where `cross(look_direction, up_vector) == Vec3::Zero`
    pub fn calculate_viewport(
        &self,
        render_opts: &RenderOpts,
    ) -> Result<Viewport, CamInvalidError> {
        let img_width = render_opts.width.get() as Number;
        let img_height = render_opts.height.get() as Number;
        let aspect_ratio = img_width / img_height;

        let projection = Matrix4::perspective_rh(self.v_fov, aspect_ratio, 0.1, 100.);
        let inv_projection = projection.try_inverse().unwrap();

        let view = Matrix4::look_at_rh(self.pos, self.pos + self.fwd, self.up);
        let inv_view = view.try_inverse().unwrap();

        Ok(Viewport {
            pos: self.pos,
            inv_projection,
            inv_view,
            img_width,
            img_height,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pos: Point3,
    inv_projection: Matrix4,
    inv_view: Matrix4,
    img_width: Number,
    img_height: Number,
}

impl Viewport {
    /// Calculates the view ray for a given pixel at the coords `(px, py)`
    /// (screen-space, top-left to bot-right)
    pub fn calc_ray(&self, px: Number, py: Number) -> Ray {
        let screen_coord = Point2::new(px / self.img_width, py / self.img_height);
        // 0..1 -> -1..1 so that the camera is centred nicely (centre pixel is coord [0,0])
        // Also flip Y for image -> uv coords
        let screen_coord = Point2::new(screen_coord.x * 2. - 1., 1. - (screen_coord.y * 2.));
        let target = self.inv_projection * Vector4::new(screen_coord.x, screen_coord.y, 1., 1.);
        let homogenous = Vector3::from(target.as_raw().xyz() / target.w);
        let ray_dir = self.inv_view.transform_vector(homogenous).normalize();

        Ray::new(self.pos, ray_dir)
    }
}
