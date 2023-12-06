use crate::backend::{InitFn, ShutdownFn, UiBackend, UpdateFn};

pub struct EFrameBackend<Init, Update, Shutdown> {
    init_fn: Init,
    update_fn: Update,
    shutdown_fn: Shutdown,
}

impl<Init: InitFn, Update: UpdateFn, Shutdown: ShutdownFn> UiBackend<Init, Update, Shutdown>
    for EFrameBackend<Init, Update, Shutdown>
{
    type RunResultSuccess = ();
    type RunResultError = eframe::Error;

    fn new(init_fn: Init, update_fn: Update, shutdown_fn: Shutdown) -> Self {
        Self {
            init_fn,
            update_fn,
            shutdown_fn,
        }
    }

    fn run(mut self) -> Result<Self::RunResultSuccess, Self::RunResultError> {
        println!("init");
        (self.init_fn)();

        println!("run");
        eframe::run_simple_native(
            crate::definitions::constants::APP_NAME,
            eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([400.0, 300.0])
                    .with_min_inner_size([300.0, 220.0]),
                ..Default::default()
            },
            move |ctx, _frame| {
                (self.update_fn)(ctx);
            },
        )?;

        println!("shutdown");
        (self.shutdown_fn)();

        Ok(())
    }
}
