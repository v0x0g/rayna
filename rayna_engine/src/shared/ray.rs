use crate::def::types::{Num, Vec3};
use puffin::profile_function;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Ray {
    pos: Vec3,
    dir: Vec3,
}

impl Ray {
    /// World-space coordinate of the ray
    #[inline(always)]
    pub fn pos(&self) -> Vec3 {
        self.pos
    }

    /// Direction vector of the ray.
    ///
    /// # Requirements
    ///     Must be normalised
    #[inline(always)]
    pub fn dir(&self) -> Vec3 {
        self.dir
    }

    pub fn new(pos: Vec3, dir: Vec3) -> Self {
        profile_function!();

        Self {
            pos,
            dir: dir.normalize(),
        }
    }

    /// Creates a new ray, without normalising the direction vector
    ///
    /// # Safety
    /// Unsafe as it does not normalise the direction, assuming the caller
    /// provided a correct vector, possibly breaking the invariant of a normalised direction
    pub unsafe fn new_unchecked(pos: Vec3, dir: Vec3) -> Self {
        profile_function!();
        Self { pos, dir }
    }

    /// Gets the position at a given distance along the ray
    ///
    /// `pos + (t * dir)`
    pub fn at(&self, t: Num) -> Vec3 {
        profile_function!();
        self.pos + (self.dir * t)
    }
}