use derivative::Derivative;
use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::shared::scene::Scene;

/// A message sent by the UI to the worker
#[derive(Debug, Clone)]
pub(crate) enum MessageToWorker {
    SetRenderOpts(RenderOpts),
    SetScene(Scene),
}

/// A message sent from the worker, to the UI
#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub(crate) enum MessageToUi {}
