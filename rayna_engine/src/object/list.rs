use derivative::Derivative;
use getset::Getters;
use rand_core::RngCore;
use smallvec::SmallVec;

use crate::material::Material;
use crate::mesh;
use crate::object::bvh::BvhObject;
use crate::object::{Object, ObjectInstance};
use crate::shared::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;
use crate::shared::transform_utils::{transform_incoming_ray, transform_outgoing_intersection};
use rayna_shared::def::types::{Number, Point3, Transform3};

#[derive(Getters, Derivative)]
#[get = "pub"]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct ObjectList<Mesh, Mat, Obj>
where
    Mesh: mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh = Mesh, Mat = Mat> + Clone,
{
    /// BVH-optimised tree of objects
    bvh: BvhObject<Mesh, Mat, Obj>,
    /// All the unbounded objects in the list (objects where [Object::aabb()] returned [None]
    unbounded: Vec<Obj>,
    transform: Option<Transform3>,
    inv_transform: Option<Transform3>,
    /// The [Aabb] for all of the enclosed objects. Will be [None] if there are unbounded objects
    aabb: Option<Aabb>,
}

// region From<> Impl

// Iter<Into<ObjType>> => ObjectList
impl<Mesh, Mat, Obj, Iter> From<Iter> for ObjectList<Mesh, Mat, Obj>
where
    Mesh: mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh = Mesh, Mat = Mat> + Clone,
    Iter: IntoIterator<Item = Obj>,
{
    fn from(value: Iter) -> Self { Self::new(value) }
}

// Iter<Into<ObjType> => ObjectInstance
impl<Mesh, Mat, Obj, Iter> From<Iter> for ObjectInstance<Mesh, Mat>
where
    Mesh: mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Into<ObjectInstance<Mesh, Mat>>,
    Iter: IntoIterator<Item = Obj>,
{
    fn from(value: Iter) -> Self {
        // Convert each object into an ObjectInstance
        let object_instances = value.into_iter().map(Obj::into);
        // Convert to plain ObjectList
        let list = ObjectList::new(object_instances);
        Self::ObjectList(list)
    }
}

// endregion From<> Impl

// region Constructors

impl<Mesh, Mat, Obj> ObjectList<Mesh, Mat, Obj>
where
    Mesh: mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh = Mesh, Mat = Mat> + Clone,
{
    /// Creates a new transformed mesh instance, using the given mesh and transform matrix.
    ///
    /// Unlike [Self::new_without_correction()], this *does* account for the mesh's translation from the origin,
    /// using the `centre` parameter. See type documentation ([super::simple::SimpleObject]) for explanation
    /// and example of this position offset correction
    pub fn new_with_correction(objects: impl IntoIterator<Item = Obj>, transform: Transform3, centre: Point3) -> Self {
        let correct_transform = Transform3::from_translation(-centre.to_vector())
            .then(transform)
            .then_translate(centre.to_vector());

        Self::new_without_correction(objects, correct_transform)
    }

    /// Creates a new transformed mesh instance, using the given mesh and transform
    ///
    /// It is assumed that the mesh is either centred at the origin and the translation is stored in
    /// the transform, or that the transform correctly accounts for the mesh's translation.
    /// See type documentation ([super::simple::SimpleObject]) for explanation
    pub fn new_without_correction(objects: impl IntoIterator<Item = Obj>, transform: Transform3) -> Self {
        let (bvh, unbounded, aabb) = Self::process_objects(objects);

        // Calculate the resulting AABB by transforming the corners of the input AABB.
        // And then we encompass those
        let aabb = aabb
            .as_ref()
            .map(Aabb::corners)
            .map(|corners| corners.map(|c| transform.map_point(c)))
            .map(Aabb::encompass_points);

        let inv_transform = transform.inverse();

        Self {
            aabb,
            transform: Some(transform),
            inv_transform: Some(inv_transform),
            bvh,
            unbounded,
        }
    }

    /// Creates a new [ObjectList] instance, using the given mesh. This method does not transform the [ObjectList]
    pub fn new(objects: impl IntoIterator<Item = Obj>) -> Self {
        let (bvh, unbounded, aabb) = Self::process_objects(objects);

        Self {
            bvh,
            unbounded,
            aabb,
            transform: None,
            inv_transform: None,
        }
    }

    /// A helper method for transforming an iterator of objects into a [BvhObject] tree, a [Vec] of unbounded objects, and an AABB
    fn process_objects(objects: impl IntoIterator<Item = Obj>) -> (BvhObject<Mesh, Mat, Obj>, Vec<Obj>, Option<Aabb>) {
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
            Some(Aabb::encompass_iter(
                bounded.iter().map(Object::aabb).map(Option::unwrap),
            ))
        } else {
            None
        };
        let bvh = BvhObject::new(bounded);

        (bvh, unbounded, aabb)
    }
}

// endregion Constructors

impl<Mesh, Mat, Obj> Object for ObjectList<Mesh, Mat, Obj>
where
    Mesh: mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh = Mesh, Mat = Mat> + Clone,
{
    type Mesh = Mesh;
    type Mat = Mat;

    fn full_intersect<'o>(
        &'o self,
        orig_ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o, Mat>> {
        // NOTE: This transformation code is practically the same as SimpleObject's implementation
        return match (&self.transform, &self.inv_transform) {
            (Some(transform), Some(inv_transform)) => {
                let trans_ray = transform_incoming_ray(orig_ray, inv_transform);

                let bvh_int = self.bvh.full_intersect(&trans_ray, bounds, rng).into_iter();
                let unbound_int = self
                    .unbounded
                    .iter()
                    .filter_map(|o| o.full_intersect(&trans_ray, bounds, rng));
                let mut intersect = Iterator::chain(bvh_int, unbound_int).min()?;

                intersect.intersection = transform_outgoing_intersection(orig_ray, intersect.intersection, transform);
                Some(intersect)
            }
            _ => {
                let bvh_int = self.bvh.full_intersect(orig_ray, bounds, rng).into_iter();
                let unbound_int = self
                    .unbounded
                    .iter()
                    .filter_map(|o| o.full_intersect(orig_ray, bounds, rng));
                Iterator::chain(bvh_int, unbound_int).min()
            }
        };
    }

    fn full_intersect_all<'o>(
        &'o self,
        orig_ray: &Ray,
        output: &mut SmallVec<[FullIntersection<'o, Mat>; 32]>,
        rng: &mut dyn RngCore,
    ) {
        if let (Some(transform), Some(inv_transform)) = (&self.transform, &self.inv_transform) {
            let trans_ray = transform_incoming_ray(orig_ray, inv_transform);

            let mut inner_intersects = SmallVec::new();
            self.bvh.full_intersect_all(&trans_ray, &mut inner_intersects, rng);
            self.unbounded
                .iter()
                .for_each(|o| o.full_intersect_all(&trans_ray, &mut inner_intersects, rng));

            // Undo the transform on the intersections
            output.extend(inner_intersects.into_iter().map(|mut inner| {
                inner.intersection = transform_outgoing_intersection(orig_ray, inner.intersection, transform);
                inner
            }));
        } else {
            self.bvh.full_intersect_all(orig_ray, output, rng);
            self.unbounded
                .iter()
                .for_each(|o| o.full_intersect_all(orig_ray, output, rng));
        }
    }

    fn aabb(&self) -> Option<&Aabb> { self.aabb.as_ref() }
}
