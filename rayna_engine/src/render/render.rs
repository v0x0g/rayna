use crate::def::types::ImgBuf;
use derivative::Derivative;
use std::time::Duration;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct RenderStats {
    /// How long the render took
    pub duration: Duration,
    /// How many pixels were rendered
    pub num_px: usize,
    /// How many threads were used in rendering
    pub num_threads: usize,
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct Render {
    #[derivative(Debug = "ignore")]
    pub img: ImgBuf,
    pub stats: RenderStats,
}
