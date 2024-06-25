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
use rayna_engine::render::renderer::Renderer;
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
    /// Sender for messages from the worker, back to the UI
    pub msg_tx: flume::Sender<MessageToUi>,
    /// Receiver for messages from the UI, to the worker
    pub msg_rx: flume::Receiver<MessageToWorker>,
    pub render_tx: flume::Sender<Render<ColorImage>>,
    pub renderer:
        Renderer<ObjectInstance<MeshInstance, MaterialInstance<TextureInstance>>, SkyboxInstance, rand::rngs::SmallRng>,
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
        profiler::renderer::init_thread();

        let Self {
            msg_tx,
            msg_rx,
            render_tx,
            mut renderer,
        } = self;

        loop {
            profiler::renderer::lock().new_frame();

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
                        MessageToWorker::SetRenderOpts(o) => {
                            trace!(target: BG_WORKER, ?o, "got render opts from ui");
                            renderer.set_options(o);
                        }
                        MessageToWorker::SetScene(s) => {
                            trace!(target: BG_WORKER, ?s, "got scene from ui");
                            renderer.set_scene(s);
                        }
                        MessageToWorker::SetCamera(c) => {
                            trace!(target: BG_WORKER, ?c, "got scene from ui");
                            renderer.set_camera(c);
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
                let render = renderer.render();

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
    fn convert_img(mut src: Image) -> ColorImage {
        profile_function!();

        // Got a rendered image
        // Post-process, and translate to an egui-appropriate one

        {
            profile_scope!("correct_gamma");
            const GAMMA: Channel = 2.2;
            const INV_GAMMA: Channel = 1.0 / GAMMA;

            // Gamma correction is per-channel, not per-pixel
            src.deref_mut().into_par_iter().for_each(|c| *c = c.powf(INV_GAMMA));
        }
        // TODO: Pool the images?
        let mut output = {
            profile_scope!("alloc_output");
            ColorImage {
                size: [src.width(), src.height()],
                // I hope the compiler optimizes this
                pixels: vec![Color32::default(); src.len()],
            }
        };

        // Convert each pixel into array of u8 channels and write to output
        {
            profile_scope!("convert_channels_u8");
            src.indexed_iter().for_each(|((x, y), col)| {
                let [r, g, b] = col.0.map(|c| (c * 255.0) as u8);
                output[(x, y)] = Color32::from_rgb(r, g, b)
            });
        };

        output
    }
}
