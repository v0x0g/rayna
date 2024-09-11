use crate::ext::img_ext::ImageExt;
use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::targets::*;
use rayna_engine::core::profiler;
use rayna_engine::render::render::Render;
use rayna_engine::render::renderer::Renderer;
use tracing::*;

pub(super) struct BgWorker {
    /// Sender for messages from the worker, back to the UI
    pub msg_tx: flume::Sender<MessageToUi>,
    /// Receiver for messages from the UI, to the worker
    pub msg_rx: flume::Receiver<MessageToWorker>,
    pub renderer: Renderer<rand::rngs::SmallRng>,
}

impl BgWorker {
    /// Starts the worker in a background thread, returning the thread handle
    pub fn start_bg_thread(self) -> std::io::Result<std::thread::JoinHandle<()>> {
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
            mut renderer,
        } = self;

        'run: loop {
            profiler::renderer::lock().new_frame();

            puffin::profile_function!(); // place here not at the start since we are looping

            {
                puffin::profile_scope!("receive_messages");
                'recv: loop {
                    match msg_rx.try_recv() {
                        Ok(msg) => match msg {
                            MessageToWorker::SetRenderOpts(o) => {
                                debug!(target: BG_WORKER, ?o, "got render opts from ui");
                                renderer.set_options(o);
                            }
                            MessageToWorker::SetScene(s) => {
                                debug!(target: BG_WORKER, ?s, "got scene from ui");
                                renderer.set_scene(s);
                            }
                            MessageToWorker::SetCamera(c) => {
                                debug!(target: BG_WORKER, ?c, "got scene from ui");
                                renderer.set_camera(c);
                            }
                        },
                        Err(flume::TryRecvError::Empty) => {
                            trace!(target: BG_WORKER, "no messages from ui");
                            break 'recv;
                        }
                        Err(flume::TryRecvError::Disconnected) => {
                            error!(target:BG_WORKER, "failed to receive messages from ui: all senders dropped");
                            error!(target: BG_WORKER, "worker will now exit");
                            break 'run;
                        }
                    }
                }
            }

            {
                puffin::profile_scope!("waiting_channel_empty");
                // UI hasn't received the last message we sent
                if !msg_tx.is_empty() {
                    trace!(target: BG_WORKER, "channel not empty, waiting");
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    continue;
                } else {
                    trace!(target: BG_WORKER, "channel empty, sending new image");
                }
            }

            let render = {
                puffin::profile_scope!("render");
                renderer.render()
            };
            let render = {
                use crate::ext::img_ext::ImageExt as _;

                puffin::profile_scope!("convert");

                Render {
                    stats: render.stats,
                    img: (render.img.clone(), render.img.to_egui()),
                }
            };

            {
                puffin::profile_scope!("send_frame");

                // If an error is received, it means all receivers are dropped
                // meaning the main thread must have exited
                if let Err(_) = msg_tx.send(MessageToUi::RenderComplete(render)) {
                    error!(target: BG_WORKER, "failed to send render to ui: all receivers dropped");
                    error!(target: BG_WORKER, "worked will now exit");
                    break 'run;
                }
            }
        }

        info!(target: BG_WORKER, "BgWorker thread exit");
    }
}
