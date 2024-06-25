use crate::ext::UiExt;
use crate::integration::message::MessageToWorker;
use crate::integration::{Integration, IntegrationError};
use crate::profiler;
use crate::targets::*;
use crate::ui_val::{DRAG_SLOW, UNIT_DEG, UNIT_LEN, UNIT_PX};
use eframe::epaint::textures::TextureFilter;
use egui::load::SizedTexture;
use egui::{Context, CursorIcon, Key, RichText, Sense, TextureHandle, TextureOptions, TextureWrapMode, Vec2, Widget};
use puffin::{profile_function, profile_scope};
use rayna_engine::core::types::{Angle, Number, Vector3};
use rayna_engine::render::render::RenderStats;
use rayna_engine::render::render_opts::{RenderMode, RenderOpts};
use rayna_engine::scene::camera::Camera;
use rayna_engine::scene::preset::PresetScene;
use rayna_engine::scene::{self, StandardScene};
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::time::Duration;
use strum::IntoEnumIterator;
use throttle::Throttle;
use tracing::{error, info, trace, warn};

pub struct RaynaApp {
    // Engine things
    render_opts: RenderOpts,
    scene: StandardScene,
    camera: Camera,
    all_presets: Vec<PresetScene>,

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
        let preset = scene::preset::RTTNW_DEMO();
        let render_opts = Default::default();
        Self {
            integration: Integration::new(&render_opts, &preset.scene, &preset.camera)
                .expect("couldn't create integration"),
            // Max ten failures in a row, once per second
            worker_death_throttle: Throttle::new(Duration::from_secs(1), 10),

            scene: preset.scene,
            camera: preset.camera,
            render_opts,
            all_presets: scene::preset::ALL().into(),

            render_buf_tex: None,
            render_display_size: egui::vec2(1.0, 1.0),
            render_stats: Default::default(),
        }
    }
}

impl crate::backend::app::App for RaynaApp {
    fn on_update(&mut self, ctx: &Context) -> () {
        // egui/eframe call `new_frame()` for us if "puffin" feature enabled in them
        if !profiler::EGUI_CALLS_PUFFIN {
            profiler::main::lock().new_frame();
        }

        profile_function!();

        self.process_worker_messages();
        self.process_worker_render(ctx);

        let mut dirty_render_opts = false;
        let mut dirty_scene = false;
        let mut dirty_camera = false;

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

                dirty_render_opts |= w_drag.drag_stopped() || w_drag.lost_focus(); // don't use `.changed()` so it waits till interact complete
                dirty_render_opts |= h_drag.drag_stopped() || h_drag.lost_focus(); // don't use `.changed()` so it waits till interact complete

                self.render_opts.width = NonZeroUsize::new(w).unwrap_or(NonZeroUsize::MIN);
                self.render_opts.height = NonZeroUsize::new(h).unwrap_or(NonZeroUsize::MIN);

                // Button: Fill Image Dimensions

                if ui.button("Fill Canvas").clicked() {
                    dirty_render_opts = true;
                    self.render_opts.width =
                        NonZeroUsize::new(self.render_display_size.x as usize).unwrap_or(NonZeroUsize::MIN);
                    self.render_opts.height =
                        NonZeroUsize::new(self.render_display_size.y as usize).unwrap_or(NonZeroUsize::MIN);
                }

                // MSAA

                ui.label("MSAA");
                let mut msaa = self.render_opts.samples.get();
                dirty_render_opts |= egui::DragValue::new(&mut msaa).ui(ui).changed();
                self.render_opts.samples = NonZeroUsize::new(msaa).unwrap_or(NonZeroUsize::MIN);

                // RAY BOUNCE DEPTH

                ui.label("Ray Depth");
                dirty_render_opts |= egui::DragValue::new(&mut self.render_opts.ray_depth).ui(ui).changed();

                // RAY BRANCHING

                ui.label("Ray Branching");
                let mut ray_branching = self.render_opts.ray_branching.get();
                dirty_render_opts |= egui::DragValue::new(&mut ray_branching).ui(ui).changed();
                self.render_opts.ray_branching = NonZeroUsize::new(ray_branching).unwrap_or(NonZeroUsize::MIN);

                // RENDER MODE

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
                            dirty_render_opts |= resp.changed();
                        }
                    });
            });

            ui.group(|ui| {
                profile_scope!("sec/camera");

                ui.heading("Camera");

                let cam = &mut self.camera;
                ui.label("look from");
                dirty_camera |= ui.vec3_edit(cam.pos.as_array_mut(), UNIT_LEN).changed();
                ui.label("fwd");
                dirty_camera |= ui.vec3_edit(cam.fwd.as_array_mut(), UNIT_LEN).changed();
                ui.label("fov");
                dirty_camera |= ui
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
                dirty_camera |= egui::DragValue::new(&mut cam.focus_dist)
                    .suffix(UNIT_LEN)
                    .speed(DRAG_SLOW)
                    .ui(ui)
                    .changed();
                ui.label("defocus angle");
                dirty_camera |= ui
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

                let mut preset_index = None;

                egui::ComboBox::from_label("Scene Presets")
                    .selected_text("<Select a Scene>")
                    .show_ui(ui, |ui| {
                        for (i, preset) in self.all_presets.iter().enumerate() {
                            ui.selectable_value(&mut preset_index, Some(i), preset.name);
                        }
                    });

                if let Some(idx) = preset_index {
                    self.scene = self.all_presets[idx].scene.clone();
                    self.camera = self.all_presets[idx].camera.clone();

                    dirty_scene = true;
                    dirty_camera = true;
                }
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
                ui.label(format!("width:\t\t\t {}", stats.opts.width.get()));
                ui.label(format!("height:\t\t\t {}", stats.opts.height.get()));
                ui.label(format!("samples:\t\t {}", stats.opts.samples));
                ui.label(format!("depth:\t\t\t {}", stats.opts.ray_depth));
                ui.label(format!("branching:\t\t\t {}", stats.opts.ray_branching));
                ui.label(format!("mode:\t\t\t {}", stats.opts.mode));
                ui.label(format!("num threads: {}", stats.num_threads));
                ui.label(format!("duration:\t\t {}", humantime::format_duration(stats.duration)));
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

            ctx.set_cursor_icon(if img_resp.is_pointer_button_down_on() {
                CursorIcon::Grabbing
            } else if img_resp.hovered() {
                CursorIcon::Grab
            } else {
                CursorIcon::Default
            });

            // Speed multiplier to change how fast we move/rotate/zoom
            let mut speed_mult = 1.;
            if ui.input(|i| i.modifiers.shift) {
                speed_mult *= 5.;
            };
            if ui.input(|i| i.modifiers.alt) {
                speed_mult /= 5.;
            };

            // Rotate when dragged
            if img_resp.dragged() {
                // X: Yaw, Y: Pitch, Z: Roll
                let mut rot = Vector3::ZERO;
                rot.x = -img_resp.drag_delta().x as Number;
                rot.y = -img_resp.drag_delta().y as Number;
                rot.z += ui.input(|i| i.key_down(Key::Q)) as u8 as Number;
                rot.z -= ui.input(|i| i.key_down(Key::E)) as u8 as Number;

                rot *= speed_mult * ui.input(|i| i.stable_dt as Number) * 25.;

                let [yaw, pitch, roll] = rot.to_array().map(Angle::from_degrees);

                let _ = self.camera.apply_rot_delta(yaw, pitch, roll);
                dirty_camera = true;
            }

            // Also detect key presses (movement) if the mouse button is held
            if img_resp.is_pointer_button_down_on() {
                let mut pos = Vector3::ZERO;
                pos.x += ui.input(|i| i.key_down(Key::D)) as u8 as Number;
                pos.x -= ui.input(|i| i.key_down(Key::A)) as u8 as Number;
                pos.y += ui.input(|i| i.key_down(Key::Space)) as u8 as Number;
                pos.y -= ui.input(|i| i.key_down(Key::C)) as u8 as Number;
                pos.z += ui.input(|i| i.key_down(Key::W)) as u8 as Number;
                pos.z -= ui.input(|i| i.key_down(Key::S)) as u8 as Number;

                pos *= speed_mult * ui.input(|i| i.stable_dt as Number) * 5.;

                let [right_left, up_down, fwd_back] = pos.to_array();

                let _ = self.camera.apply_pos_delta(fwd_back, right_left, up_down);
                dirty_camera = true;
            }

            // Change FOV when mouse hovered
            if img_resp.hovered() {
                let mut fov_zoom = 0.;
                fov_zoom -= ui.input(|i| i.raw_scroll_delta.y as Number);
                fov_zoom -= 10. * (ui.input(|i| i.zoom_delta() as Number) - 1.);
                fov_zoom *= speed_mult * ui.input(|i| i.stable_dt as Number) * 20.;
                if fov_zoom != 0. {
                    self.camera.v_fov += Angle::from_degrees(fov_zoom);
                    dirty_camera = true;
                }
            }
        });

        if dirty_render_opts {
            profile_scope!("update_render_opts");
            info!(target: UI, render_opts = ?self.render_opts, "render opts dirty, sending to worker");

            if let Err(err) = self
                .integration
                .send_message(MessageToWorker::SetRenderOpts(self.render_opts))
            {
                warn!(target: UI, ?err)
            }
        }

        if dirty_scene {
            profile_scope!("update_scene");
            trace!(target: UI, /*scene = ?self.scene, */ "scene dirty, sending to worker");

            if let Err(err) = self
                .integration
                .send_message(MessageToWorker::SetScene(self.scene.clone()))
            {
                warn!(target: UI, ?err)
            }
        }

        if dirty_camera {
            profile_scope!("update_camera");
            trace!(target: UI, /*scene = ?self.scene, */ "camera dirty, sending to worker");

            if let Err(err) = self
                .integration
                .send_message(MessageToWorker::SetCamera(self.camera.clone()))
            {
                warn!(target: UI, ?err)
            }
        }

        // Continuously update UI
        ctx.request_repaint();
    }

    fn on_shutdown(&mut self) -> () { info!(target: UI, "ui app shutdown") }
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
            let opts = TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Linear,
                wrap_mode: TextureWrapMode::ClampToEdge,
            };
            match &mut self.render_buf_tex {
                None => {
                    profile_scope!("tex_load");
                    self.render_buf_tex = Some(ctx.load_texture("render_buffer_texture", render.img, opts))
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
                        self.integration = Integration::new(&self.render_opts, &self.scene, &self.camera)
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
