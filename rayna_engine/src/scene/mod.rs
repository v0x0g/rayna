use crate::material::Material;
use crate::object::Object;
use crate::shared::camera::Camera;
use crate::skybox::{Skybox, SkyboxInstance};

pub mod stored;

#[derive(Clone, Debug)]
pub struct Scene<Mesh, Mat, Obj, Sky = SkyboxInstance>
where
    Mesh: crate::mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh, Mat> + Clone,
    Sky: Skybox + Clone,
{
    pub objects: Obj,
    pub skybox: Sky,
    pub camera: Camera,
}
