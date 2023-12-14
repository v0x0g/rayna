use rayna_shared::::types::Num;
use nonzero::nonzero;
use num_traits::cast::ToPrimitive;
use serde::Serialize;
use std::num::NonZeroUsize;
use valuable::Valuable;

#[derive(Copy, Clone, Debug, Valuable, Serialize)]
pub struct RenderOpts {
    /// The target dimensions of the render, stored as `[width, height]`
    pub width: NonZeroUsize,
    pub height: NonZeroUsize,
}

impl RenderOpts {
    /// Returns the dimensions of the render (width and height) as a [u32] slice
    pub fn dims_u32_slice(&self) -> [u32; 2] {
        [self.width, self.height]
            .map(|x| x.get().to_u32())
            .map(|d| d.expect("image dims failed to fit inside u32"))
    }

    pub fn aspect_ratio(&self) -> Num {
        self.width.get() as Num / self.height.get() as Num
    }
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            width: nonzero!(800_usize),
            height: nonzero!(400_usize),
        }
    }
}
