use std::{marker::PhantomData, ops::DerefMut};

use super::{UiApp, UiBackend};
use crate::targets::*;
use miniquad as mq;
use puffin::profile_function;
use tracing::*;

#[derive(Debug, Copy, Clone)]
pub struct MiniquadBackend<App: UiApp>(PhantomData<App>);

impl<App: UiApp> Default for MiniquadBackend<App> {
    fn default() -> Self { Self(PhantomData::default()) }
}

impl<App: UiApp> UiBackend<App> for MiniquadBackend<App> {
    fn run(self: Box<Self>, _app_name: &str) -> anyhow::Result<()> {
        debug!(target: MAIN, ?_app_name, "running miniquad backend");

        // TODO: Figure out how to use app_name
        mq::start(mq::conf::Conf { ..Default::default() }, move || {
            trace_span!(target: MAIN, "MiniquadBackend::init");

            let mut mq_ctx = trace_span!(target: MAIN, "miniquad::new_rendering_backend()")
                .in_scope(|| mq::window::new_rendering_backend());
            let egui_mq = trace_span!(target: MAIN, "egui_miniquad::new()")
                .in_scope(|| egui_miniquad::EguiMq::new(mq_ctx.deref_mut()));
            let app = trace_span!(target: MAIN, "App::new()").in_scope(|| App::new(egui_mq.egui_ctx()));
            Box::new(MiniquadWrapper { egui_mq, app, mq_ctx }) as Box<dyn mq::EventHandler>
        });

        // mq never errors?
        Ok(())
    }
}

/// Internal struct that acts as mq app, that delegates events onto our actual app
struct MiniquadWrapper<App: UiApp> {
    egui_mq: egui_miniquad::EguiMq,
    mq_ctx: Box<dyn mq::RenderingBackend>,
    app: App,
}

/// Implement the mq::App equivalent for our wrapper, that just delegates to our crate::app object
impl<App: UiApp> mq::EventHandler for MiniquadWrapper<App> {
    // TODO: Quit/shutdown

    fn update(&mut self) {
        // Draw and update are (mostly) called together,
        // so we might as well just do everything in draw
    }

    fn draw(&mut self) {
        profile_function!();

        self.mq_ctx.clear(Some((1., 0., 1., 1.)), None, None); // Magenta error colour
        self.mq_ctx
            .begin_default_pass(mq::PassAction::clear_color(1.0, 1.0, 0.0, 1.0));
        self.mq_ctx.end_render_pass();

        // Render the egui frame (but don't draw yet)
        self.egui_mq.run(self.mq_ctx.deref_mut(), |_mq_ctx, egui_ctx| {
            self.app.on_update(egui_ctx);
        });

        // Draw things behind egui here

        self.egui_mq.draw(self.mq_ctx.deref_mut());

        // Draw things in front of egui here

        self.mq_ctx.commit_frame();
    }

    // ===== PASS-THROUGH EVENTS TO EGUI_MQ =====

    fn mouse_motion_event(&mut self, x: f32, y: f32) { self.egui_mq.mouse_motion_event(x, y); }

    fn mouse_wheel_event(&mut self, dx: f32, dy: f32) { self.egui_mq.mouse_wheel_event(dx, dy); }

    fn mouse_button_down_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.egui_mq.mouse_button_down_event(mb, x, y);
    }

    fn mouse_button_up_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.egui_mq.mouse_button_up_event(mb, x, y);
    }

    fn char_event(&mut self, character: char, _keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.key_down_event(keycode, keymods);
    }

    fn key_up_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}
