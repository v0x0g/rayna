use crate::shared::math;
use crate::shared::ray::Ray;
use dyn_clone::DynClone;
use rayna_shared::def::types::Pix;
use std::fmt::Debug;

pub trait Skybox: DynClone + Debug + Send + Sync {
    fn sky_colour(&self, ray: Ray) -> Pix;
}

// NOTE: We have to use [`DynClone`] instead of plain old [`Clone`],
// Since we will be using `Box<dyn Object>` and we need to clone those boxes
dyn_clone::clone_trait_object!(Skybox);

#[derive(Copy, Clone, Debug)]
pub struct DefaultSkybox;
impl Skybox for DefaultSkybox {
    fn sky_colour(&self, ray: Ray) -> Pix {
        let a = (0.5 * ray.dir().y) + 0.5;

        let white = Pix::from([1., 1., 1.]);
        let blue = Pix::from([0.5, 0.7, 1.]);

        math::lerp(white, blue, a)
    }
}
