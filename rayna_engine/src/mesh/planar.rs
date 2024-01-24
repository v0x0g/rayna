//! This module is not an mesh module per-se, but a helper module that provides abstractions for
//! planar types (such as planes, quads, triangles, etc)
//!
//! You should store an instance of [Planar] inside your mesh struct, and then simply validate the UV coordinates
//! of the planar intersection for whichever shape your dreams do so desire...
//!
//! Most planar types ([super::parallelogram], [super::triangle], [super::infinite_plane]) can't be instantiated directly,
//! but can be easily converted via the [From<Planar>] conversion.

use crate::core::types::{Number, Point2, Point3, Vector3};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use getset::CopyGetters;
use num_traits::Zero;

/// The recommended amount of padding around AABB's for planar objects
pub const AABB_PADDING: Number = 1e-6;

/// A helper struct that is used in planar objects (objects that exist in a subsection of a 2D plane
///
/// Use this for calculating the ray-plane intersection, instead of reimplementing for each type.
/// Then, you can restrict by validating the UV coordinates returned by the intersection
#[derive(Copy, Clone, Debug, CopyGetters)]
#[get_copy = "pub"]
pub struct Planar {
    p: Point3,
    /// The vector for the `U` direction, typically the 'right' direction
    u: Vector3,
    /// The vector for the `V` direction, typically the 'up' direction
    v: Vector3,
    /// The normal vector for the plane, perpendicular to [u] and [v], and normalised
    n: Vector3,
    /// Part of the plane equation
    d: Number,
    /// Precalculated vector `n / dot(n, cross(u,v))` (using un-normalised `n`)
    w: Vector3,
}

// region Constructors

impl Planar {
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
    /// TEXT ART CREDITS:
    ///
    /// Author: Textart.sh
    ///
    /// URL: https://textart.sh/topic/parallelogram
    pub fn new(p: impl Into<Point3>, u: impl Into<Vector3>, v: impl Into<Vector3>) -> Self {
        let (p, u, v) = (p.into(), u.into(), v.into());

        let n_raw = Vector3::cross(u, v);
        let n = n_raw
            .try_normalize()
            .expect("couldn't normalise plane normal: cross(u, v) == 0");
        let d = n.dot(p.to_vector());
        // NOTE: using non-normalised normal here
        let w = n_raw / n_raw.length_squared();
        Self { p, u, v, n, d, w }
    }

    /// Creates a [Planar] mesh from three points on the surface.
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
    ///  P ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ B                                                  
    /// ```
    ///
    /// TEXT ART CREDITS:
    ///
    /// Author: Textart.sh
    ///
    /// URL: https://textart.sh/topic/parallelogram
    pub fn new_points(p: impl Into<Point3>, a: impl Into<Point3>, b: impl Into<Point3>) -> Self {
        let (p, a, b) = (p.into(), a.into(), b.into());
        Self::new(p, a - p, b - p)
    }
}

/// Create from three point array
impl From<[Point3; 3]> for Planar {
    fn from([p, a, b]: [Point3; 3]) -> Self { Self::new_points(p, a, b) }
}
/// Create from three point tuple
impl From<(Point3, Point3, Point3)> for Planar {
    fn from((p, a, b): (Point3, Point3, Point3)) -> Self { Self::new_points(p, a, b) }
}

// endregion

// region Intersection

impl Planar {
    /// Does a full ray-plane intersection check, returning the intersection if possible. If an intersection is not found,
    /// it means that the ray is perfectly parallel to the plane, or outside the given bounds.
    ///
    /// # Arguments
    ///
    /// * `ray`: The ray to intersect with
    /// * `bounds`: Bounds to restrict the range of valid distances
    /// * `material`: Material to be used for the [Intersection] in the case of an intersection
    /// * `validate_coords`: Callable to be used to validate whether the given point on the surface is considered valid.
    /// Arguments are `validate(u, v) -> point_is_valid`. Note that `u, v` will be with respect to the [Planar.u] and [Planar.v] values,
    /// so if creating a plane from three points, `u, v` will be equal to one *at those points*, as opposed to one unit in the direction of those points,
    /// meaning scaling those points will "enlarge" the resulting shape
    #[inline(always)]
    pub fn intersect_bounded(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        let denominator = Vector3::dot(self.n, ray.dir());

        // Ray is parallel to plane
        if denominator.is_zero() {
            return None;
        }

        let t = (self.d - Vector3::dot(self.n, ray.pos().to_vector())) / denominator;

        if !bounds.contains(&t) {
            return None;
        }

        let pos_w = ray.at(t);
        let pos_l = pos_w - self.p.to_vector();

        // We would normally project so the point is `P = Q + α*u + β*v`
        // But since the vectors `u, v` don't have to be orthogonal, have to account for that too
        // Because `P = pos`, `Q = pos_l`, we need to use local pos for this
        let q = pos_l.to_vector();
        let alpha = Vector3::dot(self.w, Vector3::cross(q, self.v));
        let beta = Vector3::dot(self.w, Vector3::cross(self.u, q));

        Some(Intersection {
            pos_w,
            pos_l,
            dist: t,
            normal: self.n,
            // Positive => ray and normal same dir => must be behind plane => backface
            front_face: denominator.is_sign_negative(),
            ray_normal: -self.n * denominator.signum(),
            uv: Point2::new(alpha, beta),
            face: 0,
        })
    }
}
// endregion
