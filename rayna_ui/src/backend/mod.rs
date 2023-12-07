use crate::{func::UiFunctions, func::UiInitFn, func::UiShutdownFn, func::UiUpdateFn};
use std::error::Error;

#[cfg(feature = "backend_eframe")]
pub mod eframe;

/// A trait that represents a type that can be used as a backend for the UI
pub trait UiBackend<Init: UiInitFn, Update: UiUpdateFn, Shutdown: UiShutdownFn> {
    /// Creates a new [`UiBackend`] which uses the functions in the given [`UiFunctions`]
    /// for init, update, etc
    fn new(ui_manager: UiFunctions<Init, Update, Shutdown>) -> Self;

    /// Runs the UI, consuming the backend in the process
    ///
    /// [Self] is consumed so that the [FnOnce] bounds on [Init] and [Shutdown] are satisfied
    fn run(self) -> Result<(), Box<dyn Error>>;
}
