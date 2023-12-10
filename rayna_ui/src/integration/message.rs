use rayna_core::def::types::ImgBuf;
use rayna_core::render::render_opts::RenderOpts;

/// A message sent by the UI to the worker
#[derive(Debug, Copy, Clone)]
pub(in crate::integration) enum MessageToWorker {
    SetRenderOpts(RenderOpts),
}

/// A message sent from the worker, to the UI
#[derive(Debug, Clone)]
pub(in crate::integration) enum MessageToUi {
    /// A frame has been rendered and is available for display
    RenderFrameComplete(ImgBuf),
}
