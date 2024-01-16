use crate::material::{Material, MaterialInstance};
use crate::mesh::MeshInstance;
use crate::object::{Object, ObjectInstance};
use crate::shared::camera::Camera;
use crate::skybox::{Skybox, SkyboxInstance};
use std::marker::PhantomData;

pub mod stored;

#[derive(Clone, Debug)]
pub struct Scene<Mesh = MeshInstance, Mat = MaterialInstance, Obj = ObjectInstance<Mesh, Mat>, Sky = SkyboxInstance>
where
    Mesh: crate::mesh::Mesh + Clone,
    Mat: Material + Clone,
    Obj: Object<Mesh = Mesh, Mat = Mat> + Clone,
    Sky: Skybox + Clone,
{
    pub objects: Obj,
    pub skybox: Sky,
    pub camera: Camera,
    // Why won't the compiler figure out that Mesh and Mat are used by Object???
    pub phantom_mesh: PhantomData<Mesh>,
    pub phantom_mat: PhantomData<Mat>,
}
