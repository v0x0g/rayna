use crate::app::{App, UninitApp};

#[cfg(feature = "backend_eframe")]
pub mod eframe;
mod miniquad;

/// A trait that represents a type that can be used as a backend for the UI
pub trait UiBackend<Init: App, Uninit: UninitApp<InitApp = Init>> {
    /// Runs the UI, consuming the app in the process
    fn run(self, app: Uninit) -> anyhow::Result<()>;
}
