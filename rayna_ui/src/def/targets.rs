//! This module contains string definitions for different log targets for the [`tracing`] crate
//! Used in macros like [`tracing::info`]

use rayna_engine::tracing_targets;

tracing_targets! {
    MAIN = "main",
    BG_WORKER = "bg_worker",
    UI = "ui",
    RENDERER = "renderer"
}
