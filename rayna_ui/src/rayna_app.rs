use crate::def::targets::UI;
use crate::def::ui_val::*;
use crate::ext::UiExt;
use crate::integration::message::MessageToWorker;
use crate::integration::Integration;
use egui::load::SizedTexture;
use egui::{Context, RichText, TextureHandle, TextureOptions};
use puffin::{profile_function, profile_scope};
use rayna_engine::render::render::RenderStats;
use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::shared::scene::Scene;
use rayna_ui_base::app::App;
use std::num::NonZeroUsize;
use tracing::{error, info, trace, warn};

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
    render_stats: RenderStats,

    // The rest
    integration: Integration,
}

impl RaynaApp {
    /// Creates a new app instance, with an [`Context`] for configuring the app
    pub fn new_ctx(_ctx: &Context) -> Self {
        info!(target: UI, "ui app init");
        let scene = Scene::simple();
        let render_opts = Default::default();
        Self {
            render_opts,
            render_buf_tex: None,
            render_display_size: egui::vec2(1.0, 1.0),
            integration: Integration::new(&render_opts, &scene),
            scene,
            render_stats: Default::default(),
        }
    }
}

impl App for RaynaApp {
    fn on_update(&mut self, ctx: &Context) -> () {
        // puffin::GlobalProfiler::lock().new_frame(); // Mark start of frame

        profile_function!();

        self.process_worker_messages();
        self.process_worker_render(ctx);

        let mut render_opts_dirty = false;
        let mut scene_dirty = false;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            profile_scope!("panel/top");

            egui::menu::bar(ui, |ui| {
                // TODO: QUIT HANDLING
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            profile_scope!("panel/left");

            ui.group(|ui| {
                profile_scope!("sec/render_opts");

                ui.heading("Render Options");

                // DragValues: Image Dimensions

                let mut w = self.render_opts.width.get();
                let mut h = self.render_opts.height.get();

                ui.label("Image Width");
                let w_drag = ui.add(egui::DragValue::new(&mut w).suffix(UNIT_PX));
                ui.label("Image Height");
                let h_drag = ui.add(egui::DragValue::new(&mut h).suffix(UNIT_PX));

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
                profile_scope!("sec/camera");

                ui.heading("Camera");

                let cam = &mut self.scene.camera;
                ui.label("look from");
                scene_dirty |= ui.vec3_edit(&mut cam.look_from, UNIT_LEN).changed();
                ui.label("look towards");
                scene_dirty |= ui.vec3_edit(&mut cam.look_towards, UNIT_LEN).changed();
                ui.label("upwards");
                scene_dirty |= ui.vec3_edit(&mut cam.up_vector, "").changed();
                ui.label("fov");
                scene_dirty |= ui
                    .add(
                        egui::DragValue::new(&mut cam.vertical_fov)
                            .suffix(UNIT_DEG)
                            .clamp_range(0.0..=180.0)
                            .min_decimals(1)
                            .speed(DRAG_SLOW),
                    )
                    .changed();
            });

            ui.group(|ui| {
                profile_scope!("sec/scene");

                ui.heading("Scene");
            });

            unsafe {
                static mut ENABLE: bool = false;
                if ui.checkbox(&mut ENABLE, "profiling enable").changed() {
                    puffin::set_scopes_on(ENABLE);
                }
            }
        });

        // Central panel contains the main render window
        // Must come after all other panels
        egui::CentralPanel::default().show(ctx, |ui| {
            profile_scope!("panel/central");

            let avail_space = ui.available_size();
            self.render_display_size = avail_space;
            if let Some(tex_handle) = &mut self.render_buf_tex {
                ui.image(SizedTexture {
                    id: tex_handle.id(),
                    size: avail_space,
                });
            } else {
                ui.label(RichText::new("No texture").size(20.0));
            }
        });

        if render_opts_dirty {
            profile_scope!("update_render_opts");
            info!(target: UI, render_opts = ?self.render_opts, "render opts dirty, sending to worker");

            if let Err(err) = self
                .integration
                .send_message(MessageToWorker::SetRenderOpts(self.render_opts))
            {
                warn!(target: UI, ?err)
            }
        }

        if scene_dirty {
            profile_scope!("update_scene");
            info!(target: UI, scene = ?self.scene, "scene dirty, sending to worker");

            if let Err(err) = self
                .integration
                .send_message(MessageToWorker::SetScene(self.scene.clone()))
            {
                warn!(target: UI, ?err)
            }
        }

        // Continuously update UI
        ctx.request_repaint();
    }

    fn on_shutdown(&mut self) -> () {
        info!(target: UI, "ui app shutdown")
    }
}

impl RaynaApp {
    /// Tries to receive the next render frame from the worker
    fn process_worker_render(&mut self, ctx: &Context) {
        profile_function!();

        let Some(res) = self.integration.try_recv_render() else {
            return;
        };
        let Ok(render) = res else {
            warn!(target: UI, ?res);
            return;
        };

        // let render = match res {
        //     Err(err) => {
        //         warn!(target: UI, ?err);
        //         return;
        //     }
        //     Ok(r) => r,
        // };

        trace!(target: UI, "received new frame from worker");

        {
            profile_scope!("update_tex");
            match &mut self.render_buf_tex {
                None => {
                    profile_scope!("tex_load");
                    self.render_buf_tex = Some(ctx.load_texture(
                        "render_buffer_texture",
                        render.img,
                        TextureOptions::default(),
                    ))
                }
                Some(tex) => {
                    profile_scope!("tex_set");
                    tex.set(render.img, TextureOptions::default())
                }
            }
        }

        self.render_stats = render.stats;
    }

    /// Processes the messages from the worker
    fn process_worker_messages(&mut self) {
        profile_function!();

        while let Some(res) = self.integration.try_recv_message() {
            trace!(target: UI, ?res, "got message from worker");

            match res {
                Err(err) => {
                    warn!(target: UI, ?err)
                }

                Ok(msg) => {
                    // Don't have any messages implemented currently
                    error!(target: UI, ?msg, "TODO: Implement message handling")
                }
            }
        }
    }
}
