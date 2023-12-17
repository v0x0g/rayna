use crate::render::render_opts::RenderOpts;
use crate::shared::ray::Ray;
use glam::Vec4Swizzles;
use glamour::AsRaw;
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
    pub forward: Vector3,
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
    pub fn apply_motion(&mut self, pos_move: Vector3, _dir_move: Vector2) {
        // self.pos += self.u * pos_move.x;
        // self.pos += self.v * pos_move.y;
        // self.pos += self.w * pos_move.z;
        // TODO
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

        let view = Matrix4::look_at_rh(self.pos, self.pos + self.forward, self.up);
        let inv_view = view.try_inverse().unwrap();

        Ok(Viewport {
            pos: self.pos,
            projection,
            inv_projection,
            view,
            inv_view,
            img_width,
            img_height,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pos: Point3,
    projection: Matrix4,
    inv_projection: Matrix4,
    view: Matrix4,
    inv_view: Matrix4,
    img_width: Number,
    img_height: Number,
}

impl Viewport {
    /// Calculates the view ray for a given pixel at the coords `(px, py)`
    /// (screen-space, top-left to bot-right)
    pub fn calc_ray(&self, px: Number, py: Number) -> Ray {
        let screen_coord = Point2::new(px / self.img_width, py / self.img_height);
        let target = self.inv_projection * Vector4::new(screen_coord.x, screen_coord.y, 1., 1.);
        let homogenous = Vector3::from(target.as_raw().xyz() / target.w);
        let ray_dir = self.inv_view.transform_vector(homogenous).normalize();
        let ray_dir = Vector3::new(ray_dir.x, ray_dir.y, ray_dir.z);

        Ray::new(self.pos, ray_dir)
    }
}
