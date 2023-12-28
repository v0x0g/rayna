use crate::object::ObjectType;
use crate::shared::camera::Camera;
use crate::skybox::SkyboxType;

pub mod stored;

#[derive(Clone, Debug)]
pub struct SceneBuilder {
    pub objects: Vec<ObjectType>,
    pub camera: Camera,
    pub skybox: SkyboxType,
}

#[derive(Clone, Debug)]
pub struct Scene {}
