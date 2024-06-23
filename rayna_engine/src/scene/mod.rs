use crate::material::MaterialInstance;
use crate::mesh::MeshInstance;
use crate::object::ObjectInstance;
use crate::scene::camera::Camera;
use crate::skybox::SkyboxInstance;
use crate::texture::TextureInstance;

pub mod camera;
pub mod stored;

#[derive(Clone, Debug)]
pub struct Scene<Obj = ObjectInstance<MeshInstance, MaterialInstance<TextureInstance>>, Sky = SkyboxInstance> {
    pub objects: Obj,
    pub skybox: Sky,
    pub camera: Camera,
}
