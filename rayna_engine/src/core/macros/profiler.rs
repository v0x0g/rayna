//! This module contains custom profilers

#[macro_export]
macro_rules! profiler {
    (
        $tracing_target:expr,
        $(
            {name: $name:ident, port: $port:expr $(,install: |$install_var:ident| $install:block, drop: |$drop_var:ident| $drop:block)? $(,)?}
        ),*
    $(,)? )
    => {
        $(
            $crate::profiler!(@inner {tracing_target: $tracing_target, name: $name, port: $port $(,install: |$install_var| $install, drop: |$drop_var| $drop)?});
        )*
    };

    (@inner {tracing_target: $tracing_target:expr, name: $name:ident, port: $port:expr}) => {
        pub mod $name {
            use std::sync::{Mutex, MutexGuard};
            use once_cell::sync::Lazy;
            use puffin_http::Server;
            use puffin::{
                FrameSink, FrameSinkId, StreamInfoRef, ScopeDetails,
                ThreadProfiler, ThreadInfo, GlobalProfiler
            };

            #[doc = concat!("The address to bind the ", stringify!($name), " thread profilers' server to")]
            pub const ADDR: &'static str = concat!("127.0.0.1:", $port);

            /// Installs the server's sink into the custom profiler
            #[doc(hidden)]
            fn install(sink: FrameSink) -> FrameSinkId {
                self::lock().add_sink(sink)
            }

            /// Drops the server's sink and removes from profiler
            #[doc(hidden)]
            fn drop(id: FrameSinkId){
                self::lock().remove_sink(id);
            }

            #[doc = concat!("The instance of the ", stringify!($name), " thread profilers' server")]
            pub static SERVER : Lazy<Mutex<Server>>
                = Lazy::new(|| {
                    tracing::debug!(
                        target: $tracing_target,
                        "starting puffin_http server for {} profiler at {}",
                        stringify!($name),
                        self::ADDR
                    );
                    Mutex::new(
                        Server::new_custom(self::ADDR, self::install, self::drop)
                        .expect(&format!("{} puffin_http server failed to start", stringify!($name)))
                    )
                });

            #[doc = concat!("A custom reporter for the ", std::stringify!($name), " thread reporter")]
            pub fn reporter(info: ThreadInfo, details: &[ScopeDetails], stream: &StreamInfoRef<'_>) {
                self::lock().report(info, details, stream)
            }

            #[doc = concat!("Accessor for the ", stringify!($name), " thread reporter")]
            pub fn lock() -> MutexGuard<'static, GlobalProfiler> {
                static PROFILER: Lazy<Mutex<GlobalProfiler>> = Lazy::new(Default::default);
                PROFILER.lock().expect(&format!("poisoned std::sync::mutex for {}", stringify!($name)))
            }

            #[doc = concat!("Initialises the ", stringify!($name), " thread reporter and server.\
            Call this on each different thread you want to register with this profiler")]
            pub fn init_thread() {
                tracing::trace!(target: $tracing_target, "init thread profiler \"{}\"", stringify!($name));
                std::mem::drop(self::SERVER.lock());
                tracing::trace!(target: $tracing_target, "set thread custom profiler \"{}\"", stringify!($name));
                ThreadProfiler::initialize(::puffin::now_ns, self::reporter);
            }
        }
    };
}
