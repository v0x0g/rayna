use crate::core::types::Number;
use nonzero::nonzero;
use std::num::NonZeroUsize;
use strum_macros::{Display, EnumIter, IntoStaticStr};
use valuable::Valuable;

#[derive(Copy, Clone, Debug, Valuable)]
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
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Valuable, EnumIter, IntoStaticStr, Display)]
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
    pub fn dims(&self) -> [usize; 2] {
        [self.width.get(), self.height.get()]
    }

    pub fn aspect_ratio(&self) -> Number {
        self.width.get() as Number / self.height.get() as Number
    }
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            width: nonzero!(740_usize),
            height: nonzero!(480_usize),
            samples: nonzero!(1_usize),
            mode: Default::default(),
            ray_depth: 5,
        }
    }
}
