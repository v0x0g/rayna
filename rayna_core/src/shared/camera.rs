use crate::def::types::{Num, Vec3};
use crate::render::render_opts::RenderOpts;
use crate::shared::ray::Ray;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub focal_length: Num,
    pub pos: Vec3,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    u: Vec3,
    v: Vec3,
    delta_u: Vec3,
    delta_v: Vec3,
    upper_left: Vec3,
    pixel_origin: Vec3,
    focal_length: Num,
    cam_pos: Vec3,
}

impl Camera {
    pub fn calculate_viewport(&self, render_opts: RenderOpts) -> Viewport {
        let img_height = render_opts.height.get() as Num;
        let img_width = render_opts.width.get() as Num;
        let view_height = 2.0;
        let view_width = view_height * (img_width / img_height);
        let cam_pos = self.pos;
        let focal_length = self.focal_length;

        // Vectors across the horizontal and vertical viewport edges
        let u = Vec3::new(view_width, 0., 0.);
        let v = Vec3::new(0., -view_height, 0.);

        // Delta vectors from pixel to pixel
        let delta_u = u / view_width;
        let delta_v = v / view_height;

        // Location of first pixel (upper left corner of image)
        let upper_left = cam_pos - Vec3::new(0., 0., focal_length) - (u / 2.) - (v / 2.);
        let pixel_origin = upper_left + ((delta_u + delta_v) * 0.5);

        Viewport {
            u,
            v,
            delta_u,
            delta_v,
            upper_left,
            pixel_origin,
            cam_pos,
            focal_length,
        }
    }
}

impl Viewport {
    pub fn calculate_pixel_ray(&self, x: usize, y: usize) -> Ray {
        let pixel_centre =
            self.pixel_origin + (self.delta_u * x as Num) + (self.delta_v * y as Num);
        let ray_dir = pixel_centre - self.cam_pos;

        Ray::new(self.cam_pos, ray_dir)
    }
}
