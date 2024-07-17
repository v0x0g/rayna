use crate::core::targets::MESH;
use crate::core::types::{Number, Point3, Vector3};
use crate::mesh::list::ListMesh;
use crate::mesh::triangle::TriangleMesh;
use crate::mesh::Mesh;
use crate::scene::Scene;
use crate::shared::aabb::{Aabb, Bounded};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use getset::{CopyGetters, Getters};
use isosurface::distance::Signed;
use isosurface::extractor::IndexedInterleavedNormals;
use isosurface::math::Vec3;
use isosurface::sampler::Sampler;
use isosurface::source::{HermiteSource, ScalarSource};
use isosurface::MarchingCubes;
use itertools::Itertools;
use rand_core::RngCore;
use std::iter::zip;
use tracing::warn;

/// A mesh struct that is created by creating an isosurface from a given SDF
///
/// # Transforming
/// This mesh purposefully does not have any properties for transforming,
/// so you must offset the resulting object using a transform
#[derive(CopyGetters, Getters, Clone, Debug)]
pub struct PolygonisedIsosurfaceMesh {
    #[get_copy = "pub"]
    resolution: usize,
    #[get = "pub"]
    mesh: ListMesh,
}

pub trait SdfFunction: Fn(Point3) -> Number {}
impl<F: Fn(Point3) -> Number> SdfFunction for F {}

// region Constructors

impl PolygonisedIsosurfaceMesh {
    /// Creates a new mesh from the given isosurface, as defined by the **Signed-Distance Function** (**SDF**)
    ///
    /// # Arguments
    ///
    /// * `resolution`: How dense the resulting mesh should be.
    /// The resulting mesh has dimensions of a `N*N*N` grid, where `N = resolution`
    /// * `sdf`: The **SDF** that defines the surface for the mesh.
    /// This SDF will be evaluated in local-space: `x,y,z: [0, 1]`
    pub fn new<SDF: SdfFunction>(resolution: usize, sdf: SDF) -> Self {
        let source = SdfWrapper {
            func: sdf,
            epsilon: 1e-7,
        };
        // Raw coordinates for the vertices and normals
        let mut raw_vertex_normal_coords = vec![];
        let mut raw_indices = vec![];
        MarchingCubes::<Signed>::new(resolution).extract(
            &Sampler::new(&source),
            &mut IndexedInterleavedNormals::new(&mut raw_vertex_normal_coords, &mut raw_indices, &source),
        );

        assert_eq!(
            raw_indices.len() % 3,
            0,
            "`raw_indices.len` should be multiple of 3 (was {})",
            raw_indices.len()
        );
        assert_eq!(
            raw_vertex_normal_coords.len() % 6,
            0,
            "`raw_vertex_coords.len` should be multiple of 6 (was {})",
            raw_vertex_normal_coords.len()
        );

        // Group the vertex coordinates into groups of three, so we get a 3D point
        // Interleaved with normals, so extract that out too
        let (raw_verts, raw_normals): (Vec<_>, Vec<_>) = raw_vertex_normal_coords
            .array_chunks::<3>()
            .map(|vs| vs.map(|v| v as Number))
            .array_chunks::<2>()
            .map(|[v, n]| (Point3::from(v), Vector3::from(n)))
            .unzip();

        // Group the indices in chunks of three as well, for the three vertices of each triangle
        let triangle_indices = raw_indices
            .array_chunks::<3>()
            .map(|vs| vs.map(|v| v as usize))
            .collect_vec();

        // Loop over all indices, map them to the vertex positions, and create a triangle
        // TODO: I think I'm transposing twice here which is pointless. Maybe optimise that?
        //  Not super important though since this isn't a hot path
        let (tri_verts, tri_normals): (Vec<_>, Vec<_>) = triangle_indices
            .into_iter()
            // Unpack the vertices and normals for the triangle
            .map(|vert_indices| (vert_indices.map(|i| raw_verts[i]), vert_indices.map(|i| raw_normals[i])))
            .filter_map(|(verts, normals)| {
                // Sometimes this generates "empty" triangles that have duplicate vertices
                // This is invalid, so skip those. Not sure if it's a bug or intentional :(
                if verts[0] == verts[1] || verts[1] == verts[2] || verts[2] == verts[0] {
                    warn!(target: MESH,  "triangle with empty vertices; verts: [{verts:?}]");
                    return None;
                }
                // Normals are not normalised by [SdfSource], so do that here.
                // If for any vertex there is a zero gradient normal, skip those because
                // I don't know a good way to handle them
                let Some(normals) = normals.try_map(Vector3::try_normalize) else {
                    warn!(target: MESH,  "triangle with empty normals; normals: {normals:?}");
                    return None;
                };
                return Some((verts, normals));
            })
            .unzip();

        // TODO: Use TrianglesMesh
        let triangles = zip(tri_verts, tri_normals).map(|(v, n)| TriangleMesh::new(v, n));
        let mesh = ListMesh::new(triangles);

        Self { resolution, mesh }
    }
}

// endregion Constructors

// region Isosurface Helper

/// A custom wrapper struct around an [SdfFunction]
///
/// It is used for
struct SdfWrapper<F: SdfFunction> {
    pub func: F,
    pub epsilon: Number,
}

// TODO: See if we can use Numbers (f64) with [SdfWrapper],
//  instead of converting to/from f32
impl<F: SdfFunction> ScalarSource for SdfWrapper<F> {
    fn sample_scalar(&self, Vec3 { x, y, z }: Vec3) -> Signed {
        let point = [x, y, z].map(|n| n as Number).into();
        Signed((self.func)(point) as f32)
    }
}

impl<F: SdfFunction> HermiteSource for SdfWrapper<F> {
    fn sample_normal(&self, Vec3 { x, y, z }: Vec3) -> Vec3 {
        let p = [x, y, z].map(|n| n as Number).into();
        let v = (self.func)(p);
        let dx = (self.func)(p + Vector3::new(self.epsilon, 0.0, 0.0)) - v;
        let dy = (self.func)(p + Vector3::new(0.0, self.epsilon, 0.0)) - v;
        let dz = (self.func)(p + Vector3::new(0.0, 0.0, self.epsilon)) - v;

        // NOTE: In the case that the scalar field is completely homogenous in the region,
        //  all the values will be the same and we will have no gradient, so this will be zero.
        //  Since this is an internal API, we can skip that check here and do it in the mesh generation part
        let grad = Vector3::new(dx, dy, dz);
        Vec3::new(grad.x as f32, grad.y as f32, grad.z as f32)
    }
}

// endregion Isosurface Helper

// region Mesh Impl

impl Bounded for PolygonisedIsosurfaceMesh {
    fn aabb(&self) -> Aabb { self.mesh.aabb() }
}

impl Mesh for PolygonisedIsosurfaceMesh {
    fn intersect(
        &self,
        _scene: &Scene,
        ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<Intersection> {
        self.mesh.intersect(ray, interval, rng)
    }
}

// endregion Mesh Impl
