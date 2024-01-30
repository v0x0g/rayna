use crate::core::types::{Number, Point3, Size3, Vector3};
use crate::mesh::advanced::bvh::BvhMesh;
use crate::mesh::primitive::axis_box::AxisBoxMesh;
use crate::mesh::{Mesh, MeshProperties};
use crate::shared::aabb::{Aabb, HasAabb};
use crate::shared::intersect::Intersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;
use derivative::Derivative;
use getset::{CopyGetters, Getters};
use itertools::Itertools;
use ndarray::{ArcArray, Dimension, Ix3, Shape};
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
    voxels: BvhMesh<AxisBoxMesh>,
}

pub trait GeneratorFunction = Fn(Point3) -> Number;

// region Constructors

impl VoxelGridMesh {
    pub fn generate([width, height, depth]: [usize; 3], func: impl GeneratorFunction, thresh: Number) -> Self {
        let dims = Ix3(width, height, depth);
        let centre = Self::index_to_pos(dims) / 2.;
        // Create raw grid of voxels, using provided function for each grid point
        let data = ArcArray::from_shape_fn(Shape::from(dims), |(x, y, z)| {
            let p = Self::index_to_pos(Ix3(x, y, z)) - centre;
            func(p.to_point())
        });

        let voxels = data
            .indexed_iter()
            .filter_map(|((x, y, z), &v)| {
                if v >= thresh {
                    Some(AxisBoxMesh::new_centred(
                        (Self::index_to_pos(Ix3(x, y, z)) - centre).to_point(),
                        Size3::ONE,
                    ))
                } else {
                    None
                }
            })
            .collect_vec();
        let voxels = BvhMesh::new(voxels);

        Self {
            width,
            height,
            depth,
            count: data.len(),
            centre: centre.to_point(),
            data,
            voxels,
            thresh,
        }
    }
}

// endregion Constructors

// region Helper

impl VoxelGridMesh {
    pub fn index_to_pos(index: Ix3) -> Vector3 {
        let (x, y, z) = index.into_pattern();
        [x, y, z].map(|n| n as Number).into()
    }
}

// endregion Helper

// region Mesh Impl

impl HasAabb for VoxelGridMesh {
    fn aabb(&self) -> Option<&Aabb> { self.voxels.aabb() }
}

impl MeshProperties for VoxelGridMesh {
    fn centre(&self) -> Point3 { self.centre }
}

impl Mesh for VoxelGridMesh {
    fn intersect(&self, ray: &Ray, interval: &Interval<Number>, rng: &mut dyn RngCore) -> Option<Intersection> {
        self.voxels.intersect(ray, interval, rng)
    }
}

// endregion Mesh Impl
