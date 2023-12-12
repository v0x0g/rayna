use crate::def::targets::BG_WORKER;
use crate::integration::message::{MessageToUi, MessageToWorker};
use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::render::renderer;
use rayna_engine::shared::scene::Scene;
use std::time::Duration;
use tracing::{info, instrument, trace, warn};

#[derive(Clone, Debug)]
pub(super) struct BgWorker {
    pub render_opts: RenderOpts,
    pub scene: Scene,
    /// Sender for messages from the worker, back to the UI
    pub msg_tx: flume::Sender<MessageToUi>,
    /// Receiver for messages from the UI, to the worker
    pub msg_rx: flume::Receiver<MessageToWorker>,
}

impl BgWorker {
    #[instrument(level = tracing::Level::DEBUG, skip(self), parent = None)]
    pub fn thread_run(self) {
        info!(target: BG_WORKER, "BgWorker thread start");

        let Self {
            msg_tx: tx,
            msg_rx: rx,
            mut render_opts,
            mut scene,
        } = self;

        loop {
            if rx.is_disconnected() {
                warn!(target: BG_WORKER, "all senders disconnected from channel");
                break;
            }

            // Have two conditions: (empty) or (disconnected)
            // Checked if disconnected above and skip if empty, so just check Ok() here
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    MessageToWorker::SetRenderOpts(opts) => {
                        trace!(target: BG_WORKER, ?opts, "got render opts from ui");
                        render_opts = opts
                    }
                    MessageToWorker::SetScene(s) => {
                        trace!(target: BG_WORKER, ?scene, "got scene from ui");
                        scene = s;
                    }
                }
            }

            if !tx.is_empty() {
                trace!(target: BG_WORKER, "channel not empty, waiting");
                std::thread::sleep(Duration::from_millis(1000));
                continue;
            } else {
                trace!(target: BG_WORKER, "channel empty, sending new image");
            }

            let img = renderer::render(&scene, render_opts);

            if let Err(_) = tx.send(MessageToUi::RenderFrameComplete(img)) {
                warn!(target: BG_WORKER, "failed to send rendered frame to UI")
            }
        }

        info!(target: BG_WORKER, "BgWorker thread exit");
    }
}
