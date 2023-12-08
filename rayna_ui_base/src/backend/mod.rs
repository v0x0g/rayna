use crate::app::{App, UninitApp};
use egui::Context;

#[cfg(feature = "backend_eframe")]
pub mod eframe;

#[cfg(feature = "backend_miniquad")]
pub mod miniquad;

/// A trait that represents a type that can be used as a backend for the UI
pub trait UiBackend: Sized {
    /// Runs the UI, consuming the app in the process
    ///
    /// Takes in an uninitialised app, and initialises it before running
    fn run_init<Uninit: UninitApp>(self, app_name: &str, app: Uninit) -> anyhow::Result<()>;

    /// Runs the UI, consuming the app in the process
    ///
    /// Differs from [run_uninit] in that it there is no initialisation
    fn run_no_init<Init: App>(self, app_name: &str, app: Init) -> anyhow::Result<()> {
        struct Wrapper<T: App>(T);
        impl<T: App> UninitApp for Wrapper<T> {
            type InitApp = T;

            fn init(self, _ctx: &Context) -> Self::InitApp {
                self.0
            }
        }

        self.run_init(app_name, Wrapper(app))
    }
}
