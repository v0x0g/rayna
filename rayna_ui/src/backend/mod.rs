use std::collections::HashMap;

use tracing::debug;

#[cfg(feature = "backend_eframe")]
pub mod eframe;
#[cfg(feature = "backend_miniquad")]
pub mod miniquad;

/// A trait that represents a type that can be used as a backend for the UI
pub trait UiBackend<App: UiApp> {
    // NOTE: In order to be object-safe, this trait needs to have the `App` type parameter
    // instead of the `self.run()` method

    /// Runs the UI
    /// # Note
    /// The backend is boxed for object-safe-ness reasons (dynamic dispatch).
    /// The app should be created by calling [`UiApp::new()`] on the `App` parameter
    fn run(self: Box<Self>, app_name: &str) -> anyhow::Result<()>;
}

/// A trait representing an application that is running
pub trait UiApp: 'static {
    fn new(context: &egui::Context) -> Self;
    /// Trait for a function that is called each frame.
    ///
    /// This will be where the rendering occurs
    fn on_update(&mut self, ctx: &egui::Context) -> ();
    /// Called when the app is being shut down
    fn on_shutdown(&mut self) -> ();
}

/// Gets a map of all the [`UiBackend`] implementations available
pub fn get_all<App: UiApp + 'static>() -> HashMap<&'static str, Box<dyn UiBackend<App>>> {
    let mut backends: HashMap<&'static str, Box<dyn UiBackend<App>>> = HashMap::new();

    #[cfg(feature = "backend_eframe")]
    {
        debug!(target: crate::targets::MAIN, "have backend: eframe");
        backends.insert("eframe", Box::new(self::eframe::EframeBackend::default()));
    }
    #[cfg(feature = "backend_miniquad")]
    {
        debug!(target: crate::targets::MAIN, "have backend: miniquad");
        backends.insert("miniquad", Box::new(self::miniquad::MiniquadBackend::default()));
    }

    backends
}
