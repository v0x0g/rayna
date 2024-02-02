use crate::core::types::{Number, Point3};
use crate::mesh::advanced::bvh::BvhMesh;
use crate::mesh::planar::triangle::TriangleMesh;
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::math::Lerp;
use crate::shared::ray::Ray;
use derivative::Derivative;
use getset::{CopyGetters, Getters};
use isosurface::distance::Signed;
use isosurface::sampler::Sampler;
use isosurface::source::ScalarSource;
use isosurface::MarchingCubes;
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
    pub fn generate<F: SdfGeneratorFunction>(resolution: usize, func: F) -> Self {
        // let source = SdfSource { func };
        let mut raw_vertices = vec![];
        // let mut raw_indices = vec![];
        let source = isosurface::implicit::Sphere::new(1.0);
        let mut extractor = isosurface::extractor::OnlyVertices::new(&mut raw_vertices);
        let sampler = Sampler::new(&source);
        MarchingCubes::<Signed>::new(resolution).extract(&sampler, &mut extractor);

        // assert_eq!(
        //     raw_indices.len() % 3,
        //     0,
        //     "`indices.len` should be multiple of 3 (was {})",
        //     raw_indices.len()
        // );

        // Group the vertex coordinates into groups of three, so we get a 3D point
        // let isosurface_vertices = raw_vertices
        //     .array_chunks::<3>()
        //     .map(|vs| Point3::from(vs.map(|v| v as Number)))
        //     .collect_vec();

        // let isosurface_indices = raw_indices
        //     .array_chunks::<3>()
        //     .map(|vs| vs.map(|v| v as usize))
        //     .collect_vec();

        // let mut triangles = Vec::with_capacity(raw_indices.len() % 3);
        let mut triangles = vec![];

        for &vertices in raw_vertices.array_chunks::<9>() {
            // If we imagine the three vertices as `a, b, c`,
            // We get the coordinates as:
            // [a.x, b.x, c.x,
            //  a.y, b.y, c.y,
            //  a.z, b.z, c.z]
            // let [ax, bx, cx, ay, by, cy, az, bz, cz] = vertices.map(|n| n as Number);
            let [ax, ay, az, bx, by, bz, cx, cy, cz] = vertices.map(|n| n as Number);
            triangles.push(TriangleMesh::from([[ax, ay, az], [bx, by, bz], [cx, cy, cz]]));
        }

        // for indices in isosurface_indices {
        //     // Each index refers to the index of the `x` vertex coordinate in the buffer,
        //     // so we can divide by 3 to get the proper index as a point
        //     let vertices = indices.map(|idx| isosurface_vertices[idx]);
        //     triangles.push(TriangleMesh::from(vertices));
        // }

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
        let point = [x, y, z].map(|n| Lerp::lerp(-1., 1., n as Number)).into();
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
