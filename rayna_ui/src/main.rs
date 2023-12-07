#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(associated_type_defaults)]

use crate::rayna_app::RaynaAppUninit;
use rayna_ui_base::backend;
use rayna_ui_base::backend::UiBackend;
use std::collections::HashMap;

pub mod definitions;
mod rayna_app;

fn main() -> anyhow::Result<()> {
    let app = RaynaAppUninit;

    let mut backends = HashMap::new();
    #[cfg(feature = "backend_eframe")]
    backends.insert("eframe", Box::new(backend::eframe::EframeBackend {}));

    // TODO: Better backend selection that's not just hardcoded
    // let backend = backends
    //     .into_iter()
    //     .next()
    //     .expect("at least one backend should be enabled")
    //     .1;
    let backend = backends.remove("eframe").unwrap();

    backend.run(app)?;

    Ok(())
}
