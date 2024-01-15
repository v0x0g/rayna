use crate::shared::camera::Camera;
use crate::skybox::{Skybox, SkyboxInstance};

pub mod stored;

#[derive(Clone, Debug)]
pub struct Scene<Sky: Skybox + Clone = SkyboxInstance> {
    pub objects: SceneObjectList,
    pub skybox: Sky,
    pub camera: Camera,
}
