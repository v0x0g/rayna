use crate::mesh::dynamic::DynamicMesh;
use crate::mesh::{Mesh, MeshInstance, MeshProperties};
use crate::shared::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use derivative::Derivative;
use getset::Getters;
use rand::Rng;
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Point3};
use smallvec::SmallVec;

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""), Copy)]
pub struct HomogeneousVolumeBuilder<Obj: Mesh + Clone> {
    pub object: Obj,
    pub density: Number,
}

impl<Obj: Mesh + Clone> From<HomogeneousVolumeBuilder<Obj>> for HomogeneousVolumeMesh<Obj> {
    fn from(value: HomogeneousVolumeBuilder<Obj>) -> Self {
        Self {
            object: value.object,
            density: value.density,
            neg_inv_density: -1.0 / value.density,
        }
    }
}
impl<Obj: Mesh + Clone + 'static> From<HomogeneousVolumeBuilder<Obj>> for MeshInstance {
    fn from(value: HomogeneousVolumeBuilder<Obj>) -> Self {
        let HomogeneousVolumeBuilder { object, density } = value;
        // ObjectInstance uses HomogeneousVolumeObject<DynamicObject>, so cast the builder
        let dyn_builder = HomogeneousVolumeBuilder {
            density,
            object: DynamicMesh::from(object),
        };
        MeshInstance::HomogeneousVolumeMesh(HomogeneousVolumeMesh::from(dyn_builder))
    }
}

/// An mesh wrapper that treats the wrapped mesh as a constant-density volume
#[derive(Derivative, Getters)]
#[derivative(Debug(bound = ""), Clone(bound = ""), Copy)]
#[get = "pub"]
pub struct HomogeneousVolumeMesh<Obj: Mesh + Clone> {
    object: Obj,
    density: Number,
    neg_inv_density: Number,
}

impl<Obj: Mesh + Clone> Mesh for HomogeneousVolumeMesh<Obj> {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        // Find two samples on surface of volume
        // These should be as the ray enters and exits the mesh

        let entering = self.object.intersect(ray, bounds, rng)?;
        // Have to add a slight offset so we don't intersect with the same point twice
        let exiting = self
            .object
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

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>, rng: &mut dyn RngCore) {
        // TODO: iter with fold()?
        let mut ray = *ray;
        while let Some(i) = self.intersect(&ray, &Bounds::FULL, rng) {
            output.push(i);
            // ray.dir is arbitrary
            ray = Ray::new(i.pos_w, i.normal)
        }
    }
}

impl<Obj: Mesh + Clone> MeshProperties for HomogeneousVolumeMesh<Obj> {
    fn aabb(&self) -> Option<&Aabb> { self.object.aabb() }
    fn centre(&self) -> Point3 { self.object.centre() }
}
