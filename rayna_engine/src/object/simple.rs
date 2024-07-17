use crate::core::types::Number;
use crate::material::{Material, MaterialInstance, MaterialToken};
use crate::mesh::{Mesh as MeshTrait, MeshInstance, MeshToken};
use crate::object::transform::ObjectTransform;
use crate::object::Object;
use crate::scene::Scene;
use crate::shared::aabb::{Aabb, Bounded};
use crate::shared::intersect::ObjectIntersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use getset::{CopyGetters, Getters};
use rand_core::RngCore;

/// The main struct that encapsulates all the different "components" that make up an mesh
///
/// Very similar to a `GameObject` in a game engine, where the `ObjectInstance` and `Material` are components attached
/// to that mesh.
///
/// # Important Note
/// If using a rotating/scaling transform, ensure that the mesh you are transforming is positioned
/// at the origin (`[0., 0., 0.]`), and use the transform matrix to do the translation.
///
/// Otherwise, the centre of the mesh will be rotated/scaled around the origin as well, which will move the mesh.
///
/// Alternatively, you can also apply a post and pre-transform, to counteract the mesh's position offset:
/// ```
/// # use rayna_engine::material::lambertian::LambertianMaterial;
/// # use rayna_engine::mesh::axis_box::AxisBoxMesh;
/// # use rayna_engine::core::types::{Angle, Point3, Transform3, Vector3};
/// #
/// # let a: Point3 = [5., 1., 2.].into();
/// # let b: Point3 = [3., 4., -7.].into();
/// # let mesh = AxisBoxMesh::new(a,b);
///
/// let transform = Transform3::from_axis_angle(Vector3::Y, Angle::from_degrees(69.0));
///
/// // Fix the transform, so it scales/rotates around the mesh's centre and not the origin
/// //  1. Move centre to origin
/// //  2. Apply rotation/scaling, while it is centred at origin
/// //  3. Move centre back to original position
/// let transform = Transform3::from_translation(-mesh.centre().to_vector())
///     .then(transform)
///     .then_translate(mesh.centre().to_vector());
/// ```
///
/// This pre-/post-transform is encapsulated in [`ObjectTransform::new_corrected()`]
#[derive(Getters, CopyGetters, Clone, Debug)]
pub struct SimpleObject {
    #[get_copy = "pub"]
    mesh_tok: MeshToken,
    #[get_copy = "pub"]
    mat_tok: MaterialToken,
    #[get = "pub"]
    transform: ObjectTransform,
    aabb: Aabb,
}

// region Constructors

impl SimpleObject {
    /// Creates a new volume from a mesh and material, inserting them into the scene
    pub fn new_in(
        scene: &mut Scene,
        mesh: impl Into<MeshInstance>,
        mat: impl Into<MaterialInstance>,
        transform: impl Into<ObjectTransform>,
    ) -> Self {
        let (mesh, material, transform) = (mesh.into(), mat.into(), transform.into());
        let aabb = transform.calculate_aabb(mesh.aabb());
        let mesh_tok = scene.add_mesh(mesh);
        let mat_tok = scene.add_mat(material);

        Self {
            mesh_tok,
            mat_tok,
            aabb,
            transform,
        }
    }

    /// Creates a new volume from a mesh and material, that have already been inserted into the scene
    pub fn new_from(
        scene: &Scene,
        mesh_tok: impl Into<MeshToken>,
        mat_tok: impl Into<MaterialToken>,
        transform: impl Into<ObjectTransform>,
    ) -> Self {
        let (mesh_tok, mat_tok, transform) = (mesh_tok.into(), mat_tok.into(), transform.into());
        let aabb = transform.calculate_aabb(scene.get_mesh(mesh_tok).aabb());

        Self {
            mesh_tok,
            mat_tok,
            aabb,
            transform,
        }
    }
}

// endregion Constructors

// region Object Impl

impl Object for SimpleObject {
    fn full_intersect(
        &self,
        scene: &Scene,
        orig_ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<ObjectIntersection> {
        let trans_ray = self.transform.incoming_ray(orig_ray);
        let inner = scene.get_mesh(self.mesh_tok).intersect(&trans_ray, interval, rng)?;
        let intersect = self.transform.outgoing_intersection(orig_ray, inner);
        Some(ObjectIntersection {
            intersection: intersect,
            material: self.mat_tok,
        })
    }
}

impl Bounded for SimpleObject {
    fn aabb(&self) -> Aabb { self.aabb }
}

// endregion Object Impl
