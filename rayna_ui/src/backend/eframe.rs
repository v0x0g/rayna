use crate::backend::app::AppCtor;
use crate::backend::UiBackend;
use anyhow::anyhow;
use eframe::Theme;
use egui::ViewportBuilder;
use valuable::Valuable;

#[derive(Debug, Copy, Clone, Valuable)]
pub struct EframeBackend;

impl UiBackend for EframeBackend {
    fn run(self: Box<Self>, app_name: &str, app_ctor: AppCtor) -> anyhow::Result<()> {
        eframe::run_native(
            app_name,
            eframe::NativeOptions {
                run_and_return: true,
                default_theme: Theme::Dark,
                viewport: ViewportBuilder::default()
                    .with_min_inner_size([300.0, 220.0])
                    .with_maximized(true)
                    .with_app_id(app_name),
                vsync: false,
                centered: true,

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

impl eframe::App for Box<dyn crate::backend::app::App> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.on_update(ctx);
    }

    fn on_exit(&mut self, _glow: Option<&eframe::glow::Context>) {
        self.on_shutdown();
    }
}
