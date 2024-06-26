use crate::core::types::Number;
use crate::material::Material;
use crate::mesh::Mesh as MeshTrait;
use crate::object::transform::ObjectTransform;
use crate::object::Object;
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::{FullIntersection, Intersection};
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use crate::shared::rng;
use getset::{CopyGetters, Getters};
use rand::Rng;
use rand_core::RngCore;

/// An mesh wrapper that treats the wrapped mesh as a constant-density volume
///
/// The volume has the same shape as the wrapped `mesh`, and a constant density at all points in the volume
/// You are strongly recommended to use an instance of [`crate::material::isotropic::IsotropicMaterial`]
#[derive(Getters, CopyGetters, Clone, Debug)]
pub struct VolumetricObject<Mesh: MeshTrait, Mat: Material> {
    #[get = "pub"]
    mesh: Mesh,
    #[get = "pub"]
    material: Mat,
    #[get = "pub"]
    transform: ObjectTransform,
    #[get_copy = "pub"]
    density: Number,
    #[get_copy = "pub"]
    neg_inv_density: Number,
    aabb: Option<Aabb>,
}

// region Constructors

impl<Mesh, Mat> VolumetricObject<Mesh, Mat>
where
    Mesh: MeshTrait,
    Mat: Material,
{
    /// See [super::simple::SimpleObject::new()]
    pub fn new(
        mesh: impl Into<Mesh>,
        material: impl Into<Mat>,
        density: impl Into<Number>,
        transform: impl Into<ObjectTransform>,
    ) -> Self {
        let mesh = mesh.into();
        let transform = transform.into().with_correction(mesh.centre());
        Self::new_uncorrected(mesh, material, density, transform)
    }

    /// See [super::simple::SimpleObject::new_uncorrected()]
    pub fn new_uncorrected(
        mesh: impl Into<Mesh>,
        material: impl Into<Mat>,
        density: impl Into<Number>,
        transform: impl Into<ObjectTransform>,
    ) -> Self {
        let (mesh, material, density, transform) = (mesh.into(), material.into(), density.into(), transform.into());
        let aabb = transform.calculate_aabb(mesh.aabb());

        Self {
            mesh,
            material,
            aabb,
            transform,
            density,
            neg_inv_density: -1. / density,
        }
    }
}

// endregion Constructors

// region Object Impl

impl<Mesh, Mat> Object for VolumetricObject<Mesh, Mat>
where
    Mesh: MeshTrait,
    Mat: Material,
{
    type Mesh = Mesh;
    type Mat = Mat;

    fn full_intersect<'o>(
        &'o self,
        orig_ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Mat>> {
        let ray = self.transform.incoming_ray(orig_ray);

        // Find two samples on surface of volume
        // These should be as the ray enters and exits the mesh

        // NOTE: We should be using the `interval` parameter here, however that won't work for rays inside meshes,
        //  where the mesh is convex (many primitives are) - the first intersection will be 'behind' the ray,
        //  and so we will only get *one* forward intersection (entering), which means we don't an exiting intersection.
        //  To solve this, we check for entering intersection without interval, so that we can still check if an intersection
        //  exists at all along the ray. Then, we clamp that distance value to our interval, so we still get the right value
        let entering_dist = {
            let enter_interval = Interval::FULL;
            let d = self.mesh.intersect(&ray, &enter_interval, rng)?.dist;
            // If we have start bound, move intersection along so it happened there at the earliest
            if let Some(start) = interval.start {
                d.max(start)
            } else {
                d
            }
        };
        let exiting_dist = {
            // Have to add a slight offset so we don't intersect with the same point twice
            let exit_interval = Interval::from(entering_dist + 0.001..);
            let d = self.mesh.intersect(&ray, &exit_interval, rng)?.dist;

            // Clamp intersection dist to end of interval (if volume larger than interval)
            if let Some(end) = interval.end {
                d.min(end)
            } else {
                d
            }
        };

        // Distance between entry and exit of mesh along ray
        let dist_inside = exiting_dist - entering_dist;
        // Random distance at which we will hit
        let hit_dist = self.neg_inv_density * Number::ln(rng.gen());
        // Actual distance along the ray of the volume intersection that we'll use
        let dist = entering_dist + hit_dist;

        // NOTE: We don't do normal interval checks on intersections here, due to concavity issues given above.
        // Also, even if `exiting_dist` is outside of the range, the value `hit_dist` might be inside
        // And `hit_dist` is the one we actually use, so check that instead
        // We don't need to check `if !interval.contains(&dist)`, it's guaranteed to be inside `interval`
        // Since we clamped the entry/exit distances to the interval already

        if hit_dist > dist_inside {
            return None;
        }

        let pos_w = ray.at(dist);
        let pos_l = pos_w;

        let inter = Intersection {
            dist,
            pos_w,
            pos_l,

            // The following are all completely arbitrary
            normal: rng::normal_on_unit_sphere(rng),
            ray_normal: rng::normal_on_unit_sphere(rng),
            uv: rng::vector_in_unit_square_01(rng).to_point(),
            side: 0,
            front_face: true,
        };

        let intersect = self.transform.outgoing_intersection(orig_ray, inter);
        Some(intersect.make_full(&self.material))
    }
}

impl<Mesh: MeshTrait, Mat: Material> HasAabb for VolumetricObject<Mesh, Mat> {
    fn aabb(&self) -> Option<&Aabb> { self.aabb.as_ref() }
}

// endregion Object Impl
