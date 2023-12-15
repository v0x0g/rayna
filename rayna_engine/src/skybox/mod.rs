use crate::shared::math;
use crate::shared::ray::Ray;
use dyn_clone::DynClone;
use rayna_shared::def::types::Pixel;
use std::fmt::Debug;

pub trait Skybox: DynClone + Debug + Send + Sync {
    fn sky_colour(&self, ray: Ray) -> Pixel;
}

// NOTE: We have to use [`DynClone`] instead of plain old [`Clone`],
// Since we will be using `Box<dyn Object>` and we need to clone those boxes
dyn_clone::clone_trait_object!(Skybox);

#[derive(Copy, Clone, Debug)]
pub struct DefaultSkybox;
impl Skybox for DefaultSkybox {
    fn sky_colour(&self, ray: Ray) -> Pixel {
        let a = (0.5 * ray.dir().y) + 0.5;

        let white = Pixel::from([1., 1., 1.]);
        let blue = Pixel::from([0.5, 0.7, 1.]);

        math::lerp(white, blue, a)
    }
}
