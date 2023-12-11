//! # [rayna_core::integration]
//! This module acts as the integration ("glue") between the rendering backend for [rayna],
//! and the UI frontend.

use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::integration::worker::BgWorker;
use rayna_core::def::types::Vec3;
use rayna_core::obj::sphere::Sphere;
use rayna_core::scene;
use rayna_core::shared::camera::Camera;
use std::thread::JoinHandle;
use thiserror::Error;
use tracing::error;

pub mod message;
mod worker;

pub type Result<T> = core::result::Result<T, IntegrationError>;

#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("channel to background worker disconnected")]
    TxChannelDisconnected,
    #[error("channel from background worker disconnected")]
    RxChannelDisconnected,
    #[error("worker thread died unexpectedly")]
    WorkerThreadDied,
}

pub(crate) struct Integration {
    msg_tx: flume::Sender<MessageToWorker>,
    msg_rx: flume::Receiver<MessageToUi>,
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
            scene: scene! {
                camera: Camera {
                    look_from: Vec3::new(0., 0., -1.),
                    look_towards: Vec3::ZERO,
                    up_vector: Vec3::Y,
                    focus_dist: 1.,
                    lens_radius: 0.,
                    vertical_fov: 90.
                },
                objects: [
                    Sphere {
                        pos: Vec3::new(0., 0., 0.),
                        radius: 0.5
                    }
                ]
            },
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
