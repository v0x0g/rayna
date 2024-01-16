use nonzero::nonzero;
use num_traits::cast::ToPrimitive;
use rayna_shared::def::types::Number;
use serde::Serialize;
use std::num::NonZeroUsize;
use strum_macros::{EnumIter, IntoStaticStr};
use valuable::Valuable;

#[derive(Copy, Clone, Debug, Valuable, Serialize)]
pub struct RenderOpts {
    /// The target dimensions of the render, stored as `[width, height]`
    pub width: NonZeroUsize,
    pub height: NonZeroUsize,
    /// How many samples to take for each pixel (MSAA)
    pub msaa: NonZeroUsize,
    pub mode: RenderMode,
    pub bounces: usize,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Valuable, Serialize, EnumIter, IntoStaticStr)]
pub enum RenderMode {
    #[default]
    PBR,
    RayNormal,
    OutwardNormal,
    Scatter,
    Distance,
    Uv,
    Face,
}

impl RenderOpts {
    /// Returns the dimensions of the render (width and height) as a [u32] slice
    pub fn dims_u32_slice(&self) -> [u32; 2] {
        [self.width, self.height]
            .map(|x| x.get().to_u32())
            .map(|d| d.expect("image dims failed to fit inside u32"))
    }

    pub fn aspect_ratio(&self) -> Number { self.width.get() as Number / self.height.get() as Number }
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            width: nonzero!(740_usize),
            height: nonzero!(480_usize),
            msaa: nonzero!(1_usize),
            mode: Default::default(),
            bounces: 20,
        }
    }
}
