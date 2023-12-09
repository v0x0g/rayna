use crate::integration::message::{MessageToUi, MessageToWorker};
use rayna_core::render::render_opts::RenderOpts;
use rayna_core::scene::Scene;

#[derive(Clone, Debug)]
pub(super) struct BgWorker {
    pub render_opts: RenderOpts,
    pub scene: Option<Scene>,
    /// Sender for messages from the worker, back to the UI
    pub msg_tx: flume::Sender<MessageToUi>,
    /// Receiver for messages from the UI, to the worker
    pub msg_rx: flume::Receiver<MessageToWorker>,
}

impl BgWorker {
    pub fn thread_run(self) {
        let Self {
            msg_tx: tx,
            msg_rx: rx,
            render_opts,
            scene,
        } = self;

        loop {
            if rx.is_disconnected() {
                println!("BgWorker: all senders disconnected from channel");
                break;
            }
        }

        println!("BgWorker: thread exit")
    }
}
