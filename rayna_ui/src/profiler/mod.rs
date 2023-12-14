//! This module contains custom profilers

use puffin::{GlobalProfiler, StreamInfoRef, ThreadInfo};

macro_rules! profiler {
    ($({name: $name:ident, port: $port:expr}),* $(,)?) => {
        paste::paste! {
            $(
                #[doc = concat!("The address to bind the ", std::stringify!([< $name:lower >]), " thread profiler's server to")]
                pub const [< $name:upper _PROFILER_ADDR >] : &'static str
                    = std::concat!("127.0.0.1:", $port);

                #[doc = concat!("The instance of the ", std::stringify!([< $name:lower >]), " thread profiler's server")]
                pub static [< $name:upper _PROFILER_SERVER >] : ::anyhow::Result<::puffin_http::Server>
                    = ::puffin_http::Server::new([< $name:upper _PROFILER_ADDR >])
                        .inspect_err(|err| tracing::warn!(target: $crate::def::targets::MAIN, ?err, "{} puffin_http server failed to start", stringify!([<$name:lower>])));

                #[doc = concat!("A custom reporter for the ", std::stringify!([< $name:lower >]), " thread reporter")]
                pub fn [< $name:lower _profiler_reporter >] (info: ThreadInfo, stream: &StreamInfoRef<'_>) {
                    [< $name:lower _profiler_lock >]().report(info, stream)
                }

                #[doc = concat!("Accessor for the ", std::stringify!([< $name:lower >]), " thread reporter")]
                pub fn [< $name:lower _profiler_lock >]() -> std::sync::MutexGuard<'static, GlobalProfiler> {
                    use once_cell::sync::Lazy;
                    use std::sync::Mutex;

                    static [< $name _PROFILER >] : Lazy<Mutex<GlobalProfiler>> = Lazy::new(Default::default);
                    [< $name _PROFILER >].lock().expect("poisoned mutex")
                }
            )*
        }
    };
}

profiler! {
    {name: MAIN, port: 8585},
    {name: WORKER, port: 8586},
}
