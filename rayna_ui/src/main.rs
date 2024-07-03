#![doc = include_str!("../readme.md")]
#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(associated_type_defaults)]
#![feature(error_generic_member_access)]
#![feature(slice_as_chunks)]
#![feature(vec_into_raw_parts)]
// Be aggressive on warnings
#![deny(rustdoc::all)]
#![deny(clippy::all)]
#![warn(
    warnings,
    future_incompatible,
    keyword_idents,
    let_underscore,
    nonstandard_style,
    refining_impl_trait,
    rust_2018_compatibility,
    rust_2021_compatibility,
    rust_2024_compatibility,
    unused
)]
// Don't allow any warnings in doctests
#![doc(test(attr(deny(all))))]

use crate::app::RaynaApp;
use crate::targets::*;
use crate::ui_val::APP_NAME;
use tracing::debug;
use tracing_subscriber::prelude::*;

mod app;
mod backend;
mod ext;
mod integration;
mod profiler;
pub(crate) mod targets;
mod ui_val;

fn main() -> anyhow::Result<()> {
    // ===== Tracing =====

    let stderr_output = tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(true)
        .log_internal_errors(true)
        .with_line_number(true)
        .with_file(true)
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .with_writer(std::sync::Arc::new(std::io::stderr()));

    let log_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::metadata::LevelFilter::INFO.into())
        .with_regex(true)
        .from_env_lossy();

    // Wrap it in an `EnvFilter` so we can configure from the environment variable,
    // and install it as the default log subscriber

    // NOTE: This is a note to future me who forgets how `tracing` works
    //
    // My understanding of tracing is that there is a singleton `Subscriber` instance
    // that controls handling of *ALL* log events that occur. We set this by calling
    // `tracing_subscriber::util::SubscriberInitExt::init()`. We choose the
    // `tracing_subscriber::registry()` as our subscriber - it's fast or something.
    //
    // We then add `Layer`s onto it, using `tracing_subscriber::layer::SubscriberExt`,
    // which branch off the main subscriber, and do their own thing. We use the
    // `tracing_subscriber::fmt::layer()` function to create a layer, which we
    // then configure to our liking (such as setting the output `with_writer()` to stderr.
    tracing_subscriber::registry()
        .with(stderr_output.with_filter(log_filter))
        .init();

    // ===== Profiling =====

    debug!(target: MAIN, "init puffin");
    // Profiling is pretty low-cost
    debug!(target: MAIN, "enable profiling");
    puffin::set_scopes_on(true);
    debug!(target: MAIN, "init main profiler");
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

    // ===== UI Backend =====

    // TODO: Allow backend selection from CLI arguments; use `clap` crate
    let mut backends = backend::get_all::<RaynaApp>();
    let backend = backends.remove("miniquad").unwrap();

    debug!(target: MAIN, "run");
    match backend.run(APP_NAME) {
        Ok(()) => debug!(target: MAIN, "run complete (success)"),
        Err(e) => debug!(target: MAIN, err = ?e, "run complete (error)"),
    }

    Ok(())
}
