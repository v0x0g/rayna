#![doc = include_str!("../readme.md")]
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
    unused
)]
// Don't allow any warnings in doctests
#![doc(test(attr(deny(all))))]

use std::ops::Deref;

use crate::targets::*;
use clap::Parser;
use tracing::*;
use tracing_subscriber::prelude::*;

pub mod app;
pub mod backend;
pub mod ext;
pub mod integration;
pub mod profiler;
pub mod targets;
pub mod ui_val;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Which UI rendering backend to use
    ///
    /// If not supplied, automatically selects any available backend
    #[arg(short, long)]
    backend: Option<String>,
}

fn main() {
    // ===== CLI Args =====

    let args = Args::parse();

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

    let backend_ctor = match args.backend {
        Some(backend_selection) => match BACKENDS.iter().find(|b| b.0 == backend_selection.deref()) {
            Some(backend) => backend,
            None => {
                error!("cannot select backend {backend_selection}: does not exist");
                info!(
                    "valid backends are: {}",
                    itertools::Itertools::join(&mut BACKENDS.iter().map(|b| b.0), ", ")
                );
                return;
            }
        },
        None => BACKENDS
            .first()
            .expect("failed to auto select backend: no backends available"),
    };

    // Doesn't return Err, expected to always succeed or panic if fatal
    debug!(target: MAIN, "run");
    (backend_ctor.1)().run(&crate::ui_val::APP_NAME);
    debug!(target: MAIN, "run complete");
}

const BACKENDS: &'static [(&'static str, fn() -> Box<dyn backend::UiBackend<app::RaynaApp>>)] = &[
    #[cfg(feature = "backend_eframe")]
    ("eframe", || Box::new(backend::eframe::EframeBackend::new())),
    #[cfg(feature = "backend_miniquad")]
    ("miniquad", || Box::new(backend::miniquad::MiniquadBackend::new())),
];

const_format::assertcp!(BACKENDS.len() > 0, "compiled without any backend flags enabled");
