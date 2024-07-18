use getset::{CopyGetters, Getters};
use itertools::Itertools;
use rand_core::RngCore;

use crate::core::types::Number;
use crate::object::{Object, ObjectInstance, ObjectToken};
use crate::scene::Scene;
use crate::shared::aabb::{Aabb, Bounded};
use crate::shared::intersect::ObjectIntersection;
use crate::shared::interval::Interval;
use crate::shared::ray::Ray;

/// A group of objects that are rendered as one object
#[derive(Clone, Debug, Getters, CopyGetters)]
pub struct ListObject {
    #[get_copy = "pub"]
    aabb: Aabb,
    #[get = "pub"]
    objects: Vec<ObjectToken>,
}

// region Constructors

impl ListObject {
    /// Creates a list of objects that have already been inserted into the scene
    pub fn new_from(scene: &Scene, tokens: impl IntoIterator<Item: Into<ObjectToken>>) -> Self {
        let tokens = tokens.into_iter().map(Into::into).collect_vec();
        let aabb = Aabb::encompass_iter(tokens.iter().map(|t| scene.get_object(t).aabb()));
        Self { aabb, objects: tokens }
    }

    /// Creates a list of objects, adding them to the scene
    pub fn new_in(scene: &mut Scene, objects: impl IntoIterator<Item: Into<ObjectInstance>>) -> Self {
        let objects = objects.into_iter().map(Into::into);
        let aabb = Aabb::encompass_iter(objects.iter().map(|m| m.aabb()));
        let tokens = objects.into_iter().map(|m| scene.add_mesh(m));
        Self { aabb, objects: tokens }
    }
}

/// Create [`ListObject`] from iterator of [`ObjectToken`]
impl<Iter: IntoIterator<Item: Into<ObjectToken>>> From<Iter> for ListObject {
    fn from(objects: Iter) -> Self { Self::new(objects) }
}

/// Create ([`ListObject`] as [`ObjectInstance`]) from iterator of [`ObjectToken`]
impl<Iter: IntoIterator<Item: Into<ObjectToken>>> From<Iter> for ObjectInstance {
    fn from(objects: Iter) -> Self { ListObject::new(objects.into_iter().map(Into::into)).into() }
}

// endregion Constructors

// region Object Impl
impl Bounded for ListObject {
    fn aabb(&self) -> Aabb { self.aabb }
}

impl Object for ListObject {
    fn full_intersect(
        &self,
        scene: &Scene,
        ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<ObjectIntersection> {
        self.objects
            .iter()
            .map(|t| scene.get_object(t))
            .filter_map(|m| m.full_intersect(scene, ray, interval, rng))
            .min()
    }
}

// endregion Object Impl
