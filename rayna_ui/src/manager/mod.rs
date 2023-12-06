/// Trait for a function that is called when the UI backend is being initialised
pub trait InitFn: FnOnce() -> () {}
/// Trait for a function that is called each frame.
///
/// This will be where the rendering occurs
pub trait UpdateFn: FnMut(&egui::Context) -> () {}
/// Trait for a function that is called when the UI backend is being shut down
pub trait ShutdownFn: FnOnce() -> () {}

// For some reason we need to manually implement, or we get warnings
// "error[E0277]: the trait bound `{closure@rayna_ui\src\main.rs:11:19: 11:21}: InitFn` is not satisfied"
impl<T: FnOnce() -> ()> InitFn for T {}
impl<T: FnMut(&egui::Context) -> ()> UpdateFn for T {}
impl<T: FnOnce() -> ()> ShutdownFn for T {}

pub struct UiRunner<Init, Update, Shutdown> {
    init_fn: Init,
    update_fn: Update,
    shutdown_fn: Shutdown,
}

impl<Init: InitFn, Update: UpdateFn, Shutdown: ShutdownFn> UiRunner<Init, Update, Shutdown> {
    fn init() {}
    fn update() {}
    fn shutdown() {}
}
