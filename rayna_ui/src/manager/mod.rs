/// Trait for a function that is called when the UI backend is being initialised
pub trait UiInitFn = FnOnce(&egui::Context) -> ();
/// Trait for a function that is called each frame.
///
/// This will be where the rendering occurs
pub trait UiUpdateFn = FnMut(&egui::Context) -> ();
/// Trait for a function that is called when the UI backend is being shut down
pub trait UiShutdownFn = FnOnce() -> ();

/// A data struct
pub struct UiManager<Init: UiInitFn, Update: UiUpdateFn, Shutdown: UiShutdownFn> {
    pub init_fn: Init,
    pub update_fn: Update,
    pub shutdown_fn: Shutdown,
}
