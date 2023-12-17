use crate::ui_val::*;
use egui::{Response, Widget};
use rayna_shared::def::types::Number;
use std::borrow::BorrowMut;

pub trait UiExt {
    fn vec3_edit(&mut self, vec: impl BorrowMut<[Number; 3]>, suffix: &str) -> Response;
}

impl UiExt for egui::Ui {
    fn vec3_edit(&mut self, mut vec: impl BorrowMut<[Number; 3]>, suffix: &str) -> Response {
        let vec = vec.borrow_mut();
        self.horizontal(|ui| {
            let x = egui::DragValue::new(&mut vec[0])
                .suffix(suffix)
                .speed(DRAG_SLOW)
                .ui(ui);
            let y = egui::DragValue::new(&mut vec[1])
                .suffix(suffix)
                .speed(DRAG_SLOW)
                .ui(ui);
            let z = egui::DragValue::new(&mut vec[2])
                .suffix(suffix)
                .speed(DRAG_SLOW)
                .ui(ui);

            x | y | z
        })
        .inner
    }
}
