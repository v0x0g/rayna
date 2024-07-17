use getset::Getters;
use rand_core::RngCore;

use super::transform::ObjectTransform;
use crate::core::types::{Number, Point3};
use crate::material::Material;
use crate::mesh;
use crate::object::bvh_object::BvhObject;
use crate::object::{Object, ObjectInstance};
use crate::shared::aabb::{Aabb, Bounded};
use crate::shared::intersect::FullIntersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;

#[derive(Getters, Clone, Debug)]
#[get = "pub"]
pub struct ObjectList<Obj: Object> {
    /// BVH-optimised tree of objects
    bvh: BvhObject<Obj>,
    /// All the unbounded objects in the list (objects where [`Bounded::aabb()`] returned [None]
    unbounded: Vec<Obj>,
    transform: ObjectTransform,
    /// The [Aabb] for all of the enclosed objects. Will be [None] if there are unbounded objects
    #[get(skip)]
    aabb: Option<Aabb>,
}

// region Constructors

impl<Obj: Object> ObjectList<Obj> {
    /// See [super::simple::SimpleObject::new()]
    pub fn new(
        objects: impl IntoIterator<Item = Obj>,
        transform: impl Into<ObjectTransform>,
        correct_centre: impl Into<Point3>,
    ) -> Self {
        let transform = transform.into().with_correction(correct_centre);

        Self::new_uncorrected(objects, transform)
    }

    /// See [super::simple::SimpleObject::new_uncorrected()]
    pub fn new_uncorrected(objects: impl IntoIterator<Item = Obj>, transform: impl Into<ObjectTransform>) -> Self {
        let transform = transform.into();

        let (bvh, unbounded, aabb) = Self::process_objects(objects);

        let aabb = transform.calculate_aabb(aabb.as_ref());

        Self {
            aabb,
            transform,
            bvh,
            unbounded,
        }
    }

    /// A helper method for transforming an iterator of objects into a [BvhObject] tree, a [Vec] of unbounded objects, and an AABB
    fn process_objects(objects: impl IntoIterator<Item = Obj>) -> (BvhObject<Obj>, Vec<Obj>, Option<Aabb>) {
        let mut bounded = vec![];
        let mut unbounded = vec![];
        for obj in objects.into_iter() {
            if let Some(_) = obj.aabb() {
                bounded.push(obj);
            } else {
                unbounded.push(obj);
            }
        }
        let aabb = if unbounded.is_empty() && !bounded.is_empty() {
            // All objects were checked for AABB so can unwrap
            Some(Aabb::encompass_iter(bounded.iter().map(Obj::aabb).map(Option::unwrap)))
        } else {
            None
        };
        let bvh = BvhObject::new_uncorrected(bounded, None);

        (bvh, unbounded, aabb)
    }
}

// Iter<Into<ObjType>> => ObjectList
impl<Obj: Object, Iter: IntoIterator<Item = Obj>> From<Iter> for ObjectList<Obj> {
    fn from(value: Iter) -> Self { Self::new_uncorrected(value, None) }
}

// Iter<Into<ObjType> => ObjectInstance
impl<Mesh, Mat, Obj, Iter> From<Iter> for ObjectInstance<Mesh, Mat>
where
    Mesh: mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Into<ObjectInstance<Mesh, Mat>>,
    Iter: IntoIterator<Item = Obj>,
{
    /// Converts an iterator of objects into an [`ObjectList`], wrapped as a [`ObjectInstance`]
    fn from(value: Iter) -> Self {
        // Convert each object into an ObjectInstance
        let object_instances = value.into_iter().map(Obj::into);
        // Convert to plain ObjectList
        let list = ObjectList::new_uncorrected(object_instances, None);
        Self::ObjectList(list)
    }
}

// endregion Constructors

// region Object Impl

impl<Obj: Object> Object for ObjectList<Obj> {
    type Mesh = Obj::Mesh;
    type Mat = Obj::Mat;

    fn full_intersect<'o>(
        &'o self,
        orig_ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Obj::Mat>> {
        let trans_ray = self.transform.incoming_ray(orig_ray);

        let bvh_int = self.bvh.full_intersect(&trans_ray, interval, rng).into_iter();
        let unbound_int = self
            .unbounded
            .iter()
            .filter_map(|o| o.full_intersect(&trans_ray, interval, rng));
        let mut intersect = Iterator::chain(bvh_int, unbound_int).min()?;

        intersect.intersection = self.transform.outgoing_intersection(orig_ray, intersect.intersection);
        Some(intersect)
    }
}
impl<Obj: Object> Bounded for ObjectList<Obj> {
    fn aabb(&self) -> Aabb { self.aabb.as_ref() }
}

// endregion Object Impl
