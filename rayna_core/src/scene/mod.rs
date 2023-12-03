use crate::obj::Object;
use crate::shared::ray::Ray;

pub struct Scene {
    // TODO: Maybe use [std::boxed::ThinBox] instead of [Box], might be better for perf
    pub objects: Vec<Box<dyn Object>>
}

impl Scene{
    pub fn test(&self, ray: Ray) {
        self.objects.iter()
            .map(|o| o.intersect(ray))
            .for_each(|i| println!("{i:#?}"));

        ()
    }
}