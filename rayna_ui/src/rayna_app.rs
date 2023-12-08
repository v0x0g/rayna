use crate::definitions::ui_str;
use nonzero::nonzero;
use rayna_ui_base::app::App;
use std::num::NonZeroUsize;

pub struct RaynaApp {
    img_dims: (NonZeroUsize, NonZeroUsize),
}

impl RaynaApp {
    pub fn new_ctx(_ctx: &egui::Context) -> Self {
        Self {
            img_dims: (nonzero!(1920_usize), nonzero!(1080_usize)),
        }
    }
}

impl App for RaynaApp {
    fn on_update(&mut self, ctx: &egui::Context) -> () {
        // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // TODO: QUIT HANDLING

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        // Central panel contains the main render window
        // Must come after all other panels
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Render");

            ui.group(|ui| {
                // Have to do a bit of magic since we can't use NonZeroUsize directly
                ui.label("width");
                let mut w = self.img_dims.0.get();
                ui.add(egui::DragValue::new(&mut w).suffix(ui_str::PIXELS_UNIT));
                self.img_dims.0 = NonZeroUsize::new(w).unwrap_or(NonZeroUsize::MIN);

                ui.label("height");
                let mut h = self.img_dims.1.get();
                ui.add(egui::DragValue::new(&mut h).suffix(ui_str::PIXELS_UNIT));
                self.img_dims.1 = NonZeroUsize::new(h).unwrap_or(NonZeroUsize::MIN);
            });

            // ui.image(
            //     tex_id,
            //     egui::vec2(self.img_dims.0.get() as f32, self.img_dims.1.get() as f32),
            // );

            // ui.group(|ui| egui::Slider::new(&mut self.img_dims.1));
            //
            // ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            // if ui.button("Increment").clicked() {
            //     self.value += 1.0;
            // }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
            });
        });
    }

    fn on_shutdown(self) -> () {
        println!("rayna_app::shutdown")
    }
}
