//! # TODO:
//! This module is a stub until I can find a good solution for working with the [`noise`] crate

pub mod boxed;

pub trait Noise<const D: usize>: crate::shared::ComponentRequirements {}
