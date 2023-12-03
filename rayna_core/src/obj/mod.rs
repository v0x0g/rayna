use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;

pub mod sphere;

pub trait Object {
    /// Attempts to perform an intersection between the given ray and the target object
    fn intersect<I: Iterator<Item=Intersection>>(&self, ray: Ray) -> Option<I>;
}