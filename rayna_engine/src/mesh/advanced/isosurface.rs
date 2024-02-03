use crate::core::types::{Number, Point2, Point3, Vector3};
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use derivative::Derivative;
use dyn_clone::DynClone;
use getset::{CopyGetters, Getters};
use rand_core::RngCore;

/// A mesh struct that is created by creating an isosurface from a given SDF
///
/// # Transforming
/// This mesh purposefully does not have any properties for transforming, so you must you a
/// [ObjectTransform].
#[derive(CopyGetters, Getters, Derivative, Clone)]
#[derivative(Debug)]
pub struct IsosurfaceMesh {
    #[derivative(Debug = "ignore")]
    sdf: Box<dyn SdfGeneratorFunction>,
}

pub trait SdfGeneratorFunction: Fn(Point3) -> Number + Send + Sync + DynClone {}
impl<T: Fn(Point3) -> Number + Send + Sync + Clone> SdfGeneratorFunction for T {}
dyn_clone::clone_trait_object!(SdfGeneratorFunction);

// region Constructors

impl IsosurfaceMesh {
    /// Creates a new mesh from the given isosurface, as defined by the **Signed-Distance Function** (**SDF**)
    ///
    /// # Arguments
    ///
    /// * `resolution`: How dense the resulting mesh should be.
    /// The resulting mesh has dimensions of a `N*N*N` grid, where `N = resolution`
    /// * `sdf`: The **SDF** that defines the surface for the mesh
    pub fn new<F: SdfGeneratorFunction + 'static>(sdf: F) -> Self { Self { sdf: Box::new(sdf) } }
}

// endregion Constructors

// region Mesh Impl

impl HasAabb for IsosurfaceMesh {
    fn aabb(&self) -> Option<&Aabb> { None }
}

impl MeshProperties for IsosurfaceMesh {
    fn centre(&self) -> Point3 { Point3::ZERO }
}

impl Mesh for IsosurfaceMesh {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, _rng: &mut dyn RngCore) -> Option<Intersection> {
        const MAX_ITERATIONS: usize = 100;
        const EPSILON: Number = 1e-5;

        // Start point at earliest pos on ray, or ray origin if unbounded
        let ro = interval.start.map(|t| ray.at(t)).unwrap_or(ray.pos());
        let dir = ray.dir();

        let mut point = ro;
        let mut dist = 0.0;
        let mut i = 0;
        loop {
            // Ray march towards surface
            let step = (self.sdf)(point);
            dist += step;
            point += dir * step;

            if step.abs() < EPSILON {
                return Some(Intersection {
                    pos_w: point,
                    pos_l: point,
                    uv: Point2::ZERO,
                    dist,
                    front_face: step.is_sign_positive(),
                    face: i,
                    normal: Vector3::X,
                    ray_normal: Vector3::X,
                });
            }

            if i > MAX_ITERATIONS {
                return None;
            }

            i += 1;
        }
    }
}

// endregion Mesh Impl
