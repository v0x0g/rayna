use std::marker::PhantomData;

use crate::backend::UiBackend;
use anyhow::anyhow;
use eframe::Theme;
use egui::ViewportBuilder;

use super::UiApp;

#[derive(Debug, Copy, Clone)]
pub struct EframeBackend<Ctor>(PhantomData<Ctor>);

impl<Ctor> Default for EframeBackend<Ctor> {
    fn default() -> Self { Self(PhantomData::default()) }
}

impl<App: UiApp> UiBackend<App> for EframeBackend<App> {
    fn run(self: Box<Self>, app_name: &str) -> anyhow::Result<()> {
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
                let app: App = App::new(&ctx.egui_ctx);
                let wrapped: Wrapper<App> = Wrapper(app);
                Box::new(wrapped) as Box<dyn ::eframe::App>
            }),
        )
        .map_err(|e| anyhow!("failed running eframe: {e:#?}"))
    }
}

struct Wrapper<App: UiApp>(App);
impl<App: UiApp> ::eframe::App for Wrapper<App> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) { self.0.on_update(ctx); }

    fn on_exit(&mut self, _glow: Option<&eframe::glow::Context>) { self.0.on_shutdown(); }
}
