use crate::core::targets::MESH;
use crate::core::types::{Number, Point3, Vector3};
use crate::mesh::advanced::bvh::BvhMesh;
//use crate::mesh::advanced::triangle::BatchTriangle;
use crate::mesh::isosurface::SdfGeneratorFunction;
use crate::mesh::primitive::triangle::Triangle;
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use derivative::Derivative;
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

/// How many triangles we batch at once
const N_TRI: usize = 1;

/// A mesh struct that is created by creating an isosurface from a given SDF
///
/// # Transforming
/// This mesh purposefully does not have any properties for transforming,
/// so you must offset the resulting object using a transform
#[derive(CopyGetters, Getters, Derivative, Clone)]
#[derivative(Debug)]
pub struct PolygonisedIsosurfaceMesh {
    #[get_copy = "pub"]
    resolution: usize,
    /// How many total triangles there are in this [PolygonisedIsosurfaceMesh]
    #[get_copy = "pub"]
    count: usize,
    #[derivative(Debug = "ignore")]
    #[get = "pub"]
    mesh: BvhMesh<Triangle>,
}

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
    pub fn new<F: SdfGeneratorFunction>(resolution: usize, sdf: F) -> Self {
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

        let mut triangles = vec![];

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

        // Now batch the triangles together
        // TODO: Don't skip the remainder
        // for (vertices, normals) in zip(tri_verts.chunks(N_TRI), tri_normals.chunks(N_TRI)) {
        for (vertices, normals) in zip(tri_verts, tri_normals) {
            triangles.push(Triangle::new(vertices, normals));
        }

        let count = triangles.len();
        let mesh = BvhMesh::new(triangles);

        Self {
            count,
            resolution,
            mesh,
        }
    }
}

// endregion Constructors

// region Isosurface Helper

/// A custom wrapper struct around an [SdfGeneratorFunction]
///
/// It is used for
struct SdfWrapper<F: SdfGeneratorFunction> {
    pub func: F,
    pub epsilon: Number,
}

// TODO: See if we can use Numbers (f64) with [SdfWrapper],
//  instead of converting to/from f32
impl<F: SdfGeneratorFunction> ScalarSource for SdfWrapper<F> {
    fn sample_scalar(&self, Vec3 { x, y, z }: Vec3) -> Signed {
        let point = [x, y, z].map(|n| n as Number).into();
        Signed((self.func)(point) as f32)
    }
}

impl<F: SdfGeneratorFunction> HermiteSource for SdfWrapper<F> {
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

impl HasAabb for PolygonisedIsosurfaceMesh {
    fn aabb(&self) -> Option<&Aabb> { self.mesh.aabb() }
}

impl MeshProperties for PolygonisedIsosurfaceMesh {
    fn centre(&self) -> Point3 { Point3::ZERO }
}

impl Mesh for PolygonisedIsosurfaceMesh {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        self.mesh.intersect(ray, interval, rng)
    }
}

// endregion Mesh Impl
