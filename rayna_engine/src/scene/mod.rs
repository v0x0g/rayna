pub mod camera;
pub mod preset;

/// Represents the environment, containing the objects in a scene along with the skybox
///
/// # Notes:
/// Only one object type `Obj` is stored, because it is expected that it will be some sort
/// of 'group' object, such as a [`crate::object::bvh::BvhObject`], which groups multiple
/// sub-objects into one
#[derive(Clone, Debug)]
pub struct Scene<Obj, Sky> {
    pub objects: Obj,
    pub skybox: Sky,
}

/// Standard definition of [`Scene`], with all the default type parameters that are commonly used
/// This is the specific form of [`Scene`] you want, almost all of the time.
pub type StandardScene = Scene<
    crate::object::ObjectInstance<
        crate::mesh::MeshInstance,
        crate::material::MaterialInstance<crate::texture::TextureInstance>,
    >,
    crate::skybox::SkyboxInstance,
>;
