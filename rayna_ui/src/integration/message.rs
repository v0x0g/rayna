use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::scene::camera::Camera;
use rayna_engine::scene::SimpleScene;

/// A message sent by the UI to the worker
#[derive(Debug, Clone)]
pub(crate) enum MessageToWorker {
    SetRenderOpts(RenderOpts),
    SetScene(SimpleScene),
    SetCamera(Camera),
}

/// A message sent from the worker, to the UI
#[derive(Clone, Debug)]
pub(crate) enum MessageToUi {}
