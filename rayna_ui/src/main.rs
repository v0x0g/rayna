#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(associated_type_defaults)]
#![feature(error_generic_member_access)]
#![feature(slice_as_chunks)]
#![feature(vec_into_raw_parts)]

use crate::app::RaynaApp;
use crate::targets::*;
use crate::ui_val::APP_NAME;
use tracing::metadata::LevelFilter;
use tracing::{debug, trace};
use tracing_subscriber::util::SubscriberInitExt;

mod app;
mod backend;
mod ext;
mod integration;
mod profiler;
pub(crate) mod targets;
mod ui_val;

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
    profiler::main::init_thread();
    // Special handling so the 'default' profiler passes on to our custom profiler
    // In this case, we already overrode the ThreadProfiler for "main" using `main_profiler_init()`,
    // So the events are already going to our custom profiler, but egui still calls `new_frame()` on the
    // global profiler. So here, pass along the `new_frame()`s to the custom one
    puffin::GlobalProfiler::lock().add_sink(Box::new(|_| {
        // Skip however if we are calling egui manually, as we don't want to double-call
        if profiler::EGUI_CALLS_PUFFIN {
            profiler::main::lock().new_frame();
        }
    }));

    // TODO: Better backend selection that's not just hardcoded
    let mut backends = backend::get_all::<RaynaApp>();
    let backend = backends.remove("eframe").unwrap();

    debug!(target: MAIN, "run");
    match backend.run(APP_NAME) {
        Ok(()) => debug!(target: MAIN, "run complete (success)"),
        Err(e) => debug!(target: MAIN, err = ?e, "run complete (error)"),
    }

    Ok(())
}
