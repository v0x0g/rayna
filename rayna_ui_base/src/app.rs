/// A trait representing an uninitialised application
///
/// The app will be initialised by calling [Self::init], which allows for
/// [egui] to be configured via the [egui::Context] parameter (e.g. setting fonts),
/// before the app actually runs
pub trait UninitApp: 'static {
    type InitApp: App;

    /// Initialises the application, returning the initialised app
    ///
    /// Can take in an [egui::Context] to setup things before the app starts
    fn init(self, ctx: &egui::Context) -> Self::InitApp;

    /// Gets the application's name
    fn app_name<'l>() -> &'l str;
}

/// A trait representing an initialised application that is running
///
/// Obtained by calling
pub trait App: 'static {
    /// Trait for a function that is called each frame.
    ///
    /// This will be where the rendering occurs
    fn on_update(&mut self, ctx: &egui::Context) -> ();
    /// Called when the app is being shut down
    fn on_shutdown(self) -> ();
}
