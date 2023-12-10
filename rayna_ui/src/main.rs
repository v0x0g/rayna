#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(associated_type_defaults)]
#![feature(error_generic_member_access)] // Used by [`thiserror::Error`] and `#[source]`

use crate::def::targets::*;
use crate::rayna_app::RaynaApp;
use rayna_ui_base::backend;
use rayna_ui_base::backend::UiBackend;
use std::collections::HashMap;
use tracing::debug;

pub mod def;
mod integration;
mod rayna_app;

fn main() -> anyhow::Result<()> {
    // TODO: Init [tracing]

    let mut backends: HashMap<_, Box<dyn UiBackend>> = HashMap::new();
    #[cfg(feature = "backend_eframe")]
    {
        debug!(target: MAIN, "have backend: eframe");
        backends.insert("eframe", Box::new(backend::eframe::EframeBackend {}));
    }
    #[cfg(feature = "backend_miniquad")]
    {
        debug!(target: MAIN, "have backend: miniquad");
        backends.insert("miniquad", Box::new(backend::miniquad::MiniquadBackend {}))
    };

    // TODO: Better backend selection that's not just hardcoded
    // let backend = backends
    //     .into_iter()
    //     .next()
    //     .expect("at least one backend should be enabled")
    //     .1;
    let backend = backends.remove("miniquad").unwrap();

    backend.run(def::constants::APP_NAME, RaynaApp::new_ctx)?;

    Ok(())
}
