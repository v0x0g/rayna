use crate::core::types::{Number, Point3, Vector3};
use crate::shared::validate;
use getset::CopyGetters;

#[derive(Copy, Clone, PartialEq, Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Ray {
    pos: Point3,
    dir: Vector3,
    inv_dir: Vector3,
}

impl Ray {
    // FIXME: This function is pretty slow :(
    pub fn new(pos: impl Into<Point3>, dir: impl Into<Vector3>) -> Self {
        let (pos, dir) = (pos.into(), dir.into());
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
    /// provided a correct vector, possibly breaking the invariant of a normalised direction.
    /// This does still validate the vector, using to the [validate] module (and hence does nothing in release)
    pub unsafe fn new_unchecked(pos: impl Into<Point3>, dir: impl Into<Vector3>) -> Self {
        let (pos, dir) = (pos.into(), dir.into());
        validate::normal3(dir);
        Self {
            pos,
            dir,
            inv_dir: dir.recip(),
        }
    }

    /// Gets the position at a given distance along the ray
    ///
    /// `pos + (t * dir)`
    pub fn at(&self, t: Number) -> Point3 { self.pos + (self.dir * t) }
}
// TODO: Impl Into<Point3>
/// Destructure ray into position and direction
impl From<Ray> for (Point3, Vector3) {
    fn from(value: Ray) -> Self { (value.pos, value.dir) }
}
impl From<&Ray> for (Point3, Vector3) {
    fn from(value: &Ray) -> Self { (value.pos, value.dir) }
}
