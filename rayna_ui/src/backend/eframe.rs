use crate::backend::{UiBackend, UiInitFn, UiShutdownFn, UiUpdateFn};
use crate::func::UiFunctions;
use eframe::Theme;
use std::error::Error;

pub struct EFrameBackend<Init: UiInitFn, Update: UiUpdateFn, Shutdown: UiShutdownFn> {
    manager: UiFunctions<Init, Update, Shutdown>,
}

impl<Init: UiInitFn, Update: UiUpdateFn, Shutdown: UiShutdownFn> UiBackend<Init, Update, Shutdown>
    for EFrameBackend<Init, Update, Shutdown>
{
    fn new(manager: UiFunctions<Init, Update, Shutdown>) -> Self {
        Self { manager }
    }

    fn run(self) -> Result<(), Box<dyn Error>> {
        let UiFunctions {
            init_fn,
            update_fn,
            shutdown_fn,
        } = self.manager;

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
            EframeWrapper::make_app_creator(init_fn, update_fn, shutdown_fn),
        )?;

        Ok(())
    }
}

/// Internal struct that acts as an app instance for [eframe]
struct EframeWrapper {
    update_fn: Box<dyn UiUpdateFn>,
    /// [Option] so that we can tell if shutdown has already been called
    shutdown_fn: Option<Box<dyn UiShutdownFn>>,
}

impl eframe::App for EframeWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        (self.update_fn)(ctx);
    }

    fn on_exit(&mut self, _glow: Option<&eframe::glow::Context>) {
        let shutdown = self
            .shutdown_fn
            .take()
            .expect("on_exit should not have been called before");
        (shutdown)();
    }
}

impl EframeWrapper {
    /// Creates an [eframe::AppCreator] (which wraps around an [EframeWrapper])
    /// for use with [eframe]
    ///
    /// # Parameters
    /// See [UiFunctions]
    pub fn make_app_creator(
        init_fn: impl UiInitFn,
        update_fn: impl UiUpdateFn,
        shutdown_fn: impl UiShutdownFn,
    ) -> eframe::AppCreator {
        // This closure is called by `eframe` to initialise the app
        // It moves all the functions into itself so that they can be called at the appropriate times
        let closure = move |ctx: &eframe::CreationContext| {
            (init_fn)(&ctx.egui_ctx);
            let app = EframeWrapper {
                update_fn: Box::new(update_fn),
                shutdown_fn: Some(Box::new(shutdown_fn)),
            };
            Box::new(app) as Box<dyn eframe::App>
        };

        Box::new(closure)
    }
}
