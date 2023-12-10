use nonzero::nonzero;
use serde::Serialize;
use std::num::NonZeroUsize;
use valuable::Valuable;

#[derive(Copy, Clone, Debug, Valuable, Serialize)]
pub struct RenderOpts {
    /// The target dimensions of the render, stored as `[width, height]`
    pub width: NonZeroUsize,
    pub height: NonZeroUsize,
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            width: nonzero!(1_usize),
            height: nonzero!(1_usize),
        }
    }
}
