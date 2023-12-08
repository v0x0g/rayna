#[cfg(feature = "backend_eframe")]
pub mod eframe;

#[cfg(feature = "backend_miniquad")]
pub mod miniquad;

/// A trait that represents a type that can be used as a backend for the UI
pub trait UiBackend: Sized {
    /// Runs the UI, consuming the app in the process
    ///
    /// Takes in a function (possibly a closure) that will initialise (and return) the application
    /// instance.
    fn run<A: crate::app::App>(
        self,
        app_name: &str,
        app_ctor: impl FnOnce(&egui::Context) -> A + 'static,
    ) -> anyhow::Result<()>;
}
