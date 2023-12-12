use egui::{Response, Widget};
use rayna_engine::def::types::Vec3;

pub trait UiExt {
    fn vec3_edit(&mut self, vec: &mut Vec3, suffix: &str) -> Response;
}

impl UiExt for egui::Ui {
    fn vec3_edit(&mut self, vec: &mut Vec3, suffix: &str) -> Response {
        self.horizontal(|ui| {
            egui::DragValue::new(&mut vec.x).suffix(suffix).ui(ui);
            egui::DragValue::new(&mut vec.y).suffix(suffix).ui(ui);
            egui::DragValue::new(&mut vec.z).suffix(suffix).ui(ui);
        })
        .response
    }
}
