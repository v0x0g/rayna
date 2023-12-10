//! This module contains string definitions for different log targets for the [`tracing`] crate
//! Used in macros like [`tracing::info`]

macro_rules! strs {
    ($( $name:ident $( = $val:expr )? ),*) => {
        $( strs!($name $( = $val )? ) )*
    };

    (@value $name:ident = $val:expr) => {pub const $name: &'static str = $val;};
    (@value $name:ident)             => {pub const $name: &'static str = stringify!($name);};
}

pub const BG_WORKER: &'static str = "BgWorker";
