use crate::core::types::{Number, Point3, Size3, Vector3};
use crate::mesh::advanced::bvh::BvhMesh;
use crate::mesh::planar::triangle::TriangleMesh;
use crate::mesh::primitive::axis_box::AxisBoxMesh;
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use derivative::Derivative;
use getset::{CopyGetters, Getters};
use itertools::Itertools;
use ndarray::{ArcArray, Ix3, Shape};
use rand_core::RngCore;

/// A mesh struct that is created from a grid of voxels
///
/// # Transforming
/// This mesh purposefully does not have any properties for transforming, so you must you a
/// [ObjectTransform].
#[derive(CopyGetters, Getters, Derivative, Clone)]
#[derivative(Debug)]
pub struct VoxelGridMesh {
    #[get_copy = "pub"]
    width: usize,
    #[get_copy = "pub"]
    height: usize,
    #[get_copy = "pub"]
    depth: usize,
    #[get_copy = "pub"]
    centre: Point3,
    /// How many total voxels there are in this [VoxelGridMesh]
    #[get_copy = "pub"]
    count: usize,
    /// The raw data for the grid
    #[derivative(Debug = "ignore")]
    #[get = "pub"]
    data: ArcArray<Number, Ix3>,
    #[get_copy = "pub"]
    thresh: Number,
    #[derivative(Debug = "ignore")]
    #[get = "pub"]
    mesh_data: BvhMesh<TriangleMesh>,
}

pub trait GeneratorFunction = Fn(Point3) -> Number;

// region Constructors

impl VoxelGridMesh {
    pub fn generate(
        resolution: [usize; 3],
        mesh_centre: impl Into<Point3>,
        mesh_scale: impl Into<Size3>,
        func_centre: impl Into<Point3>,
        func_scale: impl Into<Size3>,
        func: impl GeneratorFunction,
        thresh: Number,
    ) -> Self {
        // (The position that the voxel_centre maps to, and a scale around that centre point)
        let (func_centre, func_scale) = (func_centre.into(), func_scale.into());
        let (mesh_centre, mesh_scale) = (mesh_centre.into(), mesh_scale.into());
        let [width, height, depth] = resolution;

        // Position of the centre of the voxels (central point in the 3D grid)
        let grid_dims: Vector3 = resolution.map(|n| n as Number).into();
        let grid_centre: Vector3 = resolution.map(|n| (n - 1) as Number / 2.).into();

        // How large each voxel should be
        let voxel_size = (mesh_scale.to_vector() / grid_dims).to_size();

        // Create raw grid of voxels, using provided function for each grid point
        let data = ArcArray::from_shape_fn(Shape::from(Ix3(width, height, depth)), |p| {
            func(Self::idx_to_func_pos(
                p,
                grid_centre,
                grid_dims,
                func_centre,
                func_scale,
            ))
        });

        // Convert data to the mesh data
        let mesh_data = Self::extract_isosurface(
            data,
            grid_centre, grid_dims, mesh_centre
        )

        Self {
            width,
            height,
            depth,
            count: data.len(),
            centre: grid_centre.to_point(),
            data,
            mesh_data: voxels,
            thresh,
        }
    }
}

// endregion Constructors

// region Helper

impl VoxelGridMesh {
    fn idx_to_world_pos(
        (x, y, z): (usize, usize, usize),
        grid_centre: impl Into<Vector3>,
        grid_dims: impl Into<Vector3>,
        mesh_centre: impl Into<Vector3>,
        mesh_scale: impl Into<Vector3>,
    ) -> Point3 {
        let (grid_centre, grid_dims, mesh_centre, mesh_scale) = (
            grid_centre.into(),
            grid_dims.into(),
            mesh_centre.into(),
            mesh_scale.into(),
        );
        let idx_vec = Vector3::from([x, y, z].map(|n| n as Number));
        // centre so the coords range `-dim/2 .. dim/2`
        let idx_centred = idx_vec - grid_centre;
        // normalise to `-0.5..0.5`
        let idx_norm = idx_centred / grid_dims;
        // scale according to the mesh's scale
        let idx_scaled = idx_norm * mesh_scale;
        // offset according to the mesh's centre
        let point = mesh_centre + idx_scaled;
        point.to_point()
    }

    fn idx_to_func_pos(
        (x, y, z): (usize, usize, usize),
        grid_centre: impl Into<Vector3>,
        grid_dims: impl Into<Vector3>,
        func_centre: impl Into<Vector3>,
        func_scale: impl Into<Vector3>,
    ) -> Point3 {
        let (grid_centre, grid_dims, func_centre, func_scale) = (
            grid_centre.into(),
            grid_dims.into(),
            func_centre.into(),
            func_scale.into(),
        );
        let idx_vec = Vector3::from([x, y, z].map(|n| n as Number));
        // centre so the coords range `-dim/2 .. dim/2`
        let idx_centred = idx_vec - grid_centre;
        // normalise to `-0.5..0.5`
        let idx_norm = idx_centred / grid_dims;
        // scale according to the function's scale
        let idx_scaled = idx_norm * func_scale;
        // offset according to the function's centre
        let point = func_centre + idx_scaled;
        point.to_point()
    }

    fn extract_isosurface(data: ArcArray<Number, Ix3>,
                          grid_centre: impl Into<Vector3>,
                          grid_dims: impl Into<Vector3>,
                          mesh_centre: impl Into<Vector3>,
                          mesh_scale: impl Into<Vector3>,
                          voxel_size: impl Into<Size3>,
                          thresh: Number) -> BvhMesh<TriangleMesh> {
        let mesh_data= data
            .indexed_iter()
            .filter_map(|(p, &v)| {
                if v < thresh {
                    Some(AxisBoxMesh::new_centred(
                        Self::idx_to_world_pos(p, grid_centre, grid_dims, mesh_centre, mesh_scale),
                        voxel_size,
                    ))
                } else {
                    None
                }
            })
            .collect_vec();
        BvhMesh::new(mesh_data);
    }
}

// endregion

// region Mesh Impl

impl HasAabb for VoxelGridMesh {
    fn aabb(&self) -> Option<&Aabb> { self.mesh_data.aabb() }
}

impl MeshProperties for VoxelGridMesh {
    fn centre(&self) -> Point3 { self.centre }
}

impl Mesh for VoxelGridMesh {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        self.mesh_data.intersect(ray, interval, rng)
    }
}

// endregion Mesh Impl
