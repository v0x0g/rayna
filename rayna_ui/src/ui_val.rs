//! Strings for use in the UI, such as units or labels

#![allow(unused_variables, dead_code)]
use rayna_shared::def::types::Number;

pub const APP_NAME: &'static str = "rayna";

pub const UNIT_PX: &'static str = " px";
pub const UNIT_DEG: &'static str = " °";
pub const UNIT_LEN: &'static str = " m";

pub const DRAG_SLOW: Number = 0.1;
pub const DRAG_NORM: Number = 1.0;
pub const DRAG_FAST: Number = 3.0;
