use crate::accel::bvh::Bvh;
use crate::object::ObjectType;
use getset::Getters;

#[derive(Clone, Debug, Getters)]
pub struct ObjectList {
    objects: Vec<ObjectType>,
    objects_bvh: Bvh,
}
