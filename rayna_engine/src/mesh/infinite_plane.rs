use getset::CopyGetters;
use rand_core::RngCore;
use smallvec::SmallVec;

use rayna_shared::def::types::{Number, Point2, Point3};

use crate::mesh::planar::{Planar, PlanarBuilder};
use crate::mesh::{Object, ObjectInstance, ObjectProperties};
use crate::shared::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum UvWrappingMode {
    /// Don't wrap UV coords, keep them unbounded
    #[default]
    None,
    /// Wrap the UV coordinates when they reach `1.0`
    ///
    /// Equivalent to `x % 1.0`
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

#[derive(Copy, Clone, Debug)]
pub struct InfinitePlaneBuilder {
    pub plane: PlanarBuilder,
    pub uv_wrap: UvWrappingMode,
}

#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct InfinitePlaneObject {
    /// The plane that this mesh sits upon
    plane: Planar,
    uv_wrap: UvWrappingMode,
}

impl From<InfinitePlaneBuilder> for InfinitePlaneObject {
    fn from(builder: InfinitePlaneBuilder) -> Self {
        let plane = builder.plane.into();
        Self {
            plane,
            uv_wrap: builder.uv_wrap,
        }
    }
}

impl From<InfinitePlaneBuilder> for ObjectInstance {
    fn from(value: InfinitePlaneBuilder) -> Self { InfinitePlaneObject::from(value).into() }
}

impl Object for InfinitePlaneObject {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        let mut i = self.plane.intersect_bounded(ray, bounds)?;
        // Wrap uv's if required
        self.uv_wrap.apply_mut(&mut i.uv);
        Some(i)
    }

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>, rng: &mut dyn RngCore) {
        // Ignores infinite intersection case
        self.intersect(ray, &Bounds::FULL, rng).map(|i| output.push(i));
    }
}

impl ObjectProperties for InfinitePlaneObject {
    fn aabb(&self) -> Option<&Aabb> { None }
    fn centre(&self) -> Point3 { self.plane.p() }
}
