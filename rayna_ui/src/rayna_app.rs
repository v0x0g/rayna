use crate::def::targets::UI;
use crate::def::ui_str;
use crate::def::ui_str::LENGTH_UNIT;
use crate::ext::UiExt;
use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::integration::Integration;
use egui::{ColorImage, RichText, TextureHandle, TextureOptions};
use image::buffer::ConvertBuffer;
use image::RgbaImage;
use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::shared::scene::Scene;
use rayna_ui_base::app::App;
use std::num::NonZeroUsize;
use tracing::{info, trace, warn};

pub struct RaynaApp {
    // Engine things
    render_opts: RenderOpts,
    scene: Scene,

    // Display things
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
        let scene = Scene::simple();
        let render_opts = Default::default();
        Self {
            render_opts,
            render_buf_tex: None,
            render_display_size: egui::vec2(1.0, 1.0),
            integration: Integration::new(&render_opts, &scene),
            scene,
        }
    }
}

impl App for RaynaApp {
    fn on_update(&mut self, ctx: &egui::Context) -> () {
        let mut render_opts_dirty = false;
        let mut scene_dirty = false;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // TODO: QUIT HANDLING
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.group(|ui| {
                ui.heading("Render Options");

                // DragValues: Image Dimensions

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

                // Button: Fill Image Dimensions

                if ui.button("Fill Canvas").clicked() {
                    render_opts_dirty = true;
                    self.render_opts.width = NonZeroUsize::new(self.render_display_size.x as usize)
                        .unwrap_or(NonZeroUsize::MIN);
                    self.render_opts.height =
                        NonZeroUsize::new(self.render_display_size.y as usize)
                            .unwrap_or(NonZeroUsize::MIN);
                }
            });

            ui.group(|ui| {
                ui.heading("Camera");

                let cam = &mut self.scene.camera;
                //
                ui.label("look from");
                scene_dirty |= ui.vec3_edit(&mut cam.look_from, LENGTH_UNIT).changed();
                ui.label("look towards");
                scene_dirty |= ui.vec3_edit(&mut cam.look_towards, LENGTH_UNIT).changed();
                ui.label("upwards");
                scene_dirty |= ui.vec3_edit(&mut cam.up_vector, "").changed();
            });

            ui.group(|ui| {
                ui.heading("Scene");
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

        if scene_dirty {
            info!(target: UI, scene = ?self.scene, "scene dirty, sending to worker");

            if let Err(err) = self
                .integration
                .send_message(MessageToWorker::SetScene(self.scene.clone()))
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
