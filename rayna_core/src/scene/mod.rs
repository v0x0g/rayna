use crate::obj::Object;

#[derive(Clone, Debug)]
pub struct Scene {
    // TODO: Maybe use [std::boxed::ThinBox] instead of [Box], might be better for perf
    pub objects: Vec<Box<dyn Object>>,
}

#[macro_export]
macro_rules! scene {
    [
        $(
            $value:expr
        ),* $(,)?
    ] => {{
            let mut objs = vec![$($value),*];
            $crate::scene::Scene{
                objects: objs
            }
        }};
}
