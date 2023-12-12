//! # [rayna_engine::integration]
//! This module acts as the integration ("glue") between the rendering backend for [rayna],
//! and the UI frontend.

use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::integration::worker::BgWorker;
use rayna_engine::def::types::ImgBuf;
use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::shared::scene::Scene;
use std::thread::JoinHandle;
use thiserror::Error;
use tracing::error;

pub mod message;
mod worker;

pub type Result<T> = core::result::Result<T, IntegrationError>;

#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("message channel to background worker disconnected")]
    TxChannelDisconnected,
    #[error("message channel from background worker disconnected")]
    RxChannelDisconnected,
    #[error("render channel from background worker disconnected")]
    RenderChannelDisconnected,
    #[error("worker thread died unexpectedly")]
    WorkerThreadDied,
}

pub(crate) struct Integration {
    msg_tx: flume::Sender<MessageToWorker>,
    msg_rx: flume::Receiver<MessageToUi>,
    render_rx: flume::Receiver<ImgBuf>,
    worker_thread: JoinHandle<()>,
}

impl Integration {
    pub(crate) fn new(initial_render_opts: &RenderOpts, initial_scene: &Scene) -> Self {
        // Main thread -> Worker
        let (main_tx, work_rx) = flume::unbounded::<MessageToWorker>();
        // Worker -> Main thread
        let (work_tx, main_rx) = flume::unbounded::<MessageToUi>();
        // Worker  -> Main thread (renders)
        let (rend_tx, rend_rx) = flume::bounded::<ImgBuf>(1);

        let worker = BgWorker {
            msg_rx: work_rx,
            msg_tx: work_tx,
            render_tx: rend_tx,
            render_opts: initial_render_opts.clone(),
            scene: initial_scene.clone(),
        };

        let thread = std::thread::Builder::new()
            .name("integration::BgWorker".into())
            .spawn(move || worker.thread_run())
            // TODO: Error handling if the BgWorker thread fails to spawn
            .expect("failed to spawn thread for BgWorker}");

        Self {
            msg_tx: main_tx,
            msg_rx: main_rx,
            render_rx: rend_rx,
            worker_thread: thread,
        }
    }

    fn ensure_worker_alive(&self) -> Result<()> {
        if self.worker_thread.is_finished() {
            Err(IntegrationError::WorkerThreadDied)
        } else {
            Ok(())
        }
    }

    // region ===== SENDING =====

    /// Sends a message to the worker
    pub fn send_message(&self, message: MessageToWorker) -> Result<()> {
        self.ensure_worker_alive()?;

        self.msg_tx
            .send(message)
            .map_err(|_| IntegrationError::TxChannelDisconnected)
    }

    // endregion

    // region ===== RECEIVING =====

    //noinspection DuplicatedCode - No point extracting five lines
    /// Tries to receive the next render from the worker
    ///
    /// # Return Value
    /// See [Self::try_recv_message]
    pub fn try_recv_render(&self) -> Option<Result<ImgBuf>> {
        if let Err(e) = self.ensure_worker_alive() {
            return Some(Err(e));
        }

        return match self.render_rx.try_recv() {
            Ok(img) => Some(Ok(img)),
            Err(flume::TryRecvError::Empty) => None,
            Err(flume::TryRecvError::Disconnected) => {
                Some(Err(IntegrationError::RenderChannelDisconnected))
            }
        };
    }

    //noinspection DuplicatedCode
    /// Tries to receive the next message from the worker
    ///
    /// # Return Value
    /// The outer [`Result`] corresponds to whether there was an error during message reception,
    /// or all messages were received successfully. The inner [`Option`] corresponds to whether or not there was
    pub fn try_recv_message(&self) -> Option<Result<MessageToUi>> {
        if let Err(e) = self.ensure_worker_alive() {
            return Some(Err(e));
        }

        return match self.msg_rx.try_recv() {
            Ok(msg) => Some(Ok(msg)),
            Err(flume::TryRecvError::Empty) => None,
            Err(flume::TryRecvError::Disconnected) => {
                Some(Err(IntegrationError::RxChannelDisconnected))
            }
        };
    }

    // endregion
}
