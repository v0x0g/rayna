use crate::core::types::Number;
use nonzero::nonzero;
use serde::Serialize;
use std::num::NonZeroUsize;
use strum_macros::{Display, EnumIter, IntoStaticStr};
use valuable::Valuable;

#[derive(Copy, Clone, Debug, Valuable, Serialize)]
pub struct RenderOpts {
    /// The target dimensions of the render, stored as `[width, height]`
    pub width: NonZeroUsize,
    pub height: NonZeroUsize,
    /// A scalar to increase the number of samples taken for each pixel.
    pub samples: NonZeroUsize,
    pub mode: RenderMode,
    pub ray_depth: usize,
    pub ray_branching: NonZeroUsize,
}

#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Valuable, Serialize, EnumIter, IntoStaticStr, Display,
)]
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
    /// Returns the dimensions of the render (width and height) as a [usize] slice
    pub fn dims(&self) -> [usize; 2] { [self.width.get(), self.height.get()] }

    pub fn aspect_ratio(&self) -> Number { self.width.get() as Number / self.height.get() as Number }
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            width: nonzero!(740_usize),
            height: nonzero!(480_usize),
            samples: nonzero!(1_usize),
            mode: Default::default(),
            ray_depth: 5,
            ray_branching: nonzero!(1_usize),
        }
    }
}
