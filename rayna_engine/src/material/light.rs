use crate::core::types::{Colour, Vector3};
use crate::material::Material;
use crate::scene::Scene;
use crate::shared::intersect::MeshIntersection;
use crate::shared::ray::Ray;
use crate::texture::{Texture, TextureToken};
use rand_core::RngCore;

/// A simple emissive material for turning an mesh into a light.
///
/// Does not scatter.
#[derive(Copy, Clone, Debug)]
pub struct LightMaterial {
    pub emissive: TextureToken,
}

impl Material for LightMaterial {
    fn scatter(
        &self,
        _ray: &Ray,
        _scene: &Scene,
        _intersection: &MeshIntersection,
        _rng: &mut dyn RngCore,
    ) -> Option<Vector3> {
        None
    }

    fn emitted_light(
        &self,
        _ray: &Ray,
        _scene: &Scene,
        intersection: &MeshIntersection,
        rng: &mut dyn RngCore,
    ) -> Colour {
        scene.get_tex(self.emissive).value(intersection, rng)
    }

    fn reflected_light(
        &self,
        _ray: &Ray,
        _scene: &Scene,
        _intersection: &MeshIntersection,
        _future_ray: &Ray,
        _future_col: &Colour,
        _rng: &mut dyn RngCore,
    ) -> Colour {
        Colour::BLACK
    }
}
