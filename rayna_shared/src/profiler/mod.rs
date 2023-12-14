//! This module contains custom profilers

#![allow(unused)] // Generated code isn't smart enough to be picked up

use crate::def::targets;
use core::default::Default;
use once_cell::sync::Lazy;
use puffin::{FrameSink, FrameSinkId, GlobalProfiler, StreamInfoRef, ThreadInfo, ThreadProfiler};
use puffin_http::Server;
use std::stringify;

macro_rules! profiler {
    ($({name: $name:ident, port: $port:expr}),* $(,)?) => {
        paste::paste! {
            $(
                #[doc = concat!("The address to bind the ", stringify!([< $name:lower >]), " thread profiler's server to")]
                pub const [< $name:upper _PROFILER_ADDR >] : &'static str
                    = std::concat!("127.0.0.1:", $port);

                #[doc(hidden)]
                fn [< $name:lower _profiler_server_install >](sink: FrameSink) -> FrameSinkId {
                    [< $name:lower _profiler_lock >]().add_sink(sink)
                }

                /// Drops the server
                #[doc(hidden)]
                fn [< $name:lower _profiler_server_drop >](sink_id: FrameSinkId){
                    [< $name:lower _profiler_lock >]().remove_sink(sink_id);
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
                    Lazy::
                    tracing::trace!(target: targets::MAIN, "init thread profiler \"{}\"", stringify!([<$name:lower>]));
                    std::mem::drop([< $name:upper _PROFILER_SERVER >].lock());
                    tracing::trace!(target: targets::MAIN, "set thread custom profiler \"{}\"", stringify!([<$name:lower>]));
                    ThreadProfiler::initialize(::puffin::now_ns, [< $name:lower _profiler_reporter >]);
                }
            )*
        }
    };
}

use std::sync::{Mutex, MutexGuard};

profiler! {
    {name: MAIN,    port: 8585},
    {name: WORKER,  port: 8586},
}
