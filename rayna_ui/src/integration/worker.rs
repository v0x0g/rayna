use crate::integration::message::{MessageToUi, MessageToWorker};
use num_traits::ToPrimitive;
use rayna_core::def::types::{ImgBuf, Pix};
use rayna_core::render::render_opts::RenderOpts;
use rayna_core::scene::Scene;
use std::time::Duration;
use tracing::{info, instrument, warn};

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
    #[instrument(level = tracing::Level::DEBUG, skip(self), parent = None)]
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

            if !tx.is_empty() {
                info!("channel not empty, waiting");
                std::thread::sleep(Duration::from_millis(1000));
                continue;
            } else {
                info!("channel empty, sending new image");
            }

            let [w, h] = [render_opts.width, render_opts.height]
                .map(|x| x.get().to_u32())
                .map(|d| d.expect("image dims failed to fit inside u32"));

            let mut img = ImgBuf::new(w, h);
            img.enumerate_pixels_mut().for_each(|(x, y, p)| {
                *p = if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                    Pix::from([1.0; 3])
                } else {
                    Pix::from([0.0, 1.0, 0.0])
                }
            });

            if let Err(_) = tx.send(MessageToUi::RenderFrameComplete(img)) {
                warn!("failed to send rendered frame to UI")
            }
        }

        info!("BgWorker thread exit");
    }
}
