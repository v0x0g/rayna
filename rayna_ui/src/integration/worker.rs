use crate::integration::message::{MessageToUi, MessageToWorker};
use crate::targets::*;
use rayna_engine::core::profiler;
use rayna_engine::render::renderer::Renderer;
use tracing::{info, trace, warn};

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

        loop {
            profiler::renderer::lock().new_frame();

            puffin::profile_function!(); // place here not at the start since we are looping

            if msg_rx.is_disconnected() {
                warn!(target: BG_WORKER, "all senders disconnected from channel");
                break;
            }

            // Have two conditions: (empty) or (disconnected)
            // Checked if disconnected above and skip if empty, so just check Ok() here
            {
                puffin::profile_scope!("receive_messages");
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

            let render_result = {
                puffin::profile_scope!("make_render");
                renderer.render()
            };

            {
                puffin::profile_scope!("send_frame");

                if let Err(_) = msg_tx.send(MessageToUi::RenderComplete(render_result)) {
                    warn!(target: BG_WORKER, "failed to send rendered frame to UI")
                }
            }
        }

        info!(target: BG_WORKER, "BgWorker thread exit");
    }
}
