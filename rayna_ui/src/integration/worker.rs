use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::profiler;
use egui::{Color32, ColorImage};
use image::buffer::ConvertBuffer;
use image::RgbaImage;
use puffin::{profile_function, profile_scope};
use rayna_engine::render::render::Render;
use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::render::renderer::Renderer;
use rayna_engine::shared::scene::Scene;
use rayna_shared::def::targets::BG_WORKER;
use rayna_shared::def::types::ImgBuf;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::{info, instrument, trace, warn};

#[derive(Clone, Debug)]
pub(super) struct BgWorker {
    pub render_opts: RenderOpts,
    pub scene: Scene,
    /// Sender for messages from the worker, back to the UI
    pub msg_tx: flume::Sender<MessageToUi>,
    /// Receiver for messages from the UI, to the worker
    pub msg_rx: flume::Receiver<MessageToWorker>,
    pub render_tx: flume::Sender<Render<ColorImage>>,
    pub renderer: Renderer,
}

impl BgWorker {
    /// Starts the worker in a background thread, returning the thread handle
    pub fn start_bg_thread(self) -> std::io::Result<JoinHandle<()>> {
        std::thread::Builder::new()
            .name("BgWorker::thread".into())
            .spawn(move || self.thread_run())
    }

    /// Actually runs the thread
    /// This should be called inside [thread::spawn], it will block
    #[instrument(level = tracing::Level::DEBUG, skip(self), parent = None)]
    pub fn thread_run(self) {
        info!(target: BG_WORKER, "BgWorker thread start");
        profiler::worker_profiler_init();

        let Self {
            msg_tx,
            msg_rx,
            render_tx,
            mut render_opts,
            mut scene,
            renderer,
        } = self;

        loop {
            profiler::worker_profiler_lock().new_frame();

            profile_function!(); // place here not at the start since we are looping

            if msg_rx.is_disconnected() {
                warn!(target: BG_WORKER, "all senders disconnected from channel");
                break;
            }

            // Have two conditions: (empty) or (disconnected)
            // Checked if disconnected above and skip if empty, so just check Ok() here
            {
                profile_scope!("receive_messages");
                while let Ok(msg) = msg_rx.try_recv() {
                    match msg {
                        MessageToWorker::SetRenderOpts(opts) => {
                            trace!(target: BG_WORKER, ?opts, "got render opts from ui");
                            render_opts = opts
                        }
                        MessageToWorker::SetScene(s) => {
                            trace!(target: BG_WORKER, ?scene, "got scene from ui");
                            scene = s;
                        }
                    }
                }
            }

            {
                profile_scope!("waiting_channel_empty");
                // UI hasn't received the last message we sent
                if !msg_tx.is_empty() {
                    trace!(target: BG_WORKER, "channel not empty, waiting");
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                } else {
                    trace!(target: BG_WORKER, "channel empty, sending new image");
                }
            }

            let render_result = {
                profile_scope!("render");
                let render = renderer.render(&scene, &render_opts);
                Render {
                    img: Self::convert_img(render.img),
                    stats: render.stats,
                }
            };

            {
                profile_scope!("send_frame");

                if let Err(_) = render_tx.send(render_result) {
                    warn!(target: BG_WORKER, "failed to send rendered frame to UI")
                }
            }
        }

        info!(target: BG_WORKER, "BgWorker thread exit");
    }

    /// Converts the image outputted by the renderer into an egui-appropriate one
    fn convert_img(img: ImgBuf) -> ColorImage {
        profile_function!();

        // Got a rendered image, translate to an egui-appropriate one

        let img_as_rgba: RgbaImage = {
            profile_scope!("convert-rgba");
            img.convert()
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

            ColorImage {
                size: [img.width() as usize, img.height() as usize],
                pixels: px,
            }
        };

        img_as_egui
    }
}
