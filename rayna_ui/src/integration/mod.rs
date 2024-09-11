//! # Module [crate::integration]
//!
//! This module acts as the integration ("glue") between the rendering backend for **rayna**,
//! and the UI frontend.

use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::integration::worker::BgWorker;
use crate::targets::*;
use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::render::renderer::Renderer;
use rayna_engine::scene::camera::Camera;
use rayna_engine::scene::Scene;
use std::any::Any;
use std::sync::Arc;
use std::thread::JoinHandle;
use thiserror::Error;
use tracing::{debug, error, trace};

pub(crate) mod message;
pub(crate) mod worker;

// TODO: Refactor how the worker is handled

#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("message channel to background worker disconnected")]
    TxChannelDisconnected,
    #[error("message channel from background worker disconnected")]
    RxChannelDisconnected,
    #[error("render channel from background worker disconnected")]
    RenderChannelDisconnected,
    #[error("worker thread died unexpectedly")]
    WorkerDied(Arc<dyn Any + Send + 'static>),
    #[error("failed to spawn thread for BgWorker")]
    WorkerSpawnFailed(#[from] std::io::Error),
}

pub(crate) struct Integration {
    msg_tx: flume::Sender<MessageToWorker>,
    msg_rx: flume::Receiver<MessageToUi>,
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
    /// The worker thread had an oopsie, and pooped it's pants. Here's the error message
    Errored(Arc<dyn Any + Send + 'static>),
}

impl Integration {
    pub(crate) fn new(
        initial_render_opts: &RenderOpts,
        initial_scene: &Scene,
        initial_camera: &Camera,
    ) -> Result<Self, IntegrationError> {
        debug!(target: INTEGRATION, "creating new integration instance");

        trace!(target: INTEGRATION, "creating channels");
        // Main thread -> Worker
        let (main_tx, work_rx) = flume::unbounded::<MessageToWorker>();
        // Worker -> Main thread
        let (work_tx, main_rx) = flume::unbounded::<MessageToUi>();

        trace!(target: INTEGRATION, "creating worker");
        let worker = BgWorker {
            msg_rx: work_rx,
            msg_tx: work_tx,
            renderer: Renderer::new_from(
                initial_scene.clone(),
                initial_camera.clone(),
                initial_render_opts.clone(),
                6,
            )
            .expect("failed to create renderer"),
        };
        let thread = worker.start_bg_thread().map_err(IntegrationError::from)?;

        Ok(Self {
            msg_tx: main_tx,
            msg_rx: main_rx,
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
                let err: Arc<dyn Any + Send + 'static> = match ret_value {
                    Ok(()) => Arc::new(()),
                    Err(e) => Arc::from(e),
                };
                self.worker_handle = WorkerHandle::Errored(err.clone());
                return Err(IntegrationError::WorkerDied(err.clone()));
            }
        } else if let WorkerHandle::Errored(ref e) = self.worker_handle {
            return Err(IntegrationError::WorkerDied(e.clone()));
        }

        Ok(())
    }

    /// Sends a message to the worker
    pub fn send_message(&mut self, message: MessageToWorker) -> Result<(), IntegrationError> {
        puffin::profile_function!();

        self.ensure_worker_alive()?;

        self.msg_tx
            .send(message)
            .map_err(|_| IntegrationError::TxChannelDisconnected)
    }

    /// Tries to receive the next message from the worker
    ///
    /// # Return Value
    /// The outer [`Result`] corresponds to whether there was an error during message reception,
    /// or all messages were received successfully. The inner [`Option`] corresponds to whether or not there was
    /// a message available
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
}
