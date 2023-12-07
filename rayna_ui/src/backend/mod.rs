use crate::app::{App, UninitApp};
use std::error::Error;

#[cfg(feature = "backend_eframe")]
pub mod eframe;

/// A trait that represents a type that can be used as a backend for the UI
pub trait UiBackend<Init: App, Uninit: UninitApp<InitApp = Init>> {
    /// Runs the UI, consuming the app in the process
    fn run(app: Uninit) -> Result<(), Box<dyn Error>>;
}
