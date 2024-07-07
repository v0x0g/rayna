# Rayna

## Branches

### Overview
This branch (`refactor/flatten_scene`) is 

The aim is to refactor the scene into a flattened heirarchy, so that there are no objects/meshes/materials all nested inside each other. Instead, they will form a tree, with indices being used to reference objects, instead of containing the object themselves. E.g. an object `obj` that references mesh `mesh` will have a _token_ that represents the mesh, instead of directly holding the mesh inside the `struct`. 

This will allow multiple objects to share the same `Mesh` object by referencing the same token, without duplicating the actual mesh itself.

Generics should also be removed, and all objects should only ever use `MeshInstance`, `MaterialInstance` etc (so `Scene` would no longer be generic).


### Example usage


Creating scenes
```rust
struct Scene {
    objects: Map<ObjectToken, ObjectInstance>,
    meshes: Map<MeshToken, MeshInstance>,
    materials: Map<MaterialToken, MaterialInstance>,
    textures: Map<TextureToken, TextureInstance>
}

let scene = Scene::new();

// Using tokens, instead of references or nested structs
let tex_tok  = scene.insert_tex(Colour::RED);
let mat_tok  = scene.insert_mat(LambertianMaterial::new(tex_tok));
let mesh_tok = scene.insert_mesh(SphereMesh::new(Point3::ZERO, 1.0));
let obj_tok  = scene.insert_obj(SimpleObject::new(mesh_tok, mat_tok));
```
Objects, Meshes, etc
```rust
struct LambertianMaterial { tex: TextureToken, } // Old, nested
struct LambertianMaterial<T: Texture> { tex: T } // New, token
```

Implementing traits
```rust
// Need to add a way to get the `&MaterialInstance` ref from a token
// So add some helper method to scene
impl Scene {
    pub fn get_mat<'mat>(&'mat self, mat_tok: MaterialToken) ->  &'mat MaterialInstance {
        &self.materials[mat_tok]
    }
}

// Maybe we can use the index trait?
impl Index<MaterialToken> for Scene {
    type Output = MaterialInstance;
    fn index(&self, mat_tok: MaterialToken) -> &Self::Output { self.get_mat(mat_tok) }
}
impl Index<MeshToken> for Scene {
    type Output = MeshInstance;
    fn index(&self, mesh_tok: MeshToken) -> &Self::Output { self.get_mesh(mesh_tok) }
}
```
