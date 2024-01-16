use getset::Getters;
use crate::accel::bvh::Bvh;
use crate::mesh::{Object, ObjectInstance};

/// A group of objects that are rendered as one mesh
///
/// # Notes
/// Since this only implements [Object], and not [crate::scene::FullObject], all the sub-objects
/// will share the same material (once placed inside a [crate::scene::SceneObject]
#[derive(Clone, Debug, Getters)]
#[get = "pub"]
pub struct GroupObject<Obj: Object + Clone = ObjectInstance> {
    unbounded: Vec<Obj>
    bounded: Bvh<>
}

impl Object for GroupObject {

}
