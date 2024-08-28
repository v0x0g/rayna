//! TODO: Crate docs
//!

// Be aggressive on warnings
#![deny(rustdoc::all)]
#![deny(clippy::all)]
#![warn(
    warnings,
    future_incompatible,
    keyword_idents,
    let_underscore,
    nonstandard_style,
    refining_impl_trait,
    unused
)]
// Don't allow any warnings in doctests
#![doc(test(attr(deny(all))))]

pub mod core;
pub mod macros;
pub mod material;
pub mod math;
pub mod mesh;
pub mod noise;
pub mod object;
pub mod render;
pub mod scene;
pub mod skybox;
pub mod texture;
