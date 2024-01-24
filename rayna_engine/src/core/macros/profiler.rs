//! This module contains custom profilers

#[macro_export]
macro_rules! profiler {
    ($(
        {name: $name:ident, port: $port:expr $(,install: |$install_var:ident| $install:block, drop: |$drop_var:ident| $drop:block)? $(,)?}),*
    $(,)?)
    => {
        $(
            $crate::profiler!(@inner {name: $name, port: $port $(,install: |$install_var| $install, drop: |$drop_var| $drop)?});
        )*
    };

    (@inner {name: $name:ident, port: $port:expr}) => {
        paste::paste!{
            #[doc = concat!("The address to bind the ", std::stringify!([< $name:lower >]), " thread profiler's server to")]
                pub const [< $name:upper _PROFILER_ADDR >] : &'static str
                    = std::concat!("127.0.0.1:", $port);

                /// Installs the server's sink into the custom profiler
                #[doc(hidden)]
                fn [< $name:lower _profiler_server_install >](sink: puffin::FrameSink) -> puffin::FrameSinkId {
                    [< $name:lower _profiler_lock >]().add_sink(sink)
                }

                /// Drops the server's sink and removes from profiler
                #[doc(hidden)]
                fn [< $name:lower _profiler_server_drop >](id: puffin::FrameSinkId){
                    [< $name:lower _profiler_lock >]().remove_sink(id);
                }

                #[doc = concat!("The instance of the ", std::stringify!([< $name:lower >]), " thread profiler's server")]
                pub static [< $name:upper _PROFILER_SERVER >] : once_cell::sync::Lazy<std::sync::Mutex<puffin_http::Server>>
                    = once_cell::sync::Lazy::new(|| {
                        tracing::debug!(
                            target: targets::MAIN,
                            "starting puffin_http server for {} profiler at {}",
                            std::stringify!([<$name:lower>]),
                            [< $name:upper _PROFILER_ADDR >])
                        ;
                        std::sync::Mutex::new(
                            puffin_http::Server::new_custom(
                                [< $name:upper _PROFILER_ADDR >],
                                // Can't use closures in a const context, use fn-pointers instead
                                [< $name:lower _profiler_server_install >],
                                [< $name:lower _profiler_server_drop >],
                            )
                            .expect(&format!("{} puffin_http server failed to start", std::stringify!([<$name:lower>])))
                        )
                    });

                #[doc = concat!("A custom reporter for the ", std::stringify!([< $name:lower >]), " thread reporter")]
                pub fn [< $name:lower _profiler_reporter >] (info: puffin::ThreadInfo, stream: &puffin::StreamInfoRef<'_>) {
                    [< $name:lower _profiler_lock >]().report(info, stream)
                }

                #[doc = concat!("Accessor for the ", std::stringify!([< $name:lower >]), " thread reporter")]
                pub fn [< $name:lower _profiler_lock >]() -> std::sync::MutexGuard<'static, puffin::GlobalProfiler> {
                    static [< $name _PROFILER >] : once_cell::sync::Lazy<std::sync::Mutex<puffin::GlobalProfiler>> = once_cell::sync::Lazy::new(Default::default);
                    [< $name _PROFILER >].lock().expect("poisoned std::sync::mutex")
                }

                #[doc = concat!("Initialises the ", std::stringify!([< $name:lower >]), " thread reporter and server.\
                Call this on each different thread you want to register with this profiler")]
                pub fn [< $name:lower _profiler_init >]() {
                    tracing::trace!(target: targets::MAIN, "init thread profiler \"{}\"", std::stringify!([<$name:lower>]));
                    std::mem::drop([< $name:upper _PROFILER_SERVER >].lock());
                    tracing::trace!(target: targets::MAIN, "set thread custom profiler \"{}\"", std::stringify!([<$name:lower>]));
                    puffin::ThreadProfiler::initialize(::puffin::now_ns, [< $name:lower _profiler_reporter >]);
                }
        }
    };
}
