use crate::material::MaterialInstance;
use crate::mesh::MeshInstance;
use crate::object::{Object, ObjectInstance};
use crate::shared::camera::Camera;
use crate::skybox::{Skybox, SkyboxInstance};
use crate::texture::TextureInstance;

pub mod stored;

#[derive(Clone, Debug)]
pub struct Scene<Obj = ObjectInstance<MeshInstance, MaterialInstance<TextureInstance>>, Sky = SkyboxInstance>
where
    Obj: Object + Clone,
    Sky: Skybox + Clone,
{
    pub objects: Obj,
    pub skybox: Sky,
    pub camera: Camera,
}
