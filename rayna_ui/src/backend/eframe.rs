use crate::backend::{UiBackend, UiInitFn, UiShutdownFn, UiUpdateFn};
use crate::manager::UiManager;
use std::error::Error;

pub struct EFrameBackend<Init: UiInitFn, Update: UiUpdateFn, Shutdown: UiShutdownFn> {
    manager: UiManager<Init, Update, Shutdown>,
}

impl<Init: UiInitFn, Update: UiUpdateFn, Shutdown: UiShutdownFn> UiBackend<Init, Update, Shutdown>
    for EFrameBackend<Init, Update, Shutdown>
{
    fn new(manager: UiManager<Init, Update, Shutdown>) -> Self {
        Self { manager }
    }

    fn run(self) -> Result<(), Box<dyn Error>> {
        let UiManager {
            init_fn,
            update_fn,
            shutdown_fn,
        } = self.manager;

        /// Internal struct that acts as an app instance for [eframe]
        struct EFrameApp {
            update_fn: Box<dyn UiUpdateFn>,
            /// [Option] so that we can tell if shutdown has already been called
            shutdown_fn: Option<Box<dyn UiShutdownFn>>,
        }

        impl eframe::App for EFrameApp {
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

        // This closure is called by `eframe` to initialise the app
        // It moves all the functions into itself
        let factory_closure = move |ctx: &eframe::CreationContext| {
            (init_fn)(&ctx.egui_ctx);
            let app = EFrameApp {
                update_fn: Box::new(update_fn),
                shutdown_fn: Some(Box::new(shutdown_fn)),
            };
            Box::new(app) as Box<dyn eframe::App>
        };

        let app_creator: eframe::AppCreator = Box::new(factory_closure);

        eframe::run_native(
            crate::definitions::constants::APP_NAME,
            eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([400.0, 300.0])
                    .with_min_inner_size([300.0, 220.0]),
                ..Default::default()
            },
            app_creator,
        )?;

        unreachable!();
    }
}
