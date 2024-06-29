use crate::ext::img_ext::ImageExt;
use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::targets::BG_WORKER;
use egui::ColorImage;
use puffin::{profile_function, profile_scope};
use rayna_engine::core::profiler;
use rayna_engine::material::MaterialInstance;
use rayna_engine::mesh::MeshInstance;
use rayna_engine::object::ObjectInstance;
use rayna_engine::render::render::Render;
use rayna_engine::render::renderer::Renderer;
use rayna_engine::skybox::SkyboxInstance;
use rayna_engine::texture::TextureInstance;
use std::thread::JoinHandle;
use std::time::Duration;
use tracing::{info, trace, warn};

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

                Render {
                    img: render.img.to_egui(),
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
}
