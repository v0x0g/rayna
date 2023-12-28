use crate::accel::bvh::Bvh;
use crate::object::ObjectType;
use getset::Getters;
use itertools::Itertools;

#[derive(Clone, Debug, Getters)]
#[get = "pub"]
pub struct ObjectList {
    raw: Vec<ObjectType>,
    bvh: Bvh,
}

// Iter<Into<ObjType>> => ObjectList
impl<Obj: Into<ObjectType>, Iter: IntoIterator<Item = Obj>> From<Iter> for ObjectList {
    fn from(value: Iter) -> Self {
        let raw = value.into_iter().map(Into::into).collect_vec();
        let bvh = Bvh {};
        Self { raw, bvh }
    }
}

// impl Object for ObjectList {
//     fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {}
//
//     fn intersect_all(&self, ray: &Ray) -> Option<Box<dyn Iterator<Item = Intersection> + '_>> {}
//
//     fn bounding_box(&self) -> &Aabb {}
// }
