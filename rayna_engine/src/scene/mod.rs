pub mod camera;
pub mod preset;

#[derive(Clone, Debug)]
pub struct Scene<Obj, Sky> {
    pub objects: Obj,
    pub skybox: Sky,
}

pub type SimpleScene = Scene<
    crate::object::ObjectInstance<
        crate::mesh::MeshInstance,
        crate::material::MaterialInstance<crate::texture::TextureInstance>,
    >,
    crate::skybox::SkyboxInstance,
>;
