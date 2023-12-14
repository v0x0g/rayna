use rayna_shared::def::types::{Number, Vector};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Ray {
    pos: Vector,
    dir: Vector,
}

impl Ray {
    /// World-space coordinate of the ray
    #[inline(always)]
    pub fn pos(&self) -> Vector {
        self.pos
    }

    /// Direction vector of the ray.
    ///
    /// # Requirements
    ///     Must be normalised
    #[inline(always)]
    pub fn dir(&self) -> Vector {
        self.dir
    }

    pub fn new(pos: Vector, dir: Vector) -> Self {
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
    pub unsafe fn new_unchecked(pos: Vector, dir: Vector) -> Self {
        Self { pos, dir }
    }

    /// Gets the position at a given distance along the ray
    ///
    /// `pos + (t * dir)`
    pub fn at(&self, t: Number) -> Vector {
        self.pos + (self.dir * t)
    }
}
