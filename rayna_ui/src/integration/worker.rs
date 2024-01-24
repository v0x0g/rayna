use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::targets::BG_WORKER;
use egui::{Color32, ColorImage};
use puffin::{profile_function, profile_scope};
use rayna_engine::core::profiler;
use rayna_engine::core::types::{Channel, Image};
use rayna_engine::material::MaterialInstance;
use rayna_engine::mesh::MeshInstance;
use rayna_engine::object::ObjectInstance;
use rayna_engine::render::render::Render;
use rayna_engine::render::render_opts::RenderOpts;
use rayna_engine::render::renderer::Renderer;
use rayna_engine::scene::Scene;
use rayna_engine::skybox::SkyboxInstance;
use rayna_engine::texture::TextureInstance;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::ops::DerefMut;
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
    pub renderer: Renderer<ObjectInstance<MeshInstance, MaterialInstance<TextureInstance>>, SkyboxInstance>,
}

impl BgWorker {
    /// Starts the worker in a background thread, returning the thread handle
    pub fn start_bg_thread(self) -> std::io::Result<JoinHandle<()>> {
        std::thread::Builder::new()
            .name("BgWorker::thread".into())
            .spawn(move || self.thread_run())
    }

    /// Actually runs the thread
    /// This should be called inside [std::thread::spawn], it will block
    #[instrument(level = tracing::Level::DEBUG, skip(self), parent = None)]
    pub fn thread_run(self) {
        info!(target: BG_WORKER, "BgWorker thread start");
        profiler::renderer_profiler_init();

        let Self {
            msg_tx,
            msg_rx,
            render_tx,
            mut render_opts,
            mut scene,
            mut renderer,
        } = self;

        loop {
            profiler::renderer_profiler_lock().new_frame();

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
                profile_scope!("make_render");
                let render = renderer.render(&scene, &render_opts);

                // // TODO: REMOVE THIS IT'S TESTING ONLY
                // // save image
                // info!(target: "TESTING", "saved render to disk: {:#?}", image::DynamicImage::from(render.img.clone()).save_with_format("./render.exr", ImageFormat::OpenExr));

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

    /// Converts the image outputted by the renderer into an egui-appropriate one.
    /// Also converts from linear space to SRGB space
    fn convert_img(mut img: Image) -> ColorImage {
        profile_function!();

        // Got a rendered image
        // Post-process, and translate to an egui-appropriate one

        {
            profile_scope!("correct_gamma");
            const GAMMA: Channel = 2.2;
            const INV_GAMMA: Channel = 1.0 / GAMMA;

            // Gamma correction is per-channel, not per-pixel
            img.deref_mut().into_par_iter().for_each(|c| *c = c.powf(INV_GAMMA));
        }

        // Convert
        let img_as_rgba_u8: RgbaImage = {
            profile_scope!("convert_channels_u8");
            let mut buffer: RgbaImage = RgbaImage::new(img.width(), img.height());
            for (to, from) in buffer.pixels_mut().zip(img.pixels()) {
                to.0[0] = (from.0[0] * 255.0) as u8;
                to.0[1] = (from.0[1] * 255.0) as u8;
                to.0[2] = (from.0[2] * 255.0) as u8;
                to.0[3] = 255;
            }
            buffer
        };

        let img_as_egui = {
            profile_scope!("transmute_egui");

            let size = [img.width(), img.height()];

            // PERFORMANCE:
            // This is massively faster than calling
            // `ColorImage::from_rgba_unmultiplied(size, img_as_rgba.into_vec())`
            // It goes from ~7ms to ~1us
            // We can do this because we know alpha channel is always 1, so we can skip it

            // SAFETY:
            //  Color32 is defined as being a `[u8; 4]` internally anyway
            //  And we know that we have stored pixels `[r, g, b, a] : [u8; 4]`
            //  So we can safely transmute the vector, because they have the same
            //  internal representation and layout
            let (ptr, len, cap) = img_as_rgba_u8.into_vec().into_raw_parts();
            let px = unsafe { Vec::from_raw_parts(ptr, len / 4, cap / 4) };

            ColorImage { size, pixels: px }
        };

        img_as_egui
    }
}
