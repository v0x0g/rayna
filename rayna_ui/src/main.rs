#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(associated_type_defaults)]
#![feature(error_generic_member_access)]
#![feature(slice_as_chunks)] // Used by [`thiserror::Error`] and `#[source]`

use crate::def::targets::*;
use crate::rayna_app::RaynaApp;
use rayna_ui_base::backend;
use rayna_ui_base::backend::UiBackend;
use std::collections::HashMap;
use tracing::debug;
use tracing::metadata::LevelFilter;
use tracing_subscriber::util::SubscriberInitExt;

pub mod def;
mod ext;
mod integration;
mod rayna_app;

/// Gets a map of all the [`UiBackend`] implementations available
fn backends() -> HashMap<&'static str, Box<dyn UiBackend>> {
    let mut backends: HashMap<&'static str, Box<dyn UiBackend>> = HashMap::new();
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

    backends
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .pretty()
        .with_ansi(true)
        .log_internal_errors(true)
        .with_max_level(LevelFilter::TRACE)
        .with_line_number(true)
        .with_file(true)
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .finish()
        .init();

    puffin::set_scopes_on(true);

    // TODO: Better backend selection that's not just hardcoded
    // let backend = backends
    //     .into_iter()
    //     .next()
    //     .expect("at least one backend should be enabled")
    //     .1;
    let mut backends = backends();
    let backend = backends.remove("miniquad").unwrap();
    debug!(target: MAIN, "using backend {backend:?}");

    debug!(target: MAIN, "run");
    match backend.run(
        def::constants::APP_NAME,
        Box::new(|ctx| Box::new(RaynaApp::new_ctx(ctx))),
    ) {
        Ok(()) => debug!(target: MAIN, "run complete (success)"),
        Err(e) => debug!(target: MAIN, err = ?e, "run complete (error)"),
    }

    Ok(())
}
