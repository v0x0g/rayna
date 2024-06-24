//! # Module [crate::integration]
//!
//! This module acts as the integration ("glue") between the rendering backend for **rayna**,
//! and the UI frontend.

use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::integration::worker::BgWorker;
use crate::targets::INTEGRATION;
use egui::ColorImage;
use rayna_engine::render::render::Render;
use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::render::renderer::Renderer;
use rayna_engine::scene::camera::Camera;
use rayna_engine::scene::StandardScene;
use std::any::Any;
use std::sync::Arc;
use std::thread::JoinHandle;
use thiserror::Error;
use tracing::{debug, error, trace};

pub mod message;
mod worker;

#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("message channel to background worker disconnected")]
    TxChannelDisconnected,
    #[error("message channel from background worker disconnected")]
    RxChannelDisconnected,
    #[error("render channel from background worker disconnected")]
    RenderChannelDisconnected,
    #[error("worker thread died unexpectedly")]
    WorkerDied(Arc<Box<dyn Any + Send + 'static>>),
    #[error("failed to spawn thread for BgWorker")]
    WorkerSpawnFailed(#[from] std::io::Error),
}

pub(crate) struct Integration {
    msg_tx: flume::Sender<MessageToWorker>,
    msg_rx: flume::Receiver<MessageToUi>,
    render_rx: flume::Receiver<Render<ColorImage>>,
    worker_handle: WorkerHandle,
}

enum WorkerHandle {
    /// Worker thread is still running
    Running(JoinHandle<()>),
    // Temporary value used while converting a [WorkerHandle::Running] state that has completed,
    /// into a [WorkerHandle::Errored] state
    ///
    /// # See
    /// <https://i.imgflip.com/15ifk6.jpg>
    #[allow(non_camel_case_types)]
    TechnicalDifficulties_PleaseStandBy,
    /// The worker thread had an oopsie, and pooped it's pants. Here's the error message, nicely double-wrapped up for christmas
    Errored(Arc<Box<dyn Any + Send + 'static>>),
}

impl Integration {
    pub(crate) fn new(
        initial_render_opts: &RenderOpts,
        initial_scene: &StandardScene,
        initial_camera: &Camera,
    ) -> Result<Self, IntegrationError> {
        debug!(target: INTEGRATION, "creating new integration instance");

        trace!(target: INTEGRATION, "creating channels");
        // Main thread -> Worker
        let (main_tx, work_rx) = flume::unbounded::<MessageToWorker>();
        // Worker -> Main thread
        let (work_tx, main_rx) = flume::unbounded::<MessageToUi>();
        // Worker  -> Main thread (renders)
        let (rend_tx, rend_rx) = flume::bounded::<Render<ColorImage>>(1);

        trace!(target: INTEGRATION, "creating worker");
        let worker = BgWorker {
            msg_rx: work_rx,
            msg_tx: work_tx,
            render_tx: rend_tx,
            renderer: Renderer::new_from(
                initial_scene.clone(),
                initial_camera.clone(),
                initial_render_opts.clone(),
            )
            .expect("failed to create renderer"),
        };
        let thread = worker.start_bg_thread().map_err(IntegrationError::from)?;

        Ok(Self {
            msg_tx: main_tx,
            msg_rx: main_rx,
            render_rx: rend_rx,
            worker_handle: WorkerHandle::Running(thread),
        })
    }

    fn ensure_worker_alive(&mut self) -> Result<(), IntegrationError> {
        puffin::profile_function!();

        if let WorkerHandle::Running(ref h_join) = self.worker_handle {
            if h_join.is_finished() {
                trace!(target: INTEGRATION, "worker thread died");
                let WorkerHandle::Running(worker_handle) = std::mem::replace(
                    &mut self.worker_handle,
                    WorkerHandle::TechnicalDifficulties_PleaseStandBy,
                ) else {
                    unreachable!("already matched that worker_handle is `Running`")
                };
                let ret_value = worker_handle.join();
                let err: Arc<Box<dyn Any + Send + 'static>> = match ret_value {
                    Ok(()) => Arc::new(Box::new(())),
                    Err(e) => Arc::new(e),
                };
                self.worker_handle = WorkerHandle::Errored(err.clone());
                return Err(IntegrationError::WorkerDied(err.clone()));
            }
        } else if let WorkerHandle::Errored(ref e) = self.worker_handle {
            return Err(IntegrationError::WorkerDied(e.clone()));
        }

        Ok(())
    }

    // region ===== SENDING =====

    /// Sends a message to the worker
    pub fn send_message(&mut self, message: MessageToWorker) -> Result<(), IntegrationError> {
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
    pub fn try_recv_render(&mut self) -> Option<Result<Render<ColorImage>, IntegrationError>> {
        puffin::profile_function!();

        if let Err(e) = self.ensure_worker_alive() {
            return Some(Err(e));
        }

        return match self.render_rx.try_recv() {
            Ok(render) => Some(Ok(render)),
            Err(flume::TryRecvError::Empty) => None,
            Err(flume::TryRecvError::Disconnected) => Some(Err(IntegrationError::RenderChannelDisconnected)),
        };
    }

    //noinspection DuplicatedCode
    /// Tries to receive the next message from the worker
    ///
    /// # Return Value
    /// The outer [`IResult`] corresponds to whether there was an error during message reception,
    /// or all messages were received successfully. The inner [`Option`] corresponds to whether or not there was
    pub fn try_recv_message(&mut self) -> Option<Result<MessageToUi, IntegrationError>> {
        puffin::profile_function!();

        if let Err(e) = self.ensure_worker_alive() {
            return Some(Err(e));
        }

        return match self.msg_rx.try_recv() {
            Ok(msg) => Some(Ok(msg)),
            Err(flume::TryRecvError::Empty) => None,
            Err(flume::TryRecvError::Disconnected) => Some(Err(IntegrationError::RxChannelDisconnected)),
        };
    }

    // endregion
}
