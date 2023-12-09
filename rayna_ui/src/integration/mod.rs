//! # [rayna_core::integration]
//! This module acts as the integration ("glue") between the rendering backend for [rayna],
//! and the UI frontend.

use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::integration::worker::BgWorker;
use std::thread::JoinHandle;

mod message;
mod worker;

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
        };
        todo!()
    }
}
