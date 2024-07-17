//! This module is not an mesh module per-se, but a helper module that provides abstractions for
//! planar types (such as planes, quads, triangles, etc)
//!
//! You should store an instance of [`Plane`] inside your mesh struct, and then simply validate the UV coordinates
//! of the planar intersection for whichever shape your dreams do so desire...
//!
//! Most planar types ([`self::parallelogram::ParallelogramMesh`], [`self::infinite_plane::InfinitePlaneMesh`]) can't be instantiated directly,
//! but can be easily converted via the [`From<Plane>`] conversion.

use crate::core::types::{Number, Point2, Point3, Vector3};
use crate::mesh::Mesh;
use crate::scene::Scene;
use crate::shared::aabb::{Aabb, Bounded};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use getset::CopyGetters;
use num_traits::Zero;
use rand_core::RngCore;
// region Structs

/// The recommended amount of padding around AABBs for planar objects
///
/// Because planes are infinitely thin, we need to add padding to ensure they have at least some volume.
/// Otherwise, there is a chance that the [`crate::shared::aabb::Aabb`] will always be missed because it has zero size.
pub const PLANAR_AABB_PADDING: Number = 1e-6;

/// A helper struct that is used in planar objects (objects that exist in a subsection of a 2D plane)
///
/// Use this for calculating the ray-plane intersection, instead of reimplementing for each type.
/// Then, you can restrict by validating the UV coordinates returned by the intersection
#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct Plane {
    p: Point3,
    /// The vector for the `U` direction, typically the 'right' direction
    u: Vector3,
    /// The vector for the `V` direction, typically the 'up' direction
    v: Vector3,
    /// The normal vector for the plane, perpendicular to `u` and `v`, and normalised
    n: Vector3,
    /// Part of the plane equation
    d: Number,
    /// Precalculated vector `n / dot(n, cross(u,v))` (using un-normalised `n`)
    w: Vector3,
}

#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct ParallelogramMesh {
    /// The plane that this mesh sits upon
    plane: Plane,
    aabb: Aabb,
}

#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct InfinitePlaneMesh {
    /// The plane that this mesh sits upon
    plane: Plane,
    uv_wrap: UvWrappingMode,
}

// endregion Structs

// region UV Wrap

/// Enum for different ways UV coordinates can be wrapped (or not) on a plane
#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum UvWrappingMode {
    // TODO: Remove `None`, add ones like clamp border, clamp edge
    /// Wrap the UV coordinates when they reach `1.0`
    ///
    /// Equivalent to `x % 1.0`
    #[default]
    Wrap,
    /// Mirror the UV coordinates when they reach `1.0`, repeating each interval
    ///
    /// Equivalent to `abs((x % 2.0) - 1.0)`
    Mirror,
    /// If either of the UV coordinates goes out of the range `0..=1`, sets both components to zero
    ClampZero,
    /// Clamps UV coordinates to the range `0..=1`
    Clamp,
}

impl UvWrappingMode {
    /// Applies the wrapping mode to the UV coordinate, returning the new coordinate
    #[inline(always)]
    pub fn apply(self, uvs: Point2) -> Point2 {
        fn wrap(x: Number) -> Number { x.rem_euclid(1.0) }
        fn mirror(x: Number) -> Number { ((x % 2.0) - 1.0).abs() }
        fn clamp(x: Number) -> Number { x.clamp(0., 1.) }

        match self {
            Self::Wrap => Point2::new(wrap(uvs.x), wrap(uvs.y)),
            Self::Mirror => Point2::new(mirror(uvs.x), mirror(uvs.y)),
            Self::Clamp => Point2::new(clamp(uvs.x), clamp(uvs.y)),
            Self::ClampZero => {
                if !(0. <= uvs.x && uvs.x <= 1. && 0. <= uvs.y && uvs.y <= 1.0) {
                    Point2::ZERO
                } else {
                    uvs
                }
            }
        }
    }

    /// Applies the wrapping mode to the UV coordinate, writing the modified coordinate into the reference
    #[inline(always)]
    pub fn apply_mut(self, uvs: &mut Point2) { *uvs = self.apply(*uvs); }
}

// endregion UV Wrap

// region Constructors

impl Plane {
    /// Creates a plane from the origin point `p`, and the two side vectors `u`, `v`
    ///
    /// For a 2D plane in the `XY` plane, the point layout would be:
    ///
    /// ```text
    ///              ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒                                    
    ///              ▓▓                               ▓▓                                    
    ///            ░░░░                             ░░░░                                    
    ///            ██                               ▓▓                                      
    ///            ▒▒                               ░░                                      
    ///          ▒▒                               ▓▓                                        
    ///          ▓▓  ^                            ▒▒                                        
    ///        ░░░░  |                          ▒▒                                          
    ///        ██    V                          ▓▓                                          
    ///        ▒▒    ^                        ░░                                            
    ///      ▒▒      |                        ▓▓                                            
    ///      ██                               ▒▒                                            
    ///    ▒▒░░                             ▒▒                                              
    ///    ▓▓                               ▓▓                                              
    ///  ░░░░                             ░░░░                                              
    ///  ██                               ▓▓                                                
    ///  ▒▒                             ░░▒▒                                                
    ///  P ▓▓▓▓▓▓▓▓▓▓▓ -> U -> ▓▓▓▓▓▓▓▓▓▓▓                                                  
    /// ```
    ///
    /// # Credits
    ///
    /// Author: Textart.sh
    ///
    /// URL: <https://textart.sh/topic/parallelogram>
    pub fn new(p: impl Into<Point3>, u: impl Into<Vector3>, v: impl Into<Vector3>) -> Self {
        let (p, u, v) = (p.into(), u.into(), v.into());

        let n_raw = Vector3::cross(u, v);
        let n = n_raw
            .try_normalize()
            .expect("couldn't normalise plane normal: cross(u, v) == 0");
        let d = -Vector3::dot(n, p.to_vector());
        // NOTE: using non-normalised normal here
        let w = n_raw / n_raw.length_squared();
        Self { p, u, v, n, d, w }
    }

    pub fn new_centred(centre: impl Into<Point3>, u: impl Into<Vector3>, v: impl Into<Vector3>) -> Self {
        let (centre, u, v) = (centre.into(), u.into(), v.into());

        Self::new(centre - u - v, u * 2., v * 2.)
    }

    /// Creates a [Plane] mesh from three points on the surface.
    ///
    /// For a 2D plane in the `XY` plane, the point layout would be:
    ///
    /// ```text
    ///              A ▓▓██▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██▒▒                                    
    ///              ▓▓                               ▓▓                                    
    ///            ░░░░                             ░░░░                                    
    ///            ██                               ▓▓                                      
    ///            ▒▒                               ░░                                      
    ///          ▒▒                               ▓▓                                        
    ///          ▓▓                               ▒▒                                        
    ///        ░░░░                             ▒▒                                          
    ///        ██                               ▓▓                                          
    ///        ▒▒                             ░░                                            
    ///      ▒▒                               ▓▓                                            
    ///      ██                               ▒▒                                            
    ///    ▒▒░░                             ▒▒                                              
    ///    ▓▓                               ▓▓                                              
    ///  ░░░░                             ░░░░                                              
    ///  ██                               ▓▓                                                
    ///  ▒▒                             ░░▒▒                                                
    ///  B ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ C                                                  
    /// ```
    ///
    /// # Credits
    ///
    /// Author: Textart.sh
    ///
    /// URL: <https://textart.sh/topic/parallelogram>
    pub fn new_points(a: impl Into<Point3>, b: impl Into<Point3>, c: impl Into<Point3>) -> Self {
        let (a, b, c) = (a.into(), b.into(), c.into());
        Self::new(b, a - b, c - b)
    }
}

/// Create from three point array
impl<P: Into<Point3>> From<[P; 3]> for Plane {
    fn from([p, a, b]: [P; 3]) -> Self { Self::new_points(p, a, b) }
}
/// Create from three point tuple
impl<P: Into<Point3>, A: Into<Point3>, B: Into<Point3>> From<(P, A, B)> for Plane {
    fn from((p, a, b): (P, A, B)) -> Self { Self::new_points(p, a, b) }
}

impl ParallelogramMesh {
    pub fn new(plane: impl Into<Plane>) -> Self {
        let plane = plane.into();
        let (p, a, b, ab) = (
            plane.p(),
            plane.p() + plane.u(),
            plane.p() + plane.v(),
            plane.p() + plane.u() + plane.v(),
        );
        let aabb = Aabb::encompass_points([p, a, b, ab]).with_min_padding(PLANAR_AABB_PADDING);

        Self { plane, aabb }
    }
}

impl InfinitePlaneMesh {
    pub fn new(plane: impl Into<Plane>, uv_wrap: UvWrappingMode) -> Self {
        Self {
            plane: plane.into(),
            uv_wrap,
        }
    }
}

impl<T: Into<Plane>> From<T> for ParallelogramMesh {
    fn from(plane: T) -> Self { Self::new(plane) }
}

impl<T: Into<Plane>> From<T> for InfinitePlaneMesh {
    fn from(plane: T) -> Self { Self::new(plane, UvWrappingMode::default()) }
}

// endregion

// region Mesh Impl

impl Plane {
    /// Does a full ray-plane intersection check, returning the intersection if possible. If an intersection is not found,
    /// it means that the ray is perfectly parallel to the plane, or outside the given interval.
    ///
    /// # Arguments
    ///
    /// * `ray`: The ray to intersect with
    /// * `interval`: interval to restrict the range of valid distances
    /// * `material`: Material to be used for the [Intersection] in the case of an intersection
    /// * `validate_coords`: Callable to be used to validate whether the given point on the surface is considered valid.
    /// Arguments are `validate(u, v) -> point_is_valid`. Note that `u, v` will be with respect to the [Planar.u] and [Planar.v] values,
    /// so if creating a plane from three points, `u, v` will be equal to one *at those points*, as opposed to one unit in the direction of those points,
    /// meaning scaling those points will "enlarge" the resulting shape
    #[inline(always)]
    pub fn intersect_bounded(&self, ray: &Ray, interval: &Interval<Number>) -> Option<Intersection> {
        let denominator = Vector3::dot(self.n, ray.dir());

        // Ray is parallel to plane
        if denominator.is_zero() {
            return None;
        }

        let t = -(Vector3::dot(self.n, ray.pos().to_vector()) + self.d) / denominator;

        if !interval.contains(&t) {
            return None;
        }

        let pos_w = ray.at(t);
        let pos_l = pos_w - self.p;

        // We would normally project so the point is `P = P_local + α*u + β*v`
        // But since the vectors `u, v` don't have to be orthogonal, have to account for that too
        let alpha = Vector3::dot(self.w, Vector3::cross(pos_l, self.v));
        let beta = Vector3::dot(self.w, Vector3::cross(self.u, pos_l));

        Some(Intersection {
            pos_w,
            pos_l: pos_l.to_point(),
            dist: t,
            normal: self.n,
            // Positive => ray and normal same dir => must be behind plane => backface
            front_face: denominator.is_sign_negative(),
            ray_normal: -self.n * denominator.signum(),
            uv: Point2::new(alpha, beta),
            side: 0,
        })
    }
}

impl Mesh for InfinitePlaneMesh {
    fn intersect(
        &self,
        _scene: &Scene,
        ray: &Ray,
        interval: &Interval<Number>,
        _rng: &mut dyn RngCore,
    ) -> Option<Intersection> {
        let mut i = self.plane.intersect_bounded(ray, interval)?;
        // Wrap uv's if required
        self.uv_wrap.apply_mut(&mut i.uv);
        Some(i)
    }
}

impl Mesh for ParallelogramMesh {
    fn intersect(
        &self,
        _scene: &Scene,
        ray: &Ray,
        interval: &Interval<Number>,
        _rng: &mut dyn RngCore,
    ) -> Option<Intersection> {
        let i = self.plane.intersect_bounded(ray, interval)?;
        // Check in interval for our segment of the plane: `uv in [0, 1]`
        if (i.uv.cmple(Point2::ONE) & i.uv.cmpge(Point2::ZERO)).all() {
            Some(i)
        } else {
            None
        }
    }
}

impl Bounded for InfinitePlaneMesh {
    fn aabb(&self) -> Aabb { Aabb::INFINITE }
}

impl Bounded for ParallelogramMesh {
    fn aabb(&self) -> Aabb { self.aabb }
}

// endregion Mesh Impl
