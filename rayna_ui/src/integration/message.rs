use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::scene::SceneBuilder;

/// A message sent by the UI to the worker
#[derive(Debug, Clone)]
pub(crate) enum MessageToWorker {
    SetRenderOpts(RenderOpts),
    SetScene(SceneBuilder),
}

/// A message sent from the worker, to the UI
#[derive(Clone, Debug)]
pub(crate) enum MessageToUi {}
