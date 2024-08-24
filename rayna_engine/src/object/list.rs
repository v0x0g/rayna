use getset::{CopyGetters, Getters};
use itertools::Itertools;
use rand_core::RngCore;

use crate::core::aabb::{Aabb, Bounded};
use crate::core::intersect::ObjectIntersection;
use crate::core::interval::Interval;
use crate::core::ray::Ray;
use crate::core::types::Number;
use crate::object::{Object, ObjectInstance, ObjectToken};
use crate::scene::Scene;

/// A group of objects that are rendered as one object
#[derive(Clone, Debug, Getters, CopyGetters)]
pub struct ListObject {
    #[get_copy = "pub"]
    aabb: Aabb,
    #[get = "pub"]
    tokens: Vec<ObjectToken>,
}

// region Constructors

impl ListObject {
    /// Creates a list of objects that have already been inserted into the scene
    pub fn new_from(scene: &Scene, tokens: impl IntoIterator<Item: Into<ObjectToken>>) -> Self {
        let tokens = tokens.into_iter().map(Into::into).collect_vec();
        let aabb = Aabb::encompass_iter(tokens.iter().map(|t| scene.get_obj(t).aabb()));
        Self { aabb, tokens }
    }

    /// Creates a list of objects, adding them to the scene
    pub fn new_in(scene: &mut Scene, objects: impl IntoIterator<Item: Into<ObjectInstance>>) -> Self {
        let objects = objects.into_iter().map(Into::into).collect_vec();
        let aabb = Aabb::encompass_iter(objects.iter().map(Bounded::aabb));
        let tokens = Vec::from_iter(objects.into_iter().map(|m| scene.add_obj(m)));
        Self { aabb, tokens }
    }
}

// endregion Constructors

// region Object Impl
impl Bounded for ListObject {
    fn aabb(&self) -> Aabb { self.aabb }
}

impl Object for ListObject {
    fn intersect(
        &self,
        scene: &Scene,
        ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut dyn RngCore,
    ) -> Option<ObjectIntersection> {
        self.tokens
            .iter()
            .map(|t| scene.get_obj(t))
            .filter_map(|o| o.intersect(scene, ray, interval, rng))
            .min()
    }
}

// endregion Object Impl
