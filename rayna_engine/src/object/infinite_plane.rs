use getset::Getters;
use smallvec::SmallVec;

use rayna_shared::def::types::{Number, Point2, Point3, Vector3};

use crate::accel::aabb::Aabb;
use crate::material::MaterialInstance;
use crate::object::planar::Planar;
use crate::object::{Object, ObjectInstance, ObjectProperties};
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
        fn wrap(x: Number) -> Number { x % 1.0 }
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

#[derive(Clone, Debug)]
pub enum InfinitePlaneBuilder {
    /// Creates a [InfinitePlaneObject] from three points on the surface.
    ///
    /// See [super::parallelogram::ParallelogramBuilder::Points] for a visual demonstration
    Points {
        p: Point3,
        a: Point3,
        b: Point3,
        material: MaterialInstance,
        uv_wrap: UvWrappingMode,
    },
    /// Creates a parallelogram from the origin point `p`, and the two side vectors `u`, `v`
    ///
    /// See [super::parallelogram::ParallelogramBuilder::Vectors] for a visual demonstration
    Vectors {
        p: Point3,
        u: Vector3,
        v: Vector3,
        material: MaterialInstance,
        uv_wrap: UvWrappingMode,
    },
}

#[derive(Clone, Debug, Getters)]
#[get = "pub"]
pub struct InfinitePlaneObject {
    /// The plane that this object sits upon
    plane: Planar,
    uv_wrap: UvWrappingMode,
    material: MaterialInstance,
}

impl From<InfinitePlaneBuilder> for InfinitePlaneObject {
    fn from(p: InfinitePlaneBuilder) -> Self {
        match p {
            InfinitePlaneBuilder::Points {
                p,
                a,
                b,
                material,
                uv_wrap,
            } => {
                let plane = Planar::new_points(p, a, b);
                Self {
                    plane,
                    material,
                    uv_wrap,
                }
            }
            InfinitePlaneBuilder::Vectors {
                p,
                u,
                v,
                material,
                uv_wrap,
            } => {
                let plane = Planar::new(p, u, v);
                Self {
                    plane,
                    uv_wrap,
                    material,
                }
            }
        }
    }
}

impl From<InfinitePlaneBuilder> for ObjectInstance {
    fn from(value: InfinitePlaneBuilder) -> Self { InfinitePlaneObject::from(value).into() }
}

impl Object for InfinitePlaneObject {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        let mut i = self.plane.intersect_bounded(ray, bounds, &self.material)?;
        // Wrap uv's if required
        self.uv_wrap.apply_mut(&mut i.uv);
        Some(i)
    }

    //noinspection DuplicatedCode
    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>) {
        // Ignores infinite intersection case
        self.intersect(ray, &Bounds::FULL).map(|i| output.push(i));
    }
}

impl ObjectProperties for InfinitePlaneObject {
    fn aabb(&self) -> Option<&Aabb> { None }
    fn centre(&self) -> Point3 { self.plane.p() }
}
