#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(associated_type_defaults)]

use crate::backend::UiBackend;
use app::rayna_app::RaynaAppUninit;
use std::error::Error;

mod app;
mod backend;
mod definitions;

fn main() -> Result<(), Box<dyn Error>> {
    backend::eframe::EFrameBackend::run(RaynaAppUninit {})
}
