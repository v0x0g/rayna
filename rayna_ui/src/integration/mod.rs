//! # [rayna_engine::integration]
//! This module acts as the integration ("glue") between the rendering backend for [rayna],
//! and the UI frontend.

use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::integration::worker::BgWorker;
use egui::ColorImage;
use rayna_engine::render::render::Render;
use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::render::renderer::Renderer;
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
    #[error("failed to spawn thread for BgWorker")]
    WorkerSpawnFailed(#[from] std::io::Error),
}

pub(crate) struct Integration {
    msg_tx: flume::Sender<MessageToWorker>,
    msg_rx: flume::Receiver<MessageToUi>,
    render_rx: flume::Receiver<Render<ColorImage>>,
    worker_thread: JoinHandle<()>,
}

impl Integration {
    pub(crate) fn new(initial_render_opts: &RenderOpts, initial_scene: &Scene) -> Result<Self> {
        // Main thread -> Worker
        let (main_tx, work_rx) = flume::unbounded::<MessageToWorker>();
        // Worker -> Main thread
        let (work_tx, main_rx) = flume::unbounded::<MessageToUi>();
        // Worker  -> Main thread (renders)
        let (rend_tx, rend_rx) = flume::bounded::<Render<ColorImage>>(1);

        let worker = BgWorker {
            msg_rx: work_rx,
            msg_tx: work_tx,
            render_tx: rend_tx,
            render_opts: initial_render_opts.clone(),
            scene: initial_scene.clone(),
            renderer: Renderer::new().expect("failed to create renderer"),
        };

        let thread = worker.start_bg_thread().map_err(IntegrationError::from)?;

        Ok(Self {
            msg_tx: main_tx,
            msg_rx: main_rx,
            render_rx: rend_rx,
            worker_thread: thread,
        })
    }

    fn ensure_worker_alive(&self) -> Result<()> {
        puffin::profile_function!();

        if self.worker_thread.is_finished() {
            Err(IntegrationError::WorkerThreadDied)
        } else {
            Ok(())
        }
    }

    // region ===== SENDING =====

    /// Sends a message to the worker
    pub fn send_message(&self, message: MessageToWorker) -> Result<()> {
        puffin::profile_function!();

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
    pub fn try_recv_render(&self) -> Option<Result<Render<ColorImage>>> {
        puffin::profile_function!();

        if let Err(e) = self.ensure_worker_alive() {
            return Some(Err(e));
        }

        return match self.render_rx.try_recv() {
            Ok(render) => Some(Ok(render)),
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
        puffin::profile_function!();

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
