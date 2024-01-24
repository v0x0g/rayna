use crate::core::targets;
use core::default::Default;
use once_cell::sync::Lazy;
use puffin::{FrameSink, FrameSinkId, GlobalProfiler, StreamInfoRef, ThreadInfo, ThreadProfiler};
use puffin_http::Server;
use std::stringify;
use std::sync::{Mutex, MutexGuard};

crate::profiler! {
    {name: RENDERER, port: 8588},
}
