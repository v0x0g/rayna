//! # [rayna_core::integration]
//! This module acts as the integration ("glue") between the rendering backend for [rayna],
//! and the UI frontend.

use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::integration::worker::BgWorker;
use std::backtrace::Backtrace;
use std::num::NonZeroUsize;
use std::thread::JoinHandle;
use thiserror::Error;

mod message;
mod worker;

pub type Result<T> = core::result::Result<T, IntegrationError>;
#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("channel to background worker disconnected")]
    TxChannelDisconnected {
        #[from]
        source: flume::SendError<MessageToWorker>,
        #[backtrace]
        backtrace: Backtrace,
    },
    #[error("channel from background worker disconnected")]
    RxChannelDisconnected {
        #[from]
        source: flume::SendError<MessageToUi>,
        #[backtrace]
        backtrace: Backtrace,
    },
}

pub(crate) struct Integration {
    pub(self) msg_tx: flume::Sender<MessageToWorker>,
    pub(self) msg_rx: flume::Receiver<MessageToUi>,
    worker_thread: JoinHandle<()>,
}
impl Integration {
    pub(crate) fn new() -> Self {
        // Main thread -> Worker
        let (m_tx, w_rx) = flume::unbounded::<MessageToWorker>();
        // Worker -> Main thread
        let (w_tx, m_rx) = flume::unbounded::<MessageToUi>();

        let worker = BgWorker {
            msg_rx: w_rx,
            msg_tx: w_tx,
            render_opts: Default::default(),
            scene: None,
        };

        let thread = std::thread::Builder::new()
            .name("integration::BgWorker".into())
            .spawn(move || worker.thread_run())
            // TODO: Error handling if the BgWorker thread fails to spawn
            .expect("failed to spawn thread for BgWorker}");

        Self {
            msg_tx: m_tx,
            msg_rx: m_rx,
            worker_thread: thread,
        }
    }

    fn send_message(&self, message: MessageToWorker) -> Result<()> {
        self.msg_tx
            .send(message)
            .map_err(IntegrationError::TxChannelDisconnected)
    }

    pub fn update_target_img_dims(&self, new_dims: [NonZeroUsize; 2]) -> Result<()> {
        self.send_message(MessageToWorker::SetTargetRenderDims(new_dims))
    }
}
