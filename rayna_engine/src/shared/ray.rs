use getset::CopyGetters;
use rayna_shared::def::types::{Number, Point3, Vector3};

#[derive(Copy, Clone, PartialEq, Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Ray {
    pos: Point3,
    dir: Vector3,
    inv_dir: Vector3,
}

impl Ray {
    // PERF: This function is pretty slow :(
    pub fn new(pos: Point3, dir: Vector3) -> Self {
        let dir = dir.normalize();
        Self {
            pos,
            dir,
            inv_dir: dir.recip(),
        }
    }

    /// Creates a new ray, without normalising the direction vector
    ///
    /// # Safety
    /// Unsafe as it does not normalise the direction, assuming the caller
    /// provided a correct vector, possibly breaking the invariant of a normalised direction
    pub unsafe fn new_unchecked(pos: Point3, dir: Vector3) -> Self {
        Self {
            pos,
            dir,
            inv_dir: dir.recip(),
        }
    }

    /// Gets the position at a given distance along the ray
    ///
    /// `pos + (t * dir)`
    pub fn at(&self, t: Number) -> Point3 {
        self.pos + (self.dir * t)
    }
}
