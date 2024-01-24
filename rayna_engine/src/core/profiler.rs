//! This module contains custom profilers

#![allow(unused)] // Generated code isn't smart enough to be picked up

use crate::core::targets;
use core::default::Default;
use once_cell::sync::Lazy;
use puffin::{FrameSink, FrameSinkId, GlobalProfiler, StreamInfoRef, ThreadInfo, ThreadProfiler};
use puffin_http::Server;
use std::stringify;

macro_rules! profiler {
    ($(
        {name: $name:ident, port: $port:expr $(,install: |$install_var:ident| $install:block, drop: |$drop_var:ident| $drop:block)? $(,)?}),*
    $(,)?)
    => {
        $(
            profiler!(@inner {name: $name, port: $port $(,install: |$install_var| $install, drop: |$drop_var| $drop)?});
        )*
    };

    (@inner {name: $name:ident, port: $port:expr}) => {
        paste::paste!{
            #[doc = concat!("The address to bind the ", stringify!([< $name:lower >]), " thread profiler's server to")]
                pub const [< $name:upper _PROFILER_ADDR >] : &'static str
                    = std::concat!("127.0.0.1:", $port);

                /// Installs the server's sink into the custom profiler
                #[doc(hidden)]
                fn [< $name:lower _profiler_server_install >](sink: FrameSink) -> FrameSinkId {
                    [< $name:lower _profiler_lock >]().add_sink(sink)
                }

                /// Drops the server's sink and removes from profiler
                #[doc(hidden)]
                fn [< $name:lower _profiler_server_drop >](id: FrameSinkId){
                    [< $name:lower _profiler_lock >]().remove_sink(id);
                }

                #[doc = concat!("The instance of the ", stringify!([< $name:lower >]), " thread profiler's server")]
                pub static [< $name:upper _PROFILER_SERVER >] : Lazy<Mutex<Server>>
                    = Lazy::new(|| {
                        tracing::debug!(target: targets::MAIN, "starting puffin_http server for {} profiler at {}", stringify!([<$name:lower>]), [< $name:upper _PROFILER_ADDR >]);
                        Mutex::new(
                            Server::new_custom(
                                [< $name:upper _PROFILER_ADDR >],
                                // Can't use closures in a const context, use fn-pointers instead
                                [< $name:lower _profiler_server_install >],
                                [< $name:lower _profiler_server_drop >],
                            )
                            .expect(&format!("{} puffin_http server failed to start", stringify!([<$name:lower>])))
                        )
                    });

                #[doc = concat!("A custom reporter for the ", stringify!([< $name:lower >]), " thread reporter")]
                pub fn [< $name:lower _profiler_reporter >] (info: ThreadInfo, stream: &StreamInfoRef<'_>) {
                    [< $name:lower _profiler_lock >]().report(info, stream)
                }

                #[doc = concat!("Accessor for the ", stringify!([< $name:lower >]), " thread reporter")]
                pub fn [< $name:lower _profiler_lock >]() -> MutexGuard<'static, GlobalProfiler> {
                    static [< $name _PROFILER >] : Lazy<Mutex<GlobalProfiler>> = Lazy::new(Default::default);
                    [< $name _PROFILER >].lock().expect("poisoned mutex")
                }

                #[doc = concat!("Initialises the ", stringify!([< $name:lower >]), " thread reporter and server.\
                Call this on each different thread you want to register with this profiler")]
                pub fn [< $name:lower _profiler_init >]() {
                    tracing::trace!(target: targets::MAIN, "init thread profiler \"{}\"", stringify!([<$name:lower>]));
                    std::mem::drop([< $name:upper _PROFILER_SERVER >].lock());
                    tracing::trace!(target: targets::MAIN, "set thread custom profiler \"{}\"", stringify!([<$name:lower>]));
                    ThreadProfiler::initialize(::puffin::now_ns, [< $name:lower _profiler_reporter >]);
                }
        }
    };
}

use std::sync::{Arc, Mutex, MutexGuard};

/// A special flag used to mark whether [egui][egui] (or [eframe][eframe] are calling
/// [GlobalProfiler::new_frame()] for us. It controls (in `rayna_ui`) whether the
/// app calls `new_frame` itself, or lets [egui][egui] do so, as well as the pass-through to
/// our custom profiler (since egui calls [GlobalProfiler::lock()], which locks the global
/// profiler, not our own.
///
/// [egui]: <crates.io/crates/egui>
/// [eframe]: <crates.io/crates/eframe>
pub static EGUI_CALLS_PUFFIN: bool = false;

profiler! {
    {name: MAIN,    port: 8587},
    {name: WORKER,  port: 8588},
}
