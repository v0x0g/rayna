use crate::shared::camera::Camera;
use crate::skybox::SkyboxInstance;
use object_list::ObjectList;

pub mod object_list;
pub mod stored;

#[derive(Clone, Debug)]
pub struct Scene {
    pub objects: ObjectList,
    pub camera: Camera,
    pub skybox: SkyboxInstance,
}
