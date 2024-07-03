//! Module containing UI extension traits

use crate::ui_val::*;
use egui::{Response, Widget};
use rayna_engine::core::types::{Angle, Number};
use std::{num::NonZeroUsize, ops::DerefMut};

pub trait UiExt {
    fn edit_vec3(&mut self, vec: impl DerefMut<Target = [Number; 3]>, suffix: &str, speed: f64) -> Response;
    fn edit_nonzero_usize(&mut self, val: &mut NonZeroUsize, suffix: &str, speed: f64) -> Response;
    fn edit_usize(&mut self, val: &mut usize, suffix: &str, speed: f64) -> Response;
    fn edit_number(&mut self, val: &mut Number, suffix: &str, speed: f64) -> Response;
    fn edit_angle(&mut self, val: &mut Angle, range: (Number, Number)) -> Response;

    fn fill_available_width(&mut self);
    fn fill_available_height(&mut self);
}

impl UiExt for egui::Ui {
    fn edit_vec3(&mut self, mut vec: impl DerefMut<Target = [Number; 3]>, suffix: &str, speed: f64) -> Response {
        let vec = vec.deref_mut();

        self.columns(3, |cols| {
            let x = egui::DragValue::new(&mut vec[0])
                .suffix(suffix)
                .speed(speed)
                .ui(&mut cols[0]);
            let y = egui::DragValue::new(&mut vec[1])
                .suffix(suffix)
                .speed(speed)
                .ui(&mut cols[1]);
            let z = egui::DragValue::new(&mut vec[2])
                .suffix(suffix)
                .speed(speed)
                .ui(&mut cols[2]);

            x | y | z
        })
    }

    fn edit_nonzero_usize(&mut self, val: &mut NonZeroUsize, suffix: &str, speed: f64) -> Response {
        let mut temp = val.get();

        let response = egui::DragValue::new(&mut temp).suffix(suffix).speed(speed).ui(self);
        *val = NonZeroUsize::new(temp).unwrap_or(NonZeroUsize::MIN);

        response
    }

    fn edit_usize(&mut self, val: &mut usize, suffix: &str, speed: f64) -> Response {
        egui::DragValue::new(val).suffix(suffix).speed(speed).ui(self)
    }

    fn edit_number(&mut self, val: &mut Number, suffix: &str, speed: f64) -> Response {
        egui::DragValue::new(val).suffix(suffix).speed(speed).ui(self)
    }

    fn edit_angle(&mut self, val: &mut Angle, range: (Number, Number)) -> Response {
        egui::Slider::from_get_set(range.0..=range.1, |o| {
            if let Some(angle) = o {
                *val = Angle::from_degrees(angle);
            }
            val.to_degrees()
        })
        .suffix(UNIT_DEG)
        .min_decimals(1)
        .ui(self)
    }

    fn fill_available_width(&mut self) { self.allocate_space(egui::Vec2::new(self.available_width(), 0.0)); }
    fn fill_available_height(&mut self) { self.allocate_space(egui::Vec2::new(0.0, self.available_height())); }
}
