#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(associated_type_defaults)]
#![feature(error_generic_member_access)] // Used by [`thiserror::Error`] and `#[source]`

use crate::rayna_app::RaynaApp;
use rayna_ui_base::backend;
use rayna_ui_base::backend::UiBackend;
use std::collections::HashMap;

pub mod definitions;
mod integration;
mod rayna_app;

fn main() -> anyhow::Result<()> {
    let mut backends = HashMap::new();
    #[cfg(feature = "backend_eframe")]
    backends.insert("eframe", Box::new(backend::eframe::EframeBackend {}));
    #[cfg(feature = "backend_miniquad")]
    backends.insert("miniquad", Box::new(backend::eframe::EframeBackend {}));

    // TODO: Better backend selection that's not just hardcoded
    // let backend = backends
    //     .into_iter()
    //     .next()
    //     .expect("at least one backend should be enabled")
    //     .1;
    let backend = backends.remove("miniquad").unwrap();

    backend.run(definitions::constants::APP_NAME, RaynaApp::new_ctx)?;

    Ok(())
}
