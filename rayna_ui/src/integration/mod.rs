//! # [rayna_engine::integration]
//! This module acts as the integration ("glue") between the rendering backend for [rayna],
//! and the UI frontend.

use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::integration::worker::BgWorker;
use egui::{Color32, ColorImage};
use image::buffer::ConvertBuffer;
use image::RgbaImage;
use puffin::profile_scope;
use rayna_engine::def::types::ImgBuf;
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
}

pub(crate) struct Integration {
    msg_tx: flume::Sender<MessageToWorker>,
    msg_rx: flume::Receiver<MessageToUi>,
    render_rx: flume::Receiver<Render<ImgBuf>>,
    worker_thread: JoinHandle<()>,
}

impl Integration {
    pub(crate) fn new(initial_render_opts: &RenderOpts, initial_scene: &Scene) -> Self {
        // Main thread -> Worker
        let (main_tx, work_rx) = flume::unbounded::<MessageToWorker>();
        // Worker -> Main thread
        let (work_tx, main_rx) = flume::unbounded::<MessageToUi>();
        // Worker  -> Main thread (renders)
        let (rend_tx, rend_rx) = flume::bounded::<Render<ImgBuf>>(1);

        let worker = BgWorker {
            msg_rx: work_rx,
            msg_tx: work_tx,
            render_tx: rend_tx,
            render_opts: initial_render_opts.clone(),
            scene: initial_scene.clone(),
            renderer: Renderer::new().expect("failed to create renderer"),
        };

        let thread = std::thread::Builder::new()
            .name("BgWorker::thread".into())
            .spawn(move || worker.bg_worker())
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

        let render = match self.render_rx.try_recv() {
            Ok(r) => r,
            Err(flume::TryRecvError::Empty) => return None,
            Err(flume::TryRecvError::Disconnected) => {
                return Some(Err(IntegrationError::RenderChannelDisconnected))
            }
        };

        // Got a rendered image, translate to an egui-appropriate one

        let img_as_rgba: RgbaImage = {
            profile_scope!("convert-rgba");
            render.img.convert()
        };

        let img_as_egui = unsafe {
            profile_scope!("convert_egui");

            // SAFETY:
            // Color32 is defined as being a `[u8; 4]` internally anyway
            // And we know that RgbaImage stores pixels as [r, g, b, a]
            // So we can safely transmute the vector, because they have the same
            // internal representation and layout

            // PERFORMANCE:
            // This is massively faster than calling
            // `ColorImage::from_rgba_unmultiplied(size, img_as_rgba.into_vec())`
            // It goes from ~7ms to ~1us
            let (ptr, len, cap) = img_as_rgba.into_vec().into_raw_parts();
            let px = Vec::from_raw_parts(ptr as *mut Color32, len / 4, cap / 4);

            Render {
                img: ColorImage {
                    size: [render.img.width() as usize, render.img.height() as usize],
                    pixels: px,
                },
                stats: render.stats,
            }
        };

        Some(Ok(img_as_egui))
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
