use derivative::Derivative;
use rayna_core::def::types::ImgBuf;
use rayna_core::render::render_opts::RenderOpts;
use valuable::Valuable;

/// A message sent by the UI to the worker
#[derive(Debug, Copy, Clone, Valuable)]
pub(crate) enum MessageToWorker {
    SetRenderOpts(RenderOpts),
}

/// A message sent from the worker, to the UI
#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub(crate) enum MessageToUi {
    /// A frame has been rendered and is available for display
    RenderFrameComplete(#[derivative(Debug = "ignore")] ImgBuf),
}
