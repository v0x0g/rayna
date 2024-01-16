// use getset::Getters;
// use crate::mesh;
// use crate::shared::generic_bvh::GenericBvh;
//
// /// A group of meshes that are rendered as one mesh
// ///
// /// # Notes
// /// Since this only implements [Object], and not [crate::scene::FullObject], all the sub-objects
// /// will share the same material (once placed inside a [crate::scene::SceneObject]
// #[derive(Clone, Debug, Getters)]
// #[get = "pub"]
// pub struct GroupObject<Mesh: mesh::Mesh> {
//     unbounded: Vec<Mesh>
//     bounded: GenericBvh<Mesh>
// }
//
// impl Object for GroupObject {
//
// }
