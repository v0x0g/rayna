use rayna_shared::def::types::{Number, Point3, Vector3};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Ray {
    pos: Point3,
    dir: Vector3,
}

impl Ray {
    /// World-space coordinate of the ray
    #[inline(always)]
    pub fn pos(&self) -> Point3 {
        self.pos
    }

    /// Direction vector of the ray.
    ///
    /// # Requirements
    ///     Must be normalised
    #[inline(always)]
    pub fn dir(&self) -> Vector3 {
        self.dir
    }

    pub fn new(pos: Point3, dir: Vector3) -> Self {
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
    pub unsafe fn new_unchecked(pos: Point3, dir: Vector3) -> Self {
        Self { pos, dir }
    }

    /// Gets the position at a given distance along the ray
    ///
    /// `pos + (t * dir)`
    pub fn at(&self, t: Number) -> Point3 {
        self.pos + (self.dir * t)
    }
}
