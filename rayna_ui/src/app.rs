use crate::ext::ui_ext::UiExt as _;
use crate::integration::message::MessageToWorker;
use crate::integration::{Integration, IntegrationError};
use crate::targets::*;
use crate::ui_val::*;
use eframe::epaint::textures::TextureFilter;
use egui::load::SizedTexture;
use egui::{
    Checkbox, ColorImage, Context, CursorIcon, Grid, Key, Sense, TextureHandle, TextureId, TextureOptions,
    TextureWrapMode, Ui, Widget,
};
use puffin::{profile_function, profile_scope};
use rayna_engine::core::types::*;
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
    render_buf_tex: TextureHandle,
    /// Options for how the render buffer texture is displayed
    render_buf_tex_options: TextureOptions,
    /// The amount of space available to display the rendered image in
    /// This is [`egui::Ui::available_size`] inside [egui::CentralPanel]
    /// Used by the "fit canvas to screen" button
    render_display_size: egui::Vec2,
    render_stats: RenderStats,

    // Integration with the engine and worker
    integration: Integration,
    worker_death_throttle: Throttle,
}

impl crate::backend::UiApp for RaynaApp {
    /// Creates a new app instance, with an [`Context`] for configuring the app
    fn new(ctx: &Context) -> Self {
        info!(target: MAIN, "ui app init");

        trace!(target: MAIN, "loading preset scene and render opts");
        let PresetScene { scene, camera, name: _ } = scene::preset::RTTNW_DEMO();
        let render_opts = Default::default();
        let all_presets = scene::preset::ALL().into();

        trace!(target: MAIN, "creating render buffer texture");
        let render_buf_tex_options = TextureOptions {
            magnification: TextureFilter::Nearest,
            minification: TextureFilter::Linear,
            wrap_mode: TextureWrapMode::ClampToEdge,
        };
        let render_buf_tex = ctx.load_texture(
            // Default is tiny invisible texture, so it's unobtrusive
            // it will be visible up unlti first frame is received from renderer
            "RaynaApp::render_buffer_texture",
            ColorImage::new([128, 128], egui::Color32::TRANSPARENT),
            render_buf_tex_options,
        );

        trace!(target: MAIN, "creating engine integration");
        let integration = Integration::new(&render_opts, &scene, &camera).expect("failed to create integration");
        // Max ten failures in a row, once per second
        let worker_death_throttle = Throttle::new(Duration::from_secs(1), 10);

        Self {
            integration,
            worker_death_throttle,

            scene,
            camera,
            render_opts,
            all_presets,

            render_buf_tex_options,
            render_buf_tex,
            render_display_size: egui::vec2(1.0, 1.0),
            render_stats: Default::default(),
        }
    }

    fn on_shutdown(&mut self) -> () { info!(target: MAIN, "ui app shutdown") }

    fn on_update(&mut self, ctx: &Context) -> () {
        // `egui`/`eframe` call `new_frame()` for us if "puffin" feature enabled in them
        if !crate::profiler::EGUI_CALLS_PUFFIN {
            crate::profiler::main::lock().new_frame();
        }

        // TODO: Add tooltips to the UI
        //  We can use the `egui_commonmark` crate, with compile-time evaluation
        // of markdown docs (uses the `macros` feature).

        profile_function!();

        self.process_worker_messages();
        self.process_worker_render();

        let mut dirty_render_opts = false;
        let mut dirty_scene = false;
        let mut dirty_camera = false;

        {
            profile_scope!("panel/left");
            egui::SidePanel::left("left_panel").show(ctx, |ui| {
                Self::show_app_options(ui);

                Self::show_scene_options(
                    ui,
                    &self.all_presets,
                    &mut self.scene,
                    &mut self.camera,
                    &mut dirty_scene,
                    &mut dirty_camera,
                );
                Self::show_render_options(
                    ui,
                    &mut self.render_opts,
                    &mut dirty_render_opts,
                    self.render_display_size,
                );
                Self::show_camera_options(ui, &mut self.camera, &mut dirty_camera);
                Self::show_render_stats(ui, self.render_stats);
                // TODO: Add a button to save the image to disk
            });
        }

        // Central panel contains the main render window, must come after all other panels
        {
            profile_scope!("panel/central");
            egui::CentralPanel::default().show(ctx, |ui| {
                self.render_display_size =
                    Self::show_render_buf(ctx, ui, self.render_buf_tex.id(), &mut self.camera, &mut dirty_camera);
            });
        }

        self.apply_dirtiness(dirty_render_opts, dirty_scene, dirty_camera);

        // Continuously update UI
        ctx.request_repaint();
    }
}

/// Implementation for the UI code
impl RaynaApp {
    fn show_app_options(ui: &mut Ui) {
        profile_function!();

        ui.group(|ui| {
            ui.heading("Options");

            Grid::new("grid_app_options").show(ui, |ui| {
                ui.label("Profiling");
                let mut profiling = puffin::are_scopes_on();
                if Checkbox::without_text(&mut profiling).ui(ui).changed() {
                    puffin::set_scopes_on(profiling);
                }
                ui.end_row();
            });

            ui.fill_available_width();
        });
    }

    fn show_render_options(
        ui: &mut Ui,
        render_opts: &mut RenderOpts,
        dirty_render_opts: &mut bool,
        render_display_size: egui::Vec2,
    ) {
        profile_function!();

        ui.group(|ui| {
            ui.heading("Render Options");

            Grid::new("grid_render_options").show(ui, |ui| {
                ui.label("Image Width");
                ui.columns(3, |cols| {
                    let w_drag = cols[0].edit_nonzero_usize(&mut render_opts.width, UNIT_PX, DRAG_SPEED_PX);
                    let h_drag = cols[1].edit_nonzero_usize(&mut render_opts.height, UNIT_PX, DRAG_SPEED_PX);
                    let drag = h_drag | w_drag;
                    // don't use `.changed()` so it waits till interact complete
                    *dirty_render_opts = drag.drag_stopped() || (drag.lost_focus() && drag.changed());

                    if cols[2].button("Fill").clicked() {
                        *dirty_render_opts = true;
                        render_opts.width =
                            NonZeroUsize::new(render_display_size.x as usize).unwrap_or(NonZeroUsize::MIN);
                        render_opts.height =
                            NonZeroUsize::new(render_display_size.y as usize).unwrap_or(NonZeroUsize::MIN);
                    }
                });
                ui.end_row();
                ui.end_row();
                ui.label("MSAA");
                *dirty_render_opts |= ui
                    .edit_nonzero_usize(&mut render_opts.samples, "", DRAG_SPEED_NUM_SMALL)
                    .changed();
                ui.end_row();
                ui.label("Ray Depth");
                *dirty_render_opts |= ui
                    .edit_usize(&mut render_opts.ray_depth, "", DRAG_SPEED_NUM_SMALL)
                    .changed();
                ui.end_row();
                ui.label("Ray Branching");
                *dirty_render_opts |= ui
                    .edit_nonzero_usize(&mut render_opts.ray_branching, "", DRAG_SPEED_NUM_SMALL)
                    .changed();
                ui.end_row();
                ui.label("Render Mode");
                egui::ComboBox::from_id_source("mode")
                    .selected_text(<&'static str>::from(render_opts.mode))
                    .show_ui(ui, |ui| {
                        for variant in RenderMode::iter() {
                            let resp = ui.selectable_value::<RenderMode>(
                                &mut render_opts.mode,
                                variant,
                                <&'static str>::from(variant),
                            );
                            *dirty_render_opts |= resp.changed();
                        }
                    });
                ui.end_row();
            });

            ui.fill_available_width();
        });
    }

    fn show_scene_options(
        ui: &mut Ui,
        all_presets: &[PresetScene],
        scene: &mut StandardScene,
        camera: &mut Camera,
        dirty_scene: &mut bool,
        dirty_camera: &mut bool,
    ) {
        profile_function!();

        ui.group(|ui| {
            ui.heading("Scene");

            egui::Grid::new("grid_scene").show(ui, |ui| {
                let mut preset_index = None;

                ui.label("Scene Presets");
                egui::ComboBox::from_id_source("dropdown_scene_preset")
                    .selected_text("<Select a Scene>")
                    .show_ui(ui, |ui| {
                        for (i, preset) in all_presets.iter().enumerate() {
                            ui.selectable_value(&mut preset_index, Some(i), preset.name);
                        }
                    });

                if let Some(idx) = preset_index {
                    *scene = all_presets[idx].scene.clone();
                    *camera = all_presets[idx].camera.clone();

                    *dirty_scene = true;
                    *dirty_camera = true;
                }
            });

            ui.fill_available_width();
        });
    }

    fn show_camera_options(ui: &mut Ui, camera: &mut Camera, dirty_camera: &mut bool) {
        profile_function!();

        ui.group(|ui| {
            ui.heading("Camera");

            egui::Grid::new("grid_camera").show(ui, |ui| {
                ui.label("Position");
                *dirty_camera |= ui
                    .edit_vec3(camera.pos.as_array_mut(), UNIT_LEN, DRAG_SPEED_LEN)
                    .changed();
                ui.end_row();
                ui.label("Forward");
                *dirty_camera |= ui
                    .edit_vec3(camera.fwd.as_array_mut(), UNIT_LEN, DRAG_SPEED_LEN)
                    .changed();
                ui.end_row();
                ui.label("FOV");
                *dirty_camera |= ui.edit_angle(&mut camera.v_fov, (0.0, 180.0)).changed();
                ui.end_row();
                ui.label("Focus Distance");
                *dirty_camera |= ui
                    .edit_number(&mut camera.focus_dist, UNIT_LEN, DRAG_SPEED_LEN)
                    .changed();
                ui.end_row();
                ui.label("Defocus Angle");
                *dirty_camera |= ui.edit_angle(&mut camera.defocus_angle, (0.0, 90.0)).changed();
                ui.end_row();
            });

            ui.fill_available_width();
        });
    }

    /// Displays the render stats
    fn show_render_stats(ui: &mut Ui, stats: RenderStats) {
        profile_function!();

        ui.group(|ui| {
            ui.heading("Stats");

            egui::Grid::new("grid_stats").show(ui, |ui| {
                ui.label("Width");
                ui.label(&stats.opts.width.get().to_string());
                ui.end_row();
                ui.label("Height");
                ui.label(&stats.opts.height.get().to_string());
                ui.end_row();
                ui.label("Samples");
                ui.label(&stats.opts.samples.to_string());
                ui.end_row();
                ui.label("Depth");
                ui.label(&stats.opts.ray_depth.to_string());
                ui.end_row();
                ui.label("Branching");
                ui.label(&stats.opts.ray_branching.to_string());
                ui.end_row();
                ui.label("Mode");
                ui.label(&stats.opts.mode.to_string());
                ui.end_row();
                ui.label("Num Threads");
                ui.label(&stats.num_threads.to_string());
                ui.end_row();
                ui.label("Accumulated");
                ui.label(&stats.accum_frames.to_string());
                ui.end_row();
                ui.label("Duration");
                ui.label(&humantime::format_duration(stats.duration).to_string());
                ui.end_row();
            });

            ui.fill_available_width();
        });
    }

    /// Displays the render buffer in the UI, processing any use input for the camera
    ///
    /// # Arguments
    /// - `ui`, `ctx`: Required for interacting with `egui`
    /// - `render_texture_id`: [TextureId] for the render buffer, to be displayed
    /// - `camera`: Reference to a camera, which will be modified according to the user input
    /// - `dirty_camera`: Flag set to `true` if the `camera` was modified
    fn show_render_buf(
        ctx: &Context,
        ui: &mut Ui,
        render_texture_id: TextureId,
        camera: &mut Camera,
        dirty_camera: &mut bool,
    ) -> egui::Vec2 {
        profile_function!();

        // Fill entire available space
        let avail_space = ui.available_size();
        let img_resp = egui::Image::new(SizedTexture::new(render_texture_id, avail_space))
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

            if rot != Vector3::ZERO {
                let [yaw, pitch, roll] = rot.to_array().map(Angle::from_degrees);

                camera.apply_rot_delta(yaw, pitch, roll);
                *dirty_camera = true;
            }
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

            if pos != Vector3::ZERO {
                let [right_left, up_down, fwd_back] = pos.to_array();

                camera.apply_pos_delta(fwd_back, right_left, up_down);
                *dirty_camera = true;
            }
        }

        // Change FOV when mouse hovered
        if img_resp.hovered() {
            let mut fov_zoom = 0.;
            fov_zoom -= ui.input(|i| i.raw_scroll_delta.y as Number);
            fov_zoom -= 10. * (ui.input(|i| i.zoom_delta() as Number) - 1.);
            fov_zoom *= speed_mult * ui.input(|i| i.stable_dt as Number) * 20.;
            if fov_zoom != 0. {
                camera.v_fov += Angle::from_degrees(fov_zoom);
                *dirty_camera = true;
            }
        }

        return avail_space;
    }
}

/// Integration-related functions
impl RaynaApp {
    /// Tries to receive the next render frame from the worker, updating the render buffer's texture if there was one available.
    fn process_worker_render(&mut self) {
        profile_function!();

        let Some(res) = self.integration.try_recv_render() else {
            return; // No frame yet
        };
        let Ok(render) = res else {
            // warn!(target: UI, ?res);
            return;
        };

        trace!(target: UI, "received new frame from worker");

        {
            profile_scope!("update_tex");
            self.render_buf_tex.set(render.img, self.render_buf_tex_options)
        }

        self.render_stats = render.stats;
    }

    /// Processes the messages from the worker
    ///
    /// Currently does nothing, just here for future compatability
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

    /// Sends updates to the worker, if anything has changed (such as the scene or camera)
    fn apply_dirtiness(&mut self, dirty_render_opts: bool, dirty_scene: bool, dirty_camera: bool) {
        profile_function!();

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
    }
}
