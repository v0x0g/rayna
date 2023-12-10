use crate::app::AppCtor;
use std::fmt::Debug;

#[cfg(feature = "backend_eframe")]
pub mod eframe;

#[cfg(feature = "backend_miniquad")]
pub mod miniquad;

/// A trait that represents a type that can be used as a backend for the UI
pub trait UiBackend: Debug {
    /// Runs the UI
    ///
    /// Takes in a function (possibly a closure) that will initialise (and return) the application
    /// instance.
    ///
    /// # Note
    /// Both the app and closure are boxed for object-safe-ness reasons (dynamic dispatch)
    fn run(self: Box<Self>, app_name: &str, app_ctor: AppCtor) -> anyhow::Result<()>;
}
