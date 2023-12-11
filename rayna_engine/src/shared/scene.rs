use crate::obj::Object;
use crate::shared::camera::Camera;

#[derive(Clone, Debug)]
pub struct Scene {
    // TODO: Maybe use [std::boxed::ThinBox] instead of [Box], might be better for perf
    pub objects: Vec<Box<dyn Object>>,
    pub camera: Camera,
}

#[macro_export]
macro_rules! scene {
    {
        camera: $cam:expr,
        objects: [ $(
                $value:expr
        ),* $(,)? ]
    } => {
            $crate::shared::scene::Scene {
                camera: $cam,
                objects: vec![$(
                    std::boxed::Box::new($value) as std::boxed::Box<dyn $crate::obj::Object>
                ),*]
            }
        };
}
