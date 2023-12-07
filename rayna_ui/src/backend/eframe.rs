use crate::app::{App, UninitApp};
use crate::backend::UiBackend;
use eframe::Theme;
use std::error::Error;

pub struct EFrameBackend;

impl<Init: App, Uninit: UninitApp<InitApp = Init>> UiBackend<Init, Uninit> for EFrameBackend {
    fn run(uninit_app: Uninit) -> Result<(), Box<dyn Error>> {
        eframe::run_native(
            crate::definitions::constants::APP_NAME,
            eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([400.0, 300.0])
                    .with_min_inner_size([300.0, 220.0]),
                run_and_return: true,
                default_theme: Theme::Dark,

                ..Default::default()
            },
            make_app_creator(uninit_app),
        )?;

        Ok(())
    }
}

/// Internal struct that acts as an app instance for [eframe]
///
/// # Notes
/// Use an [Option] for the app, so that we can safely consume [T] on shutdown,
/// as per [App::on_shutdown] contract
struct EframeWrapper<T> {
    app: Option<T>,
}

impl<A: App> eframe::App for EframeWrapper<A> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.app
            .as_mut()
            .expect("on_update called after app has been consumed (shutdown)")
            .on_update(ctx);
    }

    fn on_exit(&mut self, _glow: Option<&eframe::glow::Context>) {
        self.app
            .take()
            .expect("on_exit called after app has been consumed (shutdown)")
            .on_shutdown();
    }
}

/// Creates an [eframe::AppCreator] (which wraps around an [EframeWrapper])
/// for use with [eframe]
///
/// # Parameters
/// See [UiFunctions]
pub fn make_app_creator<U: UninitApp>(uninit_app: U) -> eframe::AppCreator {
    // This closure is called by `eframe` to initialise the app
    // It moves all the functions into itself so that they can be called at the appropriate times
    let closure = move |ctx: &eframe::CreationContext| {
        // Initialise the app using the egui context
        let app = uninit_app.init(&ctx.egui_ctx);
        let wrapper = EframeWrapper { app: Some(app) };
        Box::new(wrapper) as Box<dyn eframe::App>
    };

    Box::new(closure)
}
