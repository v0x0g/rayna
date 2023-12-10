//! # [rayna_core::integration]
//! This module acts as the integration ("glue") between the rendering backend for [rayna],
//! and the UI frontend.

use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::integration::worker::BgWorker;
use itertools::Itertools;
use rayna_core::def::types::ImgBuf;
use rayna_core::render::render_opts::RenderOpts;
use std::backtrace::Backtrace;
use std::collections::VecDeque;
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
        source: flume::RecvError,
        #[backtrace]
        backtrace: Backtrace,
    },
}

pub(crate) struct Integration {
    msg_tx: flume::Sender<MessageToWorker>,
    msg_rx: flume::Receiver<MessageToUi>,
    worker_thread: JoinHandle<()>,
    /// Buffer for incoming messages (so we can peek certain message types)
    rx_buffer: VecDeque<MessageToUi>,
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
            rx_buffer: VecDeque::new(),
        }
    }

    // region ===== SENDING =====

    fn send_message(&self, message: MessageToWorker) -> Result<()> {
        self.msg_tx.send(message).map_err(IntegrationError::from)
    }

    /// Sends a message to the worker, telling it to update the render
    pub fn update_render_opts(&self, render_opts: RenderOpts) -> Result<()> {
        self.send_message(MessageToWorker::SetRenderOpts(render_opts))
    }

    // endregion

    // region ===== RECEIVING =====

    /// Internal function that receives all available messages into the [Self::rx_buffer]
    fn recv_all(&mut self) -> Result<()> {
        loop {
            match self.msg_rx.try_recv() {
                Ok(msg) => self.rx_buffer.push_back(msg),
                Err(flume::TryRecvError::Empty) => return Ok(()),
                Err(flume::TryRecvError::Disconnected) => {
                    return Err(IntegrationError::from(flume::RecvError::Disconnected))
                }
            }
        }
    }

    /// Tries to receive the next render from the worker
    ///
    /// # Return Value
    /// The outer [`Result`] corresponds to whether there was an error during message reception,
    /// or all messages were received successfully. The inner [`Option`] corresponds to whether or not there was
    pub fn get_next_render(&mut self) -> Result<Option<ImgBuf>> {
        self.recv_all()?;

        // Position in [rx_buffer] of the next message that contains a completed render
        let next_render_msg_pos = self
            .rx_buffer
            .iter()
            .find_position(|m| matches!(m, MessageToUi::RenderFrameComplete(..)))
            .map(|(p, _)| p);

        return match next_render_msg_pos {
            Some(pos) => {
                let msg = self.rx_buffer.remove(pos);
                let Some(MessageToUi::RenderFrameComplete(buf)) = msg else {
                    // SAFETY: In the `find_position()` call we validated the message was
                    // a `RenderFrameCompleted` variant, and that the element existed
                    // Therefore this is impossible to reach and is safe
                    unreachable!("impossible for message not to match if reached here")
                };

                Ok(Some(buf))
            }
            None => Ok(None),
        };
    }

    // endregion
}
