use crate::core::types::{Channel, Colour, Number, Point3, Vector3};
use crate::material::Material;
use crate::shared::intersect::MeshIntersection;
use crate::shared::ray::Ray;
use crate::shared::rng;
use crate::texture::{Texture, TextureToken};

use crate::scene::Scene;
use rand_core::RngCore;

/// A material that uniformly scatters rays in all directions
///
/// Normally this is paired with a [`crate::object::volumetric::VolumetricObject`]
#[derive(Copy, Clone, Debug)]
pub struct IsotropicMaterial {
    pub albedo: TextureToken,
    pub density: Number,
}

impl Default for IsotropicMaterial {
    fn default() -> Self {
        Self {
            albedo: [0.5; 3].into(),
            density: 1.,
        }
    }
}

impl Material for IsotropicMaterial {
    fn scatter(
        &self,
        _ray: &Ray,
        _scene: &Scene,
        _intersection: &MeshIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<Vector3> {
        Some(rng::normal_on_unit_sphere(rng))
    }
    //TODO: Take into account distance along travelled ray (beer's law?)
    fn reflected_light(
        &self,
        ray: &Ray,
        scene: &Scene,
        intersection: &MeshIntersection,
        _future_ray: &Ray,
        future_col: &Colour,
        rng: &mut dyn RngCore,
    ) -> Colour {
        // See [DielectricMaterial] for explanation of this

        let dist_inside = Point3::distance(intersection.pos_w, ray.pos());
        let transmission = (-self.density * dist_inside) as Channel;
        // NOTE: This is the colour at the exiting intersection, which might not be accurate if the texture
        //  is non-homogenous
        // TODO: Fix this texture issue somehow, maybe sample along the line and integrate that?
        let attenuation_col = scene.get_tex(self.albedo).value(scene, intersection, rng);

        // future_col * (attenuation_col.exp(transmission))
        future_col * attenuation_col * transmission.exp()
    }

    fn emitted_light(
        &self,
        _ray: &Ray,
        _scene: &Scene,
        _intersection: &MeshIntersection,
        _rng: &mut dyn RngCore,
    ) -> Colour {
        Colour::BLACK
    }
}
