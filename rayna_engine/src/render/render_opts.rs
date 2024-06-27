use crate::core::types::Number;
use nonzero::nonzero;
use serde::Serialize;
use std::num::NonZeroUsize;
use strum_macros::{Display, EnumIter, IntoStaticStr};
use valuable::Valuable;

#[derive(Copy, Clone, Debug, Valuable, Serialize)]
pub struct RenderOpts {
    /// The target width of the render (pixels)
    pub width: NonZeroUsize,
    /// The target height of the render (pixels)
    pub height: NonZeroUsize,
    /// A scalar to increase the number of samples taken for each pixel.
    /// Probably keep this at one and prefer accumulation instead.
    pub samples: NonZeroUsize,
    /// The way in which the render is visuaised. See [RenderMode]
    pub mode: RenderMode,
    /// How many times a ray can bounce
    pub ray_depth: usize,
    /// (Advanced) How many sub-rays each ray should split into, each time it bounces
    ///
    /// E.g. If this is `2`, we get `1 -> 2 -> 4 -> 8 -> 16 -> ...` rays at each bounce (assuming they all scatter)
    ///
    /// # Performance
    /// Note that this causes an exponential increase in the number of rays. It is advisable to keep this very low.
    /// This is mostly only effective in highly diffuse scenes.
    pub ray_branching: NonZeroUsize,
}

#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Valuable, Serialize, EnumIter, IntoStaticStr, Display,
)]
pub enum RenderMode {
    /// Used physically-based rendering, makes pretty images
    #[default]
    PBR,
    /// Visualise the normal of the mesh, going against the ray
    RayNormal,
    /// Visualise the normal of the mesh, going outwards
    OutwardNormal,
    /// Visualise the scatter direction of the material
    Scatter,
    /// Visualise whether the face is on the front or back face of the mesh
    FrontFace,
    /// Visualise how far away from the camera the intersection was
    Distance,
    /// Visualise the meshes' UV coordinates
    Uv,
    /// Visualise which side of the object was hit
    Side,
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
