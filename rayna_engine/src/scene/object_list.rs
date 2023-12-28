use getset::Getters;
use itertools::Itertools;

use crate::accel::bvh::Bvh;
use crate::object::ObjectType;

#[derive(Clone, Debug, Getters)]
#[get = "pub"]
pub struct ObjectList {
    raw: Vec<ObjectType>,
    bvh: Bvh,
}

impl<Obj: Into<ObjectType>, Iter: IntoIterator<Item = Obj>> From<Iter> for ObjectList {
    fn from(value: Iter) -> Self {
        let raw = value.into_iter().map(Into::into).collect_vec();
        let bvh = Bvh {};
        Self { raw, bvh }
    }
}
