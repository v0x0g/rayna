use crate::app::{App, UninitApp};
use crate::backend::UiBackend;
use egui_miniquad::EguiMq;
use miniquad::conf::Conf;
use miniquad::EventHandler;

pub struct MiniquadBackend;

impl<Init: App, Uninit: UninitApp<InitApp = Init>> UiBackend<Init, Uninit> for MiniquadBackend {
    fn run(self, uninit_app: Uninit) -> anyhow::Result<()> {
        miniquad::start(
            Conf {
                ..Default::default()
            },
            |ctx| Box::new(MiniquadWrapper::new(ctx)) as Box<dyn EventHandler>,
        );

        // miniquad never errors?
        Ok(())
    }
}

struct MiniquadWrapper {
    egui_mq: EguiMq,
}

impl MiniquadWrapper {
    fn new(ctx: &mut miniquad::Context) -> Self {
        Self {
            egui_mq: EguiMq::new(ctx),
        }
    }
}

impl miniquad::EventHandler for MiniquadWrapper {
    fn update(&mut self, _: &mut miniquad::Context) {
        // Draw and update are (mostly) called together,
        // so we might as well just do everything in draw
    }

    fn draw(&mut self, mq_ctx: &mut miniquad::Context) {
        mq_ctx.clear(Some((1., 1., 1., 1.)), None, None);
        mq_ctx.begin_default_pass(miniquad::PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        mq_ctx.end_render_pass();

        // Render the egui frame (but don't draw yet)
        self.egui_mq.run(mq_ctx, |_mq_ctx, egui_ctx| {
            egui::Window::new("Egui Window").show(egui_ctx, |ui| {
                ui.heading("Hello World!");
            });
        });

        // Draw things behind egui here

        self.egui_mq.draw(mq_ctx);

        // Draw things in front of egui here

        mq_ctx.commit_frame();
    }

    // ===== PASS-THROUGH EVENTS TO EGUI_MQ =====

    fn mouse_motion_event(&mut self, _: &mut miniquad::Context, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, _: &mut miniquad::Context, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut miniquad::Context,
        mb: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut miniquad::Context,
        mb: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        character: char,
        _keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
    ) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}
