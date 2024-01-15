use crate::material::MaterialInstance;
use crate::object::{Object, ObjectInstance, ObjectProperties};
use crate::scene::transform_utils::{transform_incoming_ray, transform_outgoing_intersection};
use crate::shared::aabb::Aabb;
use crate::shared::bounds::Bounds;
use crate::shared::camera::Camera;
use crate::shared::intersect::FullIntersection;
use crate::shared::ray::Ray;
use crate::skybox::SkyboxInstance;
use getset::Getters;
use object_list::SceneObjectList;
use rand_core::RngCore;
use rayna_shared::def::types::{Number, Point3, Transform3};
use smallvec::SmallVec;

pub mod object_list;
pub mod stored;
pub mod transform_utils;

#[derive(Clone, Debug)]
pub struct Scene {
    pub objects: SceneObjectList,
    pub camera: Camera,
    pub skybox: SkyboxInstance,
}

/// This trait is essentially an extension of [Object], but with a [FullIntersection] not [Intersection],
/// meaning the material of the object is also included.
///
/// This should only be implemented on [SceneObject], and any objects that group multiple objects together.
///
/// It's a bit of an implementation detail
pub trait FullObject {
    /// Attempts to perform an intersection between the given ray and the target object
    ///
    /// # Return Value
    /// This should return the *first* intersection that is within the given range, else [None]
    fn full_intersect<'o>(
        &'o self,
        ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o>>;

    /// Calculates all of the intersections for the given object.
    ///
    /// # Return Value
    /// This should append all the (unbounded) intersections, into the vector `output`.
    /// It can *not* be assumed this vector will be empty. The existing contents should not be modified
    fn full_intersect_all<'o>(
        &'o self,
        ray: &Ray,
        output: &mut SmallVec<[FullIntersection<'o>; 32]>,

        rng: &mut dyn RngCore,
    );

    fn aabb(&self) -> Option<&Aabb>;
}

/// The main struct that encapsulates all the different "components" that make up an object
///
/// Very similar to a `GameObject` in a game engine, where the `ObjectInstance` and `Material` are components attached
/// to that object.
///
/// # Important Note
/// If using a rotating/scaling transform, ensure that the object you are transforming is positioned
/// at the origin (`[0., 0., 0.]`), and use the transform matrix to do the translation.
///
/// Otherwise, the centre of the object will be rotated/scaled around the origin as well, which will move the object.
///
/// Alternatively, you can also apply a post and pre-transform, to counteract the object's position offset:
/// ```
/// # use rayna_engine::material::lambertian::LambertianMaterial;
/// # use rayna_engine::object::axis_box::{AxisBoxBuilder, AxisBoxObject};
/// # use rayna_shared::def::types::{Angle, Point3, Transform3, Vector3};
/// #
/// # let a: Point3 = [5., 1., 2.].into();
/// # let b: Point3 = [3., 4., -7.].into();
/// # let object: AxisBoxObject = AxisBoxBuilder {
/// #     corner_1: a,
/// #     corner_2: b,
/// # }.into();
///
/// let transform = Transform3::from_axis_angle(Vector3::Y, Angle::from_degrees(69.0));
///
/// // Fix the transform so it scales/rotates around the object's centre and not the origin
/// //  1. Move centre to origin
/// //  2. Apply rotate/scale, while it is centred at origin
/// //  3. Move centre back to original position
/// let transform = Transform3::from_translation(-object.centre().to_vector())
///     .then(transform)
///     .then_translate(object.centre().to_vector());
/// ```
///
/// This pre/post transform is encapsulated in [SceneObject::new_with_correction()]
#[derive(Getters, Clone, Debug)]
#[get = "pub"]
pub struct SceneObject {
    object: ObjectInstance,
    material: MaterialInstance,
    transform: Option<Transform3>,
    inv_transform: Option<Transform3>,
    aabb: Option<Aabb>,
    /// The centre of the object. May not be equal to [ObjectProperties::centre()],
    /// if the object has been translated
    centre: Point3,
    // TODO: Add a string identifier to this (name?)
}

impl FullObject for SceneObject {
    fn full_intersect<'o>(
        &'o self,
        orig_ray: &Ray,
        bounds: &Bounds<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<FullIntersection<'o>> {
        if let (Some(transform), Some(inv_transform)) = (&self.transform, &self.inv_transform) {
            let trans_ray = transform_incoming_ray(orig_ray, inv_transform);
            let inner = self.object.intersect(&trans_ray, bounds, rng)?;
            let intersect = transform_outgoing_intersection(orig_ray, &inner, transform);
            Some(intersect.make_full(&self.material))
        } else {
            Some(self.object.intersect(orig_ray, bounds, rng)?.make_full(&self.material))
        }
    }

    fn full_intersect_all<'o>(
        &'o self,
        orig_ray: &Ray,
        output: &mut SmallVec<[FullIntersection<'o>; 32]>,
        rng: &mut dyn RngCore,
    ) {
        if let (Some(transform), Some(inv_transform)) = (&self.transform, &self.inv_transform) {
            let trans_ray = transform_incoming_ray(orig_ray, inv_transform);
            let mut inner_intersects = SmallVec::new();
            self.object.intersect_all(&trans_ray, &mut inner_intersects, rng);

            output.extend(inner_intersects.into_iter().map(|mut inner| {
                inner = transform_outgoing_intersection(orig_ray, &inner, transform);
                inner.make_full(&self.material)
            }));
        } else {
            let mut inner_intersects = SmallVec::new();
            self.object.intersect_all(&orig_ray, &mut inner_intersects, rng);

            output.extend(
                inner_intersects
                    .into_iter()
                    .map(|inner| inner.make_full(&self.material)),
            );
        }
    }

    fn aabb(&self) -> Option<&Aabb> { self.aabb.as_ref() }

    // TODO: A fast method that simply checks if an intersection occurred at all, with no more info (shadow checks)
}

// region Constructors

impl SceneObject {
    /// Creates a new transformed object instance, using the given object and transform matrix.
    ///
    /// Unlike [Self::new()], this *does* account for the object's translation from the origin,
    /// using the `obj_centre` parameter. See field documentation ([Self::transform]) for explanation
    /// and example of this position offset correction
    pub fn new_with_correction(
        object: impl Into<ObjectInstance>,
        material: impl Into<MaterialInstance>,
        transform: Transform3,
    ) -> Self {
        let object = object.into();

        let obj_centre = object.centre();
        let correct_transform = Transform3::from_translation(-obj_centre.to_vector())
            .then(transform)
            .then_translate(obj_centre.to_vector());

        Self::new_without_correction(object, material, correct_transform)
    }

    /// Creates a new transformed object instance, using the given object and transform
    ///
    /// It is assumed that the object is either centred at the origin and the translation is stored in
    /// the transform, or that the transform correctly accounts for the object's translation.
    /// See field documentation ([Self::transform]) for explanation
    pub fn new_without_correction(
        object: impl Into<ObjectInstance>,
        material: impl Into<MaterialInstance>,
        transform: Transform3,
    ) -> Self {
        let object = object.into();

        // Calculate the resulting AABB by transforming the corners of the input AABB.
        // And then we encompass those
        let aabb = object
            .aabb()
            .map(Aabb::corners)
            .map(|corners| corners.map(|c| transform.map_point(c)))
            .map(Aabb::encompass_points);

        let inv_transform = transform.inverse();
        let centre = transform.map_point(object.centre());

        Self {
            object,
            aabb,
            transform: Some(transform),
            inv_transform: Some(inv_transform),
            centre,
            material: material.into(),
        }
    }

    /// Creates a new transformed object instance, using the given object. This method does not transform the [SceneObject]
    pub fn new(object: impl Into<ObjectInstance>, material: impl Into<MaterialInstance>) -> Self {
        // Calculate the resulting AABB by transforming the corners of the input AABB.
        let object = object.into();
        Self {
            aabb: object.aabb().copied(),
            transform: None,
            inv_transform: None,
            centre: object.centre(),
            material: material.into(),
            object,
        }
    }
}

// endregion Constructors
