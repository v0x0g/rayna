/// Type for a (boxed) closure that returns a (boxed) app instance
pub type AppCtor = Box<dyn FnOnce(&egui::Context) -> Box<dyn App> + 'static>;

/// A trait representing an initialised application that is running
///
/// Obtained by calling
pub trait App: 'static {
    /// Trait for a function that is called each frame.
    ///
    /// This will be where the rendering occurs
    fn on_update(&mut self, ctx: &egui::Context) -> ();
    /// Called when the app is being shut down
    fn on_shutdown(&mut self) -> ();
}
