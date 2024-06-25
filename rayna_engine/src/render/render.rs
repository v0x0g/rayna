use crate::render::render_opts::RenderOpts;
use std::time::Duration;

#[derive(Copy, Clone, Debug, Default)]
pub struct RenderStats {
    /// How long the render took
    pub duration: Duration,
    /// How many threads were used in rendering
    pub num_threads: usize,
    /// The render options that were used to make the render
    pub opts: RenderOpts,
    /// Number of frames that were accumulated so far
    pub accum_frames: usize,
}

#[derive(Clone, Debug)]
pub struct Render<T> {
    pub img: T,
    pub stats: RenderStats,
}
