use crate::def::ui_val::*;
use egui::{Response, Widget};
use rayna_shared::::types::Vec3;

pub trait UiExt {
    fn vec3_edit(&mut self, vec: &mut Vec3, suffix: &str) -> Response;
}

impl UiExt for egui::Ui {
    fn vec3_edit(&mut self, vec: &mut Vec3, suffix: &str) -> Response {
        self.horizontal(|ui| {
            let x = egui::DragValue::new(&mut vec.x)
                .suffix(suffix)
                .speed(DRAG_SLOW)
                .ui(ui);
            let y = egui::DragValue::new(&mut vec.y)
                .suffix(suffix)
                .speed(DRAG_SLOW)
                .ui(ui);
            let z = egui::DragValue::new(&mut vec.z)
                .suffix(suffix)
                .speed(DRAG_SLOW)
                .ui(ui);

            x | y | z
        })
        .inner
    }
}
