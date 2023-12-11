//! This module contains string definitions for different log targets for the [`tracing`] crate
//! Used in macros like [`tracing::info`]

#[macro_export]
macro_rules! tracing_targets {
    {$( $name:ident $(=$val:expr)? ),* $(,)?} => {
        $( tracing_targets!(@value $name $(=$val)? ); )*
    };

    (@value $name:ident = $val:expr) => {pub const $name: &'static str = concat!(env!("CARGO_PKG_NAME"), "::", $val);};
    (@value $name:ident)             => {tracing_targets!($name = stringify!($name));};
}

tracing_targets! {
    RENDERER = "renderer"
}
