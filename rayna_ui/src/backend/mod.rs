#[cfg(feature = "backend_eframe")]
pub mod eframe;

pub trait InitFn: FnOnce() -> () {}
pub trait UpdateFn: FnMut(&egui::Context) -> () + 'static {}
pub trait ShutdownFn: FnOnce() -> () {}

impl<T: FnOnce() -> ()> InitFn for T {}
impl<T: FnMut(&egui::Context) -> () + 'static> UpdateFn for T {}
impl<T: FnOnce() -> ()> ShutdownFn for T {}

pub trait UiBackend<Init: InitFn, Update: UpdateFn, Shutdown: ShutdownFn> {
    type RunResultSuccess;
    type RunResultError;

    /// Creates a new
    fn new(init_fn: Init, update_fn: Update, shutdown_fn: Shutdown) -> Self;
    fn run(self) -> Result<Self::RunResultSuccess, Self::RunResultError>;
}
