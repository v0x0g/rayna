use crate::core::types::{Colour, Number, Vector3};
use crate::material::Material;
use crate::shared::intersect::MeshIntersection;
use crate::shared::ray::Ray;
use crate::shared::{math, rng};
use crate::texture::{Texture, TextureToken};

use crate::scene::Scene;
use rand::RngCore;

#[derive(Copy, Clone, Debug)]
pub struct MetalMaterial {
    pub albedo: TextureToken,
    pub fuzz: Number,
}

impl Material for MetalMaterial {
    fn scatter(
        &self,
        ray: &Ray,
        _scene: &Scene,
        intersection: &MeshIntersection,
        rng: &mut dyn RngCore,
    ) -> Option<Vector3> {
        let reflected = math::reflect(ray.dir(), intersection.ray_normal);
        let rand = rng::normal_on_unit_sphere(rng);

        // Generate some fuzzy reflections by adding a "cloud" of random points
        // around the reflection (a sphere with `radius=fuzz` centred at `reflected)
        let vec = reflected + (rand * self.fuzz);
        // This might end up scattering beneath the surface of the mesh, so check here
        let dot = Vector3::dot(vec, intersection.ray_normal);
        return if dot > 0. {
            // Scatter ok
            Some(vec.normalize())
        } else {
            // Scattered under surface
            None
        };
    }

    //noinspection DuplicatedCode
    fn reflected_light(
        &self,
        _ray: &Ray,
        scene: &Scene,
        intersect: &MeshIntersection,
        _future_ray: &Ray,
        future_col: &Colour,
        rng: &mut dyn RngCore,
    ) -> Colour {
        future_col * scene.get_tex(self.albedo).value(scene, intersect, rng)
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
