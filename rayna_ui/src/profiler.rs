/// A special flag used to mark whether [egui][egui] or [eframe][eframe] are calling
/// [`puffin::GlobalProfiler::new_frame()`] for us. It controls (in `rayna_ui`) whether the
/// app calls `new_frame` itself, or lets [egui][egui] do so, as well as the pass-through to
/// our custom profiler (since egui calls [`puffin::GlobalProfiler::lock()`], which locks the global
/// profiler, not our own.
///
/// [egui]: <crates.io/crates/egui>
/// [eframe]: <crates.io/crates/eframe>
pub static EGUI_CALLS_PUFFIN: bool = false;

rayna_engine::profiler! {
    crate::targets::MAIN,
    {name: main,     port: 8587},
}
