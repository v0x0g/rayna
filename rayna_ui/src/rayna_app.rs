use crate::def::targets::UI;
use crate::def::ui_str;
use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::integration::Integration;
use egui::{ColorImage, RichText, TextureHandle, TextureOptions};
use image::buffer::ConvertBuffer;
use image::RgbaImage;
use rayna_core::render::render_opts::RenderOpts;
use rayna_ui_base::app::App;
use std::num::NonZeroUsize;
use tracing::{info, trace, warn};

pub struct RaynaApp {
    render_opts: RenderOpts,
    /// A handle to the texture that holds the current render buffer
    render_buf_tex: Option<TextureHandle>,
    /// The amount of space available to display the rendered image in
    /// This is [`egui::Ui::available_size`] inside [egui::CentralPanel]
    /// Used by the "fit canvas to screen" button
    render_display_size: egui::Vec2,
    integration: Integration,
}

impl RaynaApp {
    /// Creates a new app instance, with an [`egui::Context`] for configuring the app
    pub fn new_ctx(_ctx: &egui::Context) -> Self {
        info!(target: UI, "ui app init");
        Self {
            render_opts: Default::default(),
            render_buf_tex: None,
            render_display_size: egui::vec2(1.0, 1.0),
            integration: Integration::new(),
        }
    }
}

impl App for RaynaApp {
    fn on_update(&mut self, ctx: &egui::Context) -> () {
        let mut render_opts_dirty = false;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // TODO: QUIT HANDLING
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("Render Options");

            ui.group(|ui| {
                // Image Dimensions

                let mut w = self.render_opts.width.get();
                let mut h = self.render_opts.height.get();

                ui.label("Image Width");
                let w_drag = ui.add(egui::DragValue::new(&mut w).suffix(ui_str::PIXELS_UNIT));
                ui.label("Image Height");
                let h_drag = ui.add(egui::DragValue::new(&mut h).suffix(ui_str::PIXELS_UNIT));

                render_opts_dirty |= w_drag.drag_released() || w_drag.lost_focus(); // don't use `.changed()` so it waits till interact complete
                render_opts_dirty |= h_drag.drag_released() || h_drag.lost_focus(); // don't use `.changed()` so it waits till interact complete

                self.render_opts.width = NonZeroUsize::new(w).unwrap_or(NonZeroUsize::MIN);
                self.render_opts.height = NonZeroUsize::new(h).unwrap_or(NonZeroUsize::MIN);

                // Fill Image Dimensions

                if ui.button("Fill Canvas").clicked() {
                    render_opts_dirty = true;
                    self.render_opts.width = NonZeroUsize::new(self.render_display_size.x as usize)
                        .unwrap_or(NonZeroUsize::MIN);
                    self.render_opts.height =
                        NonZeroUsize::new(self.render_display_size.y as usize)
                            .unwrap_or(NonZeroUsize::MIN);
                }
            });
        });

        if render_opts_dirty {
            info!(target: UI, render_opts = ?self.render_opts, "render opts dirty, sending to worker");

            if let Err(err) = self
                .integration
                .send_message(MessageToWorker::SetRenderOpts(self.render_opts))
            {
                warn!(target: UI, ?err)
            }
        }

        // Process any messages from the worker

        while let Some(res) = self.integration.try_recv_message() {
            trace!(target: UI, ?res, "got message from worker");

            match res {
                Err(err) => {
                    warn!(target: UI, ?err)
                }

                Ok(MessageToUi::RenderFrameComplete(img)) => {
                    trace!(target: UI, "received new frame from worker");

                    // Got a rendered image, translate to an egui-appropriate one

                    let img_as_rgba: RgbaImage = img.convert();
                    // SAFETY: This may panic if the data doesn't exactly match
                    //  between the image dims and the raw buffer
                    //  This *should* be fine as long as nothing in the [`image`] or [`epaint`] crate changes
                    let img_as_egui = ColorImage::from_rgba_unmultiplied(
                        [img.width() as usize, img.height() as usize],
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

    fn on_shutdown(&mut self) -> () {
        info!(target: UI, "ui app shutdown")
    }
}
