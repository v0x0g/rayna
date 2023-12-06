/// Trait for a function that is called when the UI backend is being initialised
pub trait UiInitFn: FnOnce(&egui::Context) -> () {
    type Output = ();
}
/// Trait for a function that is called each frame.
///
/// This will be where the rendering occurs
pub trait UiUpdateFn: FnMut(&egui::Context) -> () {
    type Output = ();
}
/// Trait for a function that is called when the UI backend is being shut down
pub trait UiShutdownFn: FnOnce() -> () {
    type Output = ();
}

// For some reason we need to manually implement, or we get warnings
// "error[E0277]: the trait bound `{closure@rayna_ui\src\main.rs:11:19: 11:21}: InitFn` is not satisfied"
impl<T: FnOnce(&egui::Context) -> ()> UiInitFn for T {}
impl<T: FnMut(&egui::Context) -> ()> UiUpdateFn for T {}
impl<T: FnOnce() -> ()> UiShutdownFn for T {}

/// A data struct
pub struct UiManager<Init: UiInitFn, Update: UiUpdateFn, Shutdown: UiShutdownFn> {
    pub init_fn: Init,
    pub update_fn: Update,
    pub shutdown_fn: Shutdown,
}
