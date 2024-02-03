use crate::core::types::{Number, Point3};
use crate::mesh::advanced::bvh::BvhMesh;
use crate::mesh::planar::triangle::TriangleMesh;
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use derivative::Derivative;
use getset::{CopyGetters, Getters};
use isosurface::distance::Signed;
use isosurface::extractor::IndexedVertices;
use isosurface::feature::MinimiseQEF;
use isosurface::sampler::Sampler;
use isosurface::source::{CentralDifference, ScalarSource};
use itertools::Itertools;
use rand_core::RngCore;

/// A mesh struct that is created by creating an isosurface from a given SDF
///
/// # Transforming
/// This mesh purposefully does not have any properties for transforming, so you must you a
/// [ObjectTransform].
#[derive(CopyGetters, Getters, Derivative, Clone)]
#[derivative(Debug)]
pub struct IsosurfaceMesh {
    #[get_copy = "pub"]
    resolution: usize,
    /// How many total triangles there are in this [IsosurfaceMesh]
    #[get_copy = "pub"]
    count: usize,
    #[derivative(Debug = "ignore")]
    #[get = "pub"]
    mesh: BvhMesh<TriangleMesh>,
}

pub trait SdfGeneratorFunction = Fn(Point3) -> Number;

// region Constructors

impl IsosurfaceMesh {
    /// Creates a new mesh from the given isosurface, as defined by the **Signed-Distance Function** (**SDF**)
    ///
    /// # Arguments
    ///
    /// * `resolution`: How dense the resulting mesh should be.
    /// The resulting mesh has dimensions of a `N*N*N` grid, where `N = resolution`
    /// * `sdf`: The **SDF** that defines the surface for the mesh
    pub fn new<F: SdfGeneratorFunction>(resolution: usize, sdf: F) -> Self {
        let source = CentralDifference::new(SdfSource { func: sdf });
        let mut raw_vertex_coords = vec![];
        let mut raw_indices = vec![];
        let mut extractor = IndexedVertices::new(&mut raw_vertex_coords, &mut raw_indices);
        let sampler = Sampler::new(&source);
        isosurface::DualContouring::new(resolution, MinimiseQEF {}).extract(&sampler, &mut extractor);

        assert_eq!(
            raw_indices.len() % 3,
            0,
            "`raw_indices.len` should be multiple of 3 (was {})",
            raw_indices.len()
        );
        assert_eq!(
            raw_vertex_coords.len() % 3,
            0,
            "`raw_vertex_coords.len` should be multiple of 3 (was {})",
            raw_vertex_coords.len()
        );

        // Group the vertex coordinates into groups of three, so we get a 3D point
        let triangle_vertices = raw_vertex_coords
            .array_chunks::<3>()
            .map(|vs| Point3::from(vs.map(|v| v as Number)))
            .collect_vec();

        // Group the indices in chunks of three as well, for the three vertices of each triangle
        let triangle_indices = raw_indices
            .array_chunks::<3>()
            .map(|vs| vs.map(|v| v as usize))
            .collect_vec();

        let mut triangles = vec![];

        // Loop over all indices, map them to the vertex positions, and create a triangle
        for [a, b, c] in triangle_indices
            .into_iter()
            .map(|vert_indices| vert_indices.map(|v_i| triangle_vertices[v_i]))
        {
            // Sometimes this generates "empty" triangles that have duplicate vertices
            // This is invalid, so skip those. Not sure if it's a bug or intentional :(
            if a == b || b == c || c == a {
                continue;
            }
            // NOTE: Vertex ordering is important, should be `[a,b,c]` where `b` is adjacent to `a,c`
            triangles.push(TriangleMesh::new([a, b, c]));
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

struct SdfSource<F: SdfGeneratorFunction> {
    pub func: F,
}
impl<F: SdfGeneratorFunction> ScalarSource for SdfSource<F> {
    fn sample_scalar(&self, isosurface::math::Vec3 { x, y, z }: isosurface::math::Vec3) -> Signed {
        let point = [x, y, z].map(|n| n as Number).into();
        Signed((self.func)(point) as f32)
    }
}

// endregion Isosurface Helper

// region Mesh Impl

impl HasAabb for IsosurfaceMesh {
    fn aabb(&self) -> Option<&Aabb> { self.mesh.aabb() }
}

impl MeshProperties for IsosurfaceMesh {
    fn centre(&self) -> Point3 { Point3::ZERO }
}

impl Mesh for IsosurfaceMesh {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        self.mesh.intersect(ray, interval, rng)
    }
}

// endregion Mesh Impl
