use crate::shared::Vec3;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Ray {
    /// World-space coordinate of the ray
    pub pos: Vec3,
    /// Direction vector of the ray.
    ///
    /// # Requirements
    ///     Must be normalised
    pub dir: Vec3
}