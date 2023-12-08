use crate::app::App;
use crate::backend::UiBackend;
use anyhow::anyhow;
use eframe::Theme;

pub struct EframeBackend;

impl UiBackend for EframeBackend {
    fn run<A: App>(
        self,
        app_name: &str,
        app_ctor: impl FnOnce(&egui::Context) -> A + 'static,
    ) -> anyhow::Result<()> {
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
                let app = app_ctor(&ctx.egui_ctx);
                let wrapper = EframeWrapper { app: Some(app) };
                Box::new(wrapper) as Box<dyn eframe::App>
            }),
        )
        .map_err(|e| anyhow!("failed running eframe: {e:#?}"))?;

        Ok(())
    }
}

/// Internal struct that acts as an app instance for [eframe]
///
/// # Notes
/// Use an [Option] for the app, so that we can safely consume [T] on shutdown,
/// as per [App::on_shutdown] contract
struct EframeWrapper<T> {
    app: Option<T>,
}

impl<A: App> eframe::App for EframeWrapper<A> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.app
            .as_mut()
            .expect("on_update called after app has been consumed (shutdown)")
            .on_update(ctx);
    }

    fn on_exit(&mut self, _glow: Option<&eframe::glow::Context>) {
        self.app
            .take()
            .expect("on_exit called after app has been consumed (shutdown)")
            .on_shutdown();
    }
}
