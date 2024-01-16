use crate::mesh::dynamic::DynamicMesh;
use crate::mesh::{Mesh, MeshInstance, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use getset::Getters;
use rand::Rng;
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Point3};


#[derive(Debug, Copy, Clone)]
pub struct HomogeneousVolumeBuilder<M: Mesh> {
    /// The mesh that gives this volume it's shape
    pub mesh: M,
    /// How dense the volume is. Higher values give a "thicker" volume
    pub density: Number,
}

impl<M: Mesh> From<HomogeneousVolumeBuilder<M>> for HomogeneousVolumeMesh<M> {
    fn from(value: HomogeneousVolumeBuilder<M>) -> Self {
        Self {
            mesh: value.mesh,
            density: value.density,
            neg_inv_density: -1.0 / value.density,
        }
    }
}

// VolumeBuilder<T> => MeshInstance
impl<M: Mesh + 'static> From<HomogeneousVolumeBuilder<M>> for MeshInstance {
    fn from(value: HomogeneousVolumeBuilder<M>) -> Self {
        let HomogeneousVolumeBuilder { mesh: object, density } = value;
        // ObjectInstance uses HomogeneousVolumeObject<DynamicObject>, so cast the builder to dyn mesh
        let dyn_builder = HomogeneousVolumeBuilder {
            density,
            mesh: DynamicMesh::new(object),
        };
        MeshInstance::HomogeneousVolumeMesh(HomogeneousVolumeMesh::from(dyn_builder))
    }
}

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

impl<M: Mesh> Mesh for HomogeneousVolumeMesh<M> {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        // Find two samples on surface of volume
        // These should be as the ray enters and exits the mesh

        let entering = self.mesh.intersect(ray, bounds, rng)?;
        // Have to add a slight offset so we don't intersect with the same point twice
        let exiting = self
            .mesh
            .intersect(ray, &bounds.with_some_start(entering.dist + 1e-4), rng)?;

        if !bounds.contains(&entering.dist) || !bounds.contains(&exiting.dist) {
            return None;
        }

        // Distance between entry and exit of mesh along ray
        let dist_inside = exiting.dist - entering.dist;
        // Random distance at which we will hit
        let hit_dist = self.neg_inv_density * Number::ln(rng.gen());

        if hit_dist > dist_inside {
            return None;
        }

        let dist = entering.dist + hit_dist;
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
