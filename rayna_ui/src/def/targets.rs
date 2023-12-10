//! This module contains string definitions for different log targets for the [`tracing`] crate
//! Used in macros like [`tracing::info`]

macro_rules! targets {
    {$( $name:ident $(=$val:expr)? ),* $(,)?} => {
        $( targets!(@value $name $(=$val)? ); )*
    };

    (@value $name:ident = $val:expr) => {pub const $name: &'static str = concat!(env!("CARGO_PKG_NAME"), "::", $val);};
    (@value $name:ident)             => {targets!($name = stringify!($name));};
}

targets! {
    MAIN = "main",
    BG_WORKER = "bg_worker",
    UI = "ui"
}
