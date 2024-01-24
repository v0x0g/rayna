#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(associated_type_defaults)]
#![feature(error_generic_member_access)]
#![feature(slice_as_chunks)]
#![feature(vec_into_raw_parts)] // Used by [`thiserror::Error`] and `#[source]`

use crate::backend::UiBackend;
use crate::rayna_app::RaynaApp;
use crate::targets::*;
use crate::ui_val::APP_NAME;
use std::collections::HashMap;
use tracing::metadata::LevelFilter;
use tracing::{debug, trace};
use tracing_subscriber::util::SubscriberInitExt;

mod backend;
mod ext;
mod integration;
mod profiler;
mod rayna_app;
pub(crate) mod targets;
mod ui_val;

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
        .with_max_level(LevelFilter::DEBUG)
        .with_line_number(true)
        .with_file(true)
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .finish()
        .init();

    debug!(target: MAIN, "init puffin");
    // Profiling is pretty low-cost
    trace!(target: MAIN, "enable profiling");
    puffin::set_scopes_on(true);
    trace!(target: MAIN, "init main profiler");
    profiler::main_profiler_init();
    // Special handling so the 'default' profiler passes on to our custom profiler
    // In this case, we already overrode the ThreadProfiler for "main" using `main_profiler_init()`,
    // So the events are already going to our custom profiler, but egui still calls `new_frame()` on the
    // global profiler. So here, pass along the `new_frame()`s to the custom one
    puffin::GlobalProfiler::lock().add_sink(Box::new(|_| {
        if profiler::EGUI_CALLS_PUFFIN {
            profiler::main_profiler_lock().new_frame();
        }
    }));

    // TODO: Better backend selection that's not just hardcoded
    // let backend = backends
    //     .into_iter()
    //     .next()
    //     .expect("at least one backend should be enabled")
    //     .1;
    let mut backends = backends();
    let backend = backends.remove("eframe").unwrap();
    debug!(target: MAIN, "using backend {backend:?}");

    debug!(target: MAIN, "run");
    match backend.run(APP_NAME, Box::new(|ctx| Box::new(RaynaApp::new_ctx(ctx)))) {
        Ok(()) => debug!(target: MAIN, "run complete (success)"),
        Err(e) => debug!(target: MAIN, err = ?e, "run complete (error)"),
    }

    Ok(())
}
