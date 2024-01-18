use crate::mesh::dynamic::DynamicMesh;
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use getset::Getters;
use rand::Rng;
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Point3};

/// An mesh wrapper that treats the wrapped mesh as a constant-density volume
///
/// The volume has the same shape as the wrapped `mesh`, and a constant density at all points in the volume
#[derive(Copy, Clone, Debug, Getters)]
#[get = "pub"]
pub struct HomogeneousVolumeMesh<M: Mesh> {
    mesh: M,
    density: Number,
    neg_inv_density: Number,
}

// region Constructors

impl<M: Mesh> HomogeneousVolumeMesh<M> {
    pub fn new(mesh: M, density: Number) -> Self {
        let neg_inv_density = -1.0 / density;
        Self {
            mesh,
            density,
            neg_inv_density,
        }
    }
}

impl HomogeneousVolumeMesh<DynamicMesh> {
    /// Use this method as an alternative to [Self::new()], since it wraps the mesh in a [DynamicMesh] wrapper.
    /// Can be used directly as a [MeshInstance]
    pub fn new_dyn<M: Mesh + 'static>(mesh: M, density: Number) -> Self { Self::new(DynamicMesh::new(mesh), density) }
}

// endregion Constructors

// region Mesh Impl

impl<M: Mesh> Mesh for HomogeneousVolumeMesh<M> {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        // Find two samples on surface of volume
        // These should be as the ray enters and exits the mesh

        // NOTE: We should be using the `bounds` parameter here, however that won't work for rays inside meshes,
        // where the mesh is convex (many primitives are) - the first intersection will be 'behind' the ray,
        // and so we will only get *one* forward intersection (entering), which means we don't an exiting intersection.
        // To solve this, we check for entering intersection without bounds, so that we can still check if an intersection
        // exists at all along the ray. Then, we clamp that distance value to our bounds, so we still get the right value
        let entering_dist = {
            let d = self.mesh.intersect(ray, &Bounds::FULL, rng)?.dist;
            // If we have start bound, move intersection along so it happened there at the earliest
            if let Some(start) = bounds.start {
                d.max(start)
            } else {
                d
            }
        };
        // Have to add a slight offset so we don't intersect with the same point twice
        let exiting_dist = {
            let d = self
                .mesh
                .intersect(ray, &Bounds::from(entering_dist + 0.001..), rng)?
                .dist;

            if let Some(end) = bounds.end {
                d.min(end)
            } else {
                d
            }
        };

        // Distance between entry and exit of mesh along ray
        let dist_inside = exiting_dist - entering_dist;

        // if dist_inside < 0. { unreachable!() }

        // Random distance at which we will hit
        let hit_dist = self.neg_inv_density * Number::ln(rng.gen());

        // NOTE: We don't do normal bounds checks on intersections here, due to concavity issues given above.
        // Also, even if `exiting_dist` is outside of the range, the value `hit_dist` might be inside
        // And `hit_dist` is the one we actually use, so check that instead
        // We don't need to check `if !bounds.contains(&dist)`, it's guaranteed to be inside `bounds`
        if hit_dist > dist_inside {
            return None;
        }

        let dist = entering_dist + hit_dist;

        let pos_w = ray.at(dist);
        let pos_l = pos_w;

        Some(Intersection {
            dist,
            pos_w,
            pos_l,

            // The following are all completely arbitrary
            normal: rng::vector_on_unit_sphere(rng),
            ray_normal: rng::vector_on_unit_sphere(rng),
            uv: rng::vector_in_unit_square_01(rng).to_point(),
            face: 0,
            front_face: true,
        })
    }
}

impl<M: Mesh> HasAabb for HomogeneousVolumeMesh<M> {
    fn aabb(&self) -> Option<&Aabb> { self.mesh.aabb() }
}

impl<M: Mesh> MeshProperties for HomogeneousVolumeMesh<M> {
    fn centre(&self) -> Point3 { self.mesh.centre() }
}

// endregion Mesh Impl
