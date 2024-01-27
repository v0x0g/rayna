use crate::core::types::{Colour, Image, Number};
use crate::mesh::primitive::sphere;
use crate::shared::ray::Ray;
use crate::skybox::Skybox;

/// A skybox that uses a **High Dynamic Range Image** (**HDRI**) as the skybox
#[derive(Clone, Debug)]
pub struct HdrImageSkybox {
    pub image: Image,
}

impl From<Image> for HdrImageSkybox {
    fn from(image: Image) -> Self { Self { image } }
}

impl Skybox for HdrImageSkybox {
    fn sky_colour(&self, ray: &Ray) -> Colour {
        // Kinda cheating here, using the `sphere_uv()` function
        // Since `ray.dir` is a unit vector, which is also a point on a sphere with `radius: 1.0`
        let (u, v) = sphere::sphere_uv(ray.dir()).into();

        let i = u * self.image.width() as Number;
        let j = (1. - v) * self.image.height() as Number;
        self.image.get_bilinear(i, j)
    }
}
