/// Trait for a function that is called when the UI backend is being initialised
pub trait UiInitFn = FnOnce(&egui::Context) -> () + 'static;
/// Trait for a function that is called each frame.
///s
/// This will be where the rendering occurs
pub trait UiUpdateFn = FnMut(&egui::Context) -> () + 'static;
/// Trait for a function that is called when the UI backend is being shut down
pub trait UiShutdownFn = FnOnce() -> () + 'static;

/// A data struct containing functions for the Ui
pub struct UiFunctions<Init: UiInitFn, Update: UiUpdateFn, Shutdown: UiShutdownFn> {
    /// The function to be called during initialisation.
    /// Can take in an [egui::Context] to setup things before the app starts
    pub init_fn: Init,
    /// Called every frame, should do the rendering
    pub update_fn: Update,
    pub shutdown_fn: Shutdown,
}
