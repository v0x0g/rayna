use getset::CopyGetters;
use rand_core::RngCore;

use rayna_shared::def::types::{Number, Point2, Point3};

use crate::mesh::planar::Planar;
use crate::mesh::{Mesh, MeshInstance, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;

// region UV Wrap

/// Enum for different ways UV coordinates can be wrapped (or not) on a plane
#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum UvWrappingMode {
    /// Don't wrap UV coords, keep them unbounded
    None,
    /// Wrap the UV coordinates when they reach `1.0`
    ///
    /// Equivalent to `x % 1.0`
    #[default]
    Wrap,
    /// Mirror the UV coordinates when they reach `1.0`, repeating each interval
    ///
    /// Equivalent to `abs((x % 2.0) - 1.0)
    Mirror,
}

impl UvWrappingMode {
    /// Applies the wrapping mode to the UV coordinate, returning the new coordinate
    #[inline(always)]
    pub fn apply(self, uvs: Point2) -> Point2 {
        fn wrap(x: Number) -> Number { (x % 1.0).abs() }
        fn mirror(x: Number) -> Number { ((x % 2.0) - 1.0).abs() }

        match self {
            Self::None => uvs,
            Self::Wrap => Point2::new(wrap(uvs.x), wrap(uvs.y)),
            Self::Mirror => Point2::new(mirror(uvs.x), mirror(uvs.y)),
        }
    }

    /// Applies the wrapping mode to the UV coordinate, writing the modified coordinate into the reference
    #[inline(always)]
    pub fn apply_mut(self, uvs: &mut Point2) { *uvs = self.apply(*uvs); }
}

// endregion UV Wrap

#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct InfinitePlaneMesh {
    /// The plane that this mesh sits upon
    plane: Planar,
    uv_wrap: UvWrappingMode,
}

// region Constructors

impl InfinitePlaneMesh {
    pub fn new(plane: Planar, uv_wrap: UvWrappingMode) -> Self { Self { plane, uv_wrap } }
}

impl From<Planar> for InfinitePlaneMesh {
    fn from(plane: Planar) -> Self { Self::new(plane, UvWrappingMode::default()) }
}

// endregion Constructors

// region Mesh Impl

impl Mesh for InfinitePlaneMesh {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        let mut i = self.plane.intersect_bounded(ray, bounds)?;
        // Wrap uv's if required
        self.uv_wrap.apply_mut(&mut i.uv);
        Some(i)
    }
}

impl HasAabb for InfinitePlaneMesh {
    fn aabb(&self) -> Option<&Aabb> { None }
}

impl MeshProperties for InfinitePlaneMesh {
    fn centre(&self) -> Point3 { self.plane.p() }
}

// endregion Mesh Impl
