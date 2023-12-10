use crate::app::{App, AppCtor};
use crate::backend::UiBackend;
use anyhow::anyhow;
use eframe::Theme;
use valuable::Valuable;

#[derive(Debug, Copy, Clone, Valuable)]
pub struct EframeBackend;

impl UiBackend for EframeBackend {
    fn run(self: Box<Self>, app_name: &str, app_ctor: AppCtor) -> anyhow::Result<()> {
        eframe::run_native(
            app_name,
            eframe::NativeOptions {
                min_window_size: Some([300.0, 220.0].into()),
                initial_window_size: Some([400.0, 300.0].into()),
                run_and_return: true,
                default_theme: Theme::Dark,

                ..Default::default()
            },
            // This closure is called by `eframe` to initialise the app
            // It moves all the functions into itself so that they can be called at the appropriate times
            Box::new(move |ctx: &eframe::CreationContext| {
                let box_app = app_ctor(&ctx.egui_ctx);
                // Box<dyn crate::app> implements eframe::App
                Box::new(box_app) as Box<dyn eframe::App>
            }),
        )
        .map_err(|e| anyhow!("failed running eframe: {e:#?}"))?;

        Ok(())
    }
}

impl eframe::App for Box<dyn App> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.on_update(ctx);
    }

    fn on_exit(&mut self, _glow: Option<&eframe::glow::Context>) {
        self.on_shutdown();
    }
}
