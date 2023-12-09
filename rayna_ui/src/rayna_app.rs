use crate::definitions::ui_str;
use egui::{ColorImage, RichText, TextureHandle, TextureOptions};
use image::buffer::ConvertBuffer;
use image::RgbaImage;
use nonzero::nonzero;
use num_traits::ToPrimitive;
use rayna_core::def::types::{ImgBuf, Pix};
use rayna_ui_base::app::App;
use std::num::NonZeroUsize;

pub struct RaynaApp {
    /// The target image dimensions we want, stored as `[width, height]`
    target_img_dims: [NonZeroUsize; 2],
    /// A handle to the texture that holds the current render buffer
    render_buf_tex: Option<TextureHandle>,
    render_display_size: egui::Vec2,
}

impl RaynaApp {
    pub fn new_ctx(_ctx: &egui::Context) -> Self {
        Self {
            target_img_dims: [nonzero!(1_usize), nonzero!(1_usize)],
            render_buf_tex: None,
            render_display_size: egui::vec2(1.0, 1.0),
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

        let mut dirty = false;

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("Render Options");

            ui.group(|ui| {
                // Have to do a bit of magic since we can't use NonZeroUsize directly
                ui.label("Image Width");
                let mut w = self.target_img_dims[0].get();
                dirty |= ui
                    .add(egui::DragValue::new(&mut w).suffix(ui_str::PIXELS_UNIT))
                    .changed();
                self.target_img_dims[0] = NonZeroUsize::new(w).unwrap_or(NonZeroUsize::MIN);

                ui.label("Image Height");
                let mut h = self.target_img_dims[1].get();
                dirty |= ui
                    .add(egui::DragValue::new(&mut h).suffix(ui_str::PIXELS_UNIT))
                    .changed();
                self.target_img_dims[1] = NonZeroUsize::new(h).unwrap_or(NonZeroUsize::MIN);

                if ui.button("Fill Canvas").clicked() {
                    dirty = true;
                    self.target_img_dims[0] =
                        NonZeroUsize::new(self.render_display_size.x as usize)
                            .unwrap_or(NonZeroUsize::MIN);
                    self.target_img_dims[1] =
                        NonZeroUsize::new(self.render_display_size.y as usize)
                            .unwrap_or(NonZeroUsize::MIN);
                }
            });
        });

        if dirty {
            // ===== THIS WOULD BE INSIDE RENDERER/CORE =====
            let img_orig = {
                let [w, h] = self
                    .target_img_dims
                    .map(|x| x.get().to_u32())
                    .map(|d| d.expect("image dims failed to fit inside u32"));
                let mut img = ImgBuf::new(w, h);
                img.enumerate_pixels_mut().for_each(|(x, y, p)| {
                    *p = if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                        Pix::from([1.0; 3])
                    } else {
                        Pix::from([0.0, 1.0, 0.0])
                    }
                });
                img
            };

            // Pretend we have 'received' this image from the renderer now
            // We now translate the image into an egui-appropriate one
            // And update the texture from it
            let img_as_rgba: RgbaImage = img_orig.convert();
            let img_as_egui = ColorImage::from_rgba_unmultiplied(
                [img_orig.width() as usize, img_orig.height() as usize],
                img_as_rgba.as_raw().as_slice(),
            );

            match &mut self.render_buf_tex {
                None => {
                    self.render_buf_tex = Some(ctx.load_texture(
                        "render_buffer_texture",
                        img_as_egui,
                        TextureOptions::default(),
                    ))
                }
                Some(tex) => tex.set(img_as_egui, TextureOptions::default()),
            }
        }

        // Central panel contains the main render window
        // Must come after all other panels
        egui::CentralPanel::default().show(ctx, |ui| {
            let avail_space = ui.available_size();
            self.render_display_size = avail_space;
            if let Some(tex_id) = &mut self.render_buf_tex {
                ui.image(tex_id, avail_space);
            } else {
                ui.label(RichText::new("No texture").size(20.0));
            }
        });
    }

    fn on_shutdown(self) -> () {
        println!("rayna_app::shutdown")
    }
}
