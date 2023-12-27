use crate::ext::UiExt;
use crate::integration::message::MessageToWorker;
use crate::integration::{Integration, IntegrationError};
use crate::profiler;
use crate::ui_val::{DRAG_SLOW, UNIT_DEG, UNIT_LEN, UNIT_PX};
use egui::load::SizedTexture;
use egui::{
    Context, CursorIcon, Key, RichText, Sense, TextureHandle, TextureOptions, Vec2, Widget,
};
use puffin::{profile_function, profile_scope};
use rayna_engine::render::render::RenderStats;
use rayna_engine::render::render_opts::{RenderMode, RenderOpts};
use rayna_engine::shared::scene::Scene;
use rayna_shared::def::targets::*;
use rayna_shared::def::types::{Angle, Number, Vector3};
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::time::Duration;
use strum::IntoEnumIterator;
use throttle::Throttle;
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
    render_display_size: Vec2,
    render_stats: RenderStats,

    // The rest
    integration: Integration,
    worker_death_throttle: Throttle,
}

impl RaynaApp {
    /// Creates a new app instance, with an [`Context`] for configuring the app
    pub fn new_ctx(_ctx: &Context) -> Self {
        info!(target: UI, "ui app init");
        let scene = Scene::glass();
        let render_opts = Default::default();
        Self {
            render_opts,
            render_buf_tex: None,
            render_display_size: egui::vec2(1.0, 1.0),
            integration: Integration::new(&render_opts, &scene)
                .expect("couldn't create integration"),
            scene,
            render_stats: Default::default(),
            // Max ten failures in a row, once per second
            worker_death_throttle: Throttle::new(Duration::from_secs(1), 10),
        }
    }
}

impl crate::backend::app::App for RaynaApp {
    fn on_update(&mut self, ctx: &Context) -> () {
        // egui/eframe call `new_frame()` for us if "puffin" feature enabled in them
        if !profiler::EGUI_CALLS_PUFFIN {
            profiler::main_profiler_lock().new_frame();
        }

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
                let w_drag = egui::DragValue::new(&mut w).suffix(UNIT_PX).ui(ui);
                ui.label("Image Height");
                let h_drag = egui::DragValue::new(&mut h).suffix(UNIT_PX).ui(ui);

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

                // MSAA

                let mut msaa = self.render_opts.msaa.get();
                ui.label("MSAA");
                render_opts_dirty |= egui::DragValue::new(&mut msaa).ui(ui).changed();
                self.render_opts.msaa = NonZeroUsize::new(msaa).unwrap_or(NonZeroUsize::MIN);

                ui.label("Mode");
                egui::ComboBox::from_id_source("mode")
                    .selected_text(<&'static str>::from(self.render_opts.mode))
                    .show_ui(ui, |ui| {
                        for variant in RenderMode::iter() {
                            let resp = ui.selectable_value::<RenderMode>(
                                &mut self.render_opts.mode,
                                variant,
                                <&'static str>::from(variant),
                            );
                            render_opts_dirty |= resp.changed();
                        }
                    });

                ui.label("Bounces");
                render_opts_dirty |= egui::DragValue::new(&mut self.render_opts.bounces)
                    .ui(ui)
                    .changed();
            });

            ui.group(|ui| {
                profile_scope!("sec/camera");

                ui.heading("Camera");

                let cam = &mut self.scene.camera;
                ui.label("look from");
                scene_dirty |= ui.vec3_edit(cam.pos.as_array_mut(), UNIT_LEN).changed();
                ui.label("fwd");
                scene_dirty |= ui.vec3_edit(cam.fwd.as_array_mut(), UNIT_LEN).changed();
                ui.label("fov");
                scene_dirty |= ui
                    .add(
                        egui::DragValue::from_get_set(|o| {
                            if let Some(val) = o {
                                cam.v_fov = Angle::from_degrees(val);
                            }
                            cam.v_fov.to_degrees()
                        })
                        .suffix(UNIT_DEG)
                        .clamp_range(0.0..=180.0)
                        .min_decimals(1)
                        .speed(DRAG_SLOW),
                    )
                    .changed();
                ui.label("focus dist");
                scene_dirty |= egui::DragValue::new(&mut cam.focus_dist)
                    .suffix(UNIT_LEN)
                    .speed(DRAG_SLOW)
                    .ui(ui)
                    .changed();
                ui.label("defocus angle");
                scene_dirty |= ui
                    .add(
                        egui::DragValue::from_get_set(|o| {
                            if let Some(val) = o {
                                cam.defocus_angle = Angle::from_degrees(val);
                            }
                            cam.defocus_angle.to_degrees()
                        })
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

            ui.group(|ui| {
                profile_scope!("sec/options");

                ui.heading("Options");

                let mut profiling = puffin::are_scopes_on();
                if ui.checkbox(&mut profiling, "Profiling").changed() {
                    puffin::set_scopes_on(profiling);
                }
            });
            ui.group(|ui| {
                profile_scope!("sec/stats");

                ui.heading("Stats");

                let stats = self.render_stats;
                ui.label(format!("num pixels: {}", stats.num_px));
                ui.label(format!("num threads: {}", stats.num_threads));
                ui.label(format!("duration: {:?}", stats.duration));
            });
        });

        // Central panel contains the main render window
        // Must come after all other panels
        egui::CentralPanel::default().show(ctx, |ui| {
            profile_scope!("panel/central");

            let avail_space = ui.available_size();
            self.render_display_size = avail_space;

            let Some(ref mut tex_handle) = self.render_buf_tex else {
                ui.label(RichText::new("No texture").size(20.0));
                return;
            };

            // Display the image and get drag inputs

            let img_resp = egui::Image::new(SizedTexture {
                id: tex_handle.id(),
                size: avail_space,
            })
            .sense(Sense::click_and_drag())
            .ui(ui);

            let mut cam_changed = false;

            let mut rot_dirs = Vector3::ZERO;
            let mut move_dirs = Vector3::ZERO;
            let mut fov_zoom = 0.;

            ctx.set_cursor_icon(if img_resp.is_pointer_button_down_on() {
                CursorIcon::Grabbing
            } else if img_resp.hovered() {
                CursorIcon::Grab
            } else {
                CursorIcon::Default
            });

            if img_resp.dragged() {
                cam_changed = true;
                let [x, y] = img_resp.drag_delta().into();
                rot_dirs = Vector3::new(x as Number, y as Number, 0.);
                rot_dirs.z += ui.input(|i| i.key_down(Key::Q)) as u8 as Number;
                rot_dirs.z -= ui.input(|i| i.key_down(Key::E)) as u8 as Number;
                rot_dirs *= ui.input(|i| i.stable_dt as Number) * 5.;
            }

            // Now also detect key presses if the mouse button is help
            if img_resp.is_pointer_button_down_on() {
                cam_changed = true;
                move_dirs.x += ui.input(|i| i.key_down(Key::D)) as u8 as Number;
                move_dirs.x -= ui.input(|i| i.key_down(Key::A)) as u8 as Number;
                move_dirs.y += ui.input(|i| i.key_down(Key::Space)) as u8 as Number;
                move_dirs.y -= ui.input(|i| i.modifiers.ctrl) as u8 as Number;
                move_dirs.z += ui.input(|i| i.key_down(Key::W)) as u8 as Number;
                move_dirs.z -= ui.input(|i| i.key_down(Key::S)) as u8 as Number;
                move_dirs *= ui.input(|i| i.stable_dt as Number) * 3.0;
            }

            if img_resp.hovered() {
                fov_zoom -= ui.input(|i| i.scroll_delta.y as Number);
                fov_zoom -= 10. * (ui.input(|i| i.zoom_delta() as Number) - 1.);
                fov_zoom *= 0.05;
                cam_changed |= fov_zoom != 0.;
            }

            let mut speed_mult = 1.;
            if ui.input(|i| i.modifiers.shift) {
                speed_mult *= 5.;
            };
            if ui.input(|i| i.modifiers.alt) {
                speed_mult /= 5.;
            };

            if cam_changed {
                scene_dirty = true;
                self.scene.camera.apply_motion(
                    move_dirs * speed_mult,
                    rot_dirs * speed_mult,
                    fov_zoom * speed_mult,
                );
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
            // warn!(target: UI, ?res);
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
            let opts = TextureOptions::NEAREST;
            match &mut self.render_buf_tex {
                None => {
                    profile_scope!("tex_load");
                    self.render_buf_tex =
                        Some(ctx.load_texture("render_buffer_texture", render.img, opts))
                }
                Some(tex) => {
                    profile_scope!("tex_set");
                    tex.set(render.img, opts)
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
                Err(IntegrationError::WorkerDied(err)) => {
                    if self.worker_death_throttle.accept().is_ok() {
                        warn!(target: UI, err = ? err.deref(), "worker thread died");
                        // Try restarting integration
                        self.integration = Integration::new(&self.render_opts, &self.scene)
                            .expect("failed to re-initialise integration");
                    } else {
                        trace!(target: UI, "worker thread died again... sigh")
                    }
                    // Prevents endless loops if it keeps crashing
                    // Logs will still get spammed though
                    break;
                }

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
