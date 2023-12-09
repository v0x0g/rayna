use rayna_core::def::types::ImgBuf;
use std::num::NonZeroUsize;

/// A message sent by the UI to the worker
pub(in crate::integration) enum MessageToWorker {
    SetTargetRenderDims([NonZeroUsize; 2]),
}

/// A message sent from the worker, to the UI
pub(in crate::integration) enum MessageToUi {
    /// A frame has been rendered and is available for display
    RenderFrameComplete(ImgBuf),
}
