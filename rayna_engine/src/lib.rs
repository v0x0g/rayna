//! TODO: Crate docs
//!

#![feature(array_chunks)]
#![feature(array_try_map)]
#![feature(iter_map_windows)]
#![feature(iter_collect_into)]
#![feature(const_for)]
#![feature(vec_into_raw_parts)]
#![feature(negative_impls)]
#![feature(trait_alias)]
#![feature(new_uninit)]
#![feature(box_patterns)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]
#![feature(error_generic_member_access)]
#![feature(array_windows)]
#![feature(portable_simd)]
#![feature(iter_array_chunks)]
#![feature(doc_notable_trait)]
// Be aggressive on warnings
#![deny(rustdoc::all)]
#![deny(clippy::all)]

pub mod core;
pub mod material;
pub mod mesh;
pub mod object;
pub mod render;
pub mod scene;
pub mod shared;
pub mod skybox;
pub mod texture;
