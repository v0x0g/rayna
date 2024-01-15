use derivative::Derivative;
use getset::Getters;
use rand_core::RngCore;
use smallvec::SmallVec;

use crate::material::Material;
use crate::object::bvh::Bvh;
use crate::object::Object;
use crate::shared::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;
use rayna_shared::def::types::Number;

#[derive(Getters, Derivative)]
#[get = "pub"]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct ObjectList<Mesh, Mat, Obj>
where
    Mesh: crate::mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh, Mat> + Clone,
{
    /// BVH-optimised tree of objects
    bvh: Bvh<Mesh, Mat, Obj>,
    /// All the unbounded objects in the list (objects where [Object::aabb()] returned [None]
    unbounded: Vec<Obj>,
}

// Iter<Into<ObjType>> => ObjectList
impl<Mesh, Mat, Obj, Iter> From<Iter> for ObjectList<Mesh, Mat, Obj>
where
    Mesh: crate::mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh, Mat> + Clone,
    Iter: IntoIterator<Item = Obj>,
{
    fn from(value: Iter) -> Self {
        let mut bounded = vec![];
        let mut unbounded = vec![];
        for obj in value.into_iter() {
            if let Some(_) = obj.aabb() {
                bounded.push(obj);
            } else {
                unbounded.push(obj);
            }
        }
        let bvh = Bvh::new(bounded);
        Self { bvh, unbounded }
    }
}

impl<Mesh, Mat, Obj> Object<Mesh, Mat> for ObjectList<Mesh, Mat, Obj>
where
    Mesh: crate::mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh, Mat> + Clone,
{
    fn full_intersect<'o>(
        &'o self,
        ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Mat>> {
        let bvh_int = self.bvh.full_intersect(ray, bounds, rng).into_iter();
        let unbound_int = self.unbounded.iter().filter_map(|o| o.full_intersect(ray, bounds, rng));
        Iterator::chain(bvh_int, unbound_int).min()
    }

    fn full_intersect_all<'o>(
        &'o self,
        ray: &Ray,
        output: &mut SmallVec<[FullIntersection<'o, Mat>; 32]>,
        rng: &mut dyn RngCore,
    ) {
        self.bvh.full_intersect_all(ray, output, rng);
        self.unbounded
            .iter()
            .for_each(|o| o.full_intersect_all(ray, output, rng));
    }

    fn aabb(&self) -> Option<&Aabb> {
        // List may have unbounded objects, so we can't return Some()
        None
    }
}
