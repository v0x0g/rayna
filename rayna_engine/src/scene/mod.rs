use crate::core::intersect::ObjectIntersection;
use crate::core::interval::Interval;
use crate::core::ray::Ray;
use crate::core::token::IdToken;
use crate::core::types::Number;
use crate::material::{MaterialInstance, MaterialToken};
use crate::mesh::{MeshInstance, MeshToken};
use crate::noise::{NoiseInstance, NoiseToken};
use crate::object::{Object as _, ObjectInstance, ObjectToken};
use crate::skybox::SkyboxInstance;
use crate::texture::{TextureInstance, TextureToken};
use rand_core::RngCore;
use std::collections::HashMap;

pub mod camera;
pub mod preset;

/// Represents the environment, containing the objects in a scene along with the skybox.
///
/// # Tokens
///
/// To make the hierarchy flatter and avoid duplication, objects don't store each other
/// directly, instead they store references in the form of [tokens][`IdToken`]. This has two benefits:
///
/// 1. Objects can abstract away the specifics of their meshes etc., simplifying them
/// 2. The same material or texture can be easily reused many times, by simply copying the token
/// (previously the entire material would need to be duplicated).
///
/// TODO: More docs, and some proper doc tests
///
///
/// # Examples
///
/// ## API Comparison
///
/// ```ignore // Old API won't compile anyway
/// # use rayna_engine::core::types::{Colour, Point3};
/// # use rayna_engine::material::lambertian::LambertianMaterial;
/// # use rayna_engine::mesh::primitive::sphere::SphereMesh;
/// # use rayna_engine::object::simple::SimpleObject;
/// # use rayna_engine::scene::Scene;
/// # use rayna_engine::texture::TextureInstance;
/// # let scene = Scene::new();
/// // Old Version:
/// // Note the duplicated material etc
/// SimpleObject::new(
///     SphereMesh::new( [0., 0., -1.], 1.0 ),
///     LambertianMaterial {
///         albedo: TextureInstance::from(Colour::RED)
///     },
///     None
/// );
/// SimpleObject::new(
///     SphereMesh::new( [1., 1., 0.], 1.0 ),
///     LambertianMaterial {
///         albedo: TextureInstance::from(Colour::RED)
///     },
///     None
/// );
/// ```
///
/// ```
/// # use rayna_engine::core::types::{Colour, Point3};
/// # use rayna_engine::material::lambertian::LambertianMaterial;
/// # use rayna_engine::mesh::primitive::sphere::SphereMesh;
/// # use rayna_engine::object::simple::SimpleObject;
/// # use rayna_engine::scene::Scene;
/// # use rayna_engine::texture::TextureInstance;
/// # let scene = Scene::new();
/// // New Version:
/// // Note we just reuse the same material token instead
/// let tex_tok  = scene.add_tex(Colour::RED);
/// let mat_tok  = scene.add_mat(LambertianMaterial::new(tex_tok));
/// let mesh1_tok = scene.add_mesh(SphereMesh::new([0., 0., -1.], 1.0));
/// let obj1_tok  = scene.add_obj(SimpleObject::new(mesh1_tok, mat_tok, None));
/// let mesh2_tok = scene.add_mesh(SphereMesh::new([1., 1., 0.], 1.0));
/// let obj2_tok  = scene.add_obj(SimpleObject::new(mesh2_tok, mat_tok, None));
/// ```
///
/// ## Limitations
///
/// The following example will not compile, giving error `E0499`. This is not a limitation of the API,
/// but the compiler's borrow analysis.
/// The compiler is unable to reason that the calls to `scene.add_xxx()` immediately drop the reference on returning
/// and so a temporary explicit variable is needed.
///
/// See examples in discussions [here](https://users.rust-lang.org/t/error-e0499-cannot-borrow-self-as-mutable-more-than-once-at-a-time/46006/2) and [here](https://internals.rust-lang.org/t/why-compiler-reqires-explicit-vars-to-resolve-cannot-borrow-more-than-once-in-this-case/19720)
///
/// ```compile_fail
/// # use rayna_engine::core::types::{Colour, Point3};
/// # use rayna_engine::material::lambertian::LambertianMaterial;
/// # use rayna_engine::mesh::primitive::sphere::SphereMesh;
/// # use rayna_engine::object::simple::SimpleObject;
/// # use rayna_engine::scene::Scene;
/// # let scene = Scene::new();
/// scene.add_obj(SimpleObject::new_from(
///     &scene,
///     scene.add_mesh(SphereMesh::new((0., -0.3, 0.), 0.1)),
///     scene.add_mat(LambertianMaterial::new(scene.add_tex(Colour::WHITE))),
///     None,
/// ));
/// ```
#[derive(Clone, Debug)]
pub struct Scene {
    // TODO: See if there's a way to get rid of the duplicated token/insertion code,
    //  it might be possible using some fancy trait, and some `unsafe` trickery
    noise2d: HashMap<NoiseToken<2>, NoiseInstance<2>>,
    noise3d: HashMap<NoiseToken<3>, NoiseInstance<3>>,
    textures: HashMap<TextureToken, TextureInstance>,
    materials: HashMap<MaterialToken, MaterialInstance>,
    meshes: HashMap<MeshToken, MeshInstance>,
    objects: HashMap<ObjectToken, ObjectInstance>,
    custom_root: Option<ObjectInstance>,
    skybox: SkyboxInstance,
}

impl Scene {
    /// Generates a new [`IdToken`]
    ///
    /// The exact order of token generation is unspecified and an internal implementation detail
    fn new_token_id() -> IdToken {
        use rand::thread_rng;
        use rand_core::RngCore as _;
        use std::sync::atomic::{AtomicU32, Ordering};

        static COUNTER: AtomicU32 = AtomicU32::new(1);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed) as u64;
        let mask = thread_rng().next_u32() as u64;
        // Concat [mask][count] for a guaranteed unique ID
        IdToken::from((mask << (u32::BITS)) | count)
    }

    pub fn new() -> Self {
        Self {
            noise2d: HashMap::new(),
            noise3d: HashMap::new(),
            textures: HashMap::new(),
            materials: HashMap::new(),
            meshes: HashMap::new(),
            objects: HashMap::new(),
            custom_root: None,
            skybox: SkyboxInstance::default(),
        }
    }

    /// Sets the custom root object for this scene
    ///
    /// This overrides the default behaviour, so all intersection calls are passed through
    /// to the root object. The default is to render all objects present in the scene
    /// (see [`Self::all_obj`]
    pub fn set_custom_root(&mut self, obj: impl Into<ObjectInstance>) {
        self.custom_root = Some(obj.into());
    }
    /// Gets the object that was set as the custom scene root
    ///
    /// See [`Self::set_custom_root`] for an explanation of custom roots
    pub fn get_custom_root(&self) -> Option<&ObjectInstance> {
        self.custom_root.as_ref()
    }

    pub fn get_skybox(&self) -> &SkyboxInstance {
        &self.skybox
    }
    pub fn set_skybox(&mut self, skybox: impl Into<SkyboxInstance>) {
        self.skybox = skybox.into()
    }

    /// See [`Object::intersect()``]
    pub fn intersect(
        &self,
        ray: &Ray,
        interval: &Interval<Number>,
        rng: &mut impl RngCore,
    ) -> Result<ObjectIntersection, &SkyboxInstance> {
        // Use custom root if present, else iterate all objects
        match self.custom_root.as_ref() {
            Some(root) => root.intersect(&self, ray, interval, rng),
            None => self
                .objects
                .values()
                .filter_map(|o| o.intersect(&self, ray, interval, rng))
                .min(),
        }
        .ok_or(&self.skybox)
    }
}

/// A helper macro that generates functions for adding and removing components to a scene.
///
/// # Parameters
///
/// - `$ident`: Short name identifier for what type of entity we have, such as `mesh` or `obj`
/// - `$field`: The field in the [`Scene`] struct that holds the mapping
/// - `$inst`:  The type of the entity instance, e.g. [`MeshInstance`]
/// - `$tok`:   The type of token used for the entity, e.g. [`MeshToken`]
///
/// # Examples
///
/// ```
/// /// Add instances of [TextureInstance], which are referenced by a [TextureToken],
/// /// These are stored in `scene.textures`, and are modified by `add_tex()`, `get_tex()` etc
/// # struct __Doc;
/// rayna_engine::gen_components!{
///    ( tex in self.textures: TextureInstance => TextureToken  ),
/// }
/// ```
#[cfg_attr(doc, macro_export)]
macro_rules! gen_components {
    {$(
        ($ident:ident in self.$field_name:ident : $inst_type:ty => $token_type:ty $(,)?)
    ),* $(,)?} => {

impl Scene { $(paste::paste!(

    #[doc = concat!(
        "Internal method to generate a ", stringify!($token_type), " token"
    )]
    fn [<new_ $ident _token>]() -> $token_type {
        $token_type::from(Self::new_token_id())
    }

    #[doc = concat!(
        "Adds a ", stringify!(inst_type), " to the scene, returning a ", stringify!(token_type), " that can be used to\
        reference it in other components",
    )]
    pub fn [<add_ $ident>] <'l>(&'l mut self, value: impl Into<$inst_type>) -> $token_type {
        let tok = Self::[<new_ $ident _token>]();
        assert!(!self.$field_name.contains_key(&tok), "generated token was not unique");
        self.$field_name.insert(tok, value.into());
        tok
    }

    #[doc = concat!(
        "Uses a ", stringify!(token_type), " to obtain a reference to a ", stringify!($inst_type), ", panicking\
        if the token did not exist in the scene",
    )]
    pub fn [<get_ $ident>] (&self, tok: &$token_type) -> &$inst_type {
        self.[<try_get_ $ident>](tok)
            .expect(&format!("{} token {} did not exist", stringify!($inst_type), tok))
    }

    #[doc = concat!(
        "Uses a ", stringify!(token_type), " to obtain a reference to a ", stringify!(inst_type), ", returning [`None`]\
        if the token did not exist in the scene",
    )]
    pub fn [<try_get_ $ident>] (&self, tok: &$token_type) -> Option<&$inst_type> {
        self.$field_name.get(tok)
    }

    #[doc = concat!(
        "Returns an iterator over all the ", stringify!(ident), " components in the scene"
    )]
    pub fn [<all_ $ident>] (&self) -> impl Iterator<Item = (&$token_type, &$inst_type)> {
        self.$field_name.iter()
    }

);)*}

    };
}

gen_components! {
    ( noise2 in self.noise2d   : NoiseInstance<2> => NoiseToken::<2> ),
    ( noise3 in self.noise3d   : NoiseInstance<3> => NoiseToken::<3> ),
    ( tex    in self.textures  : TextureInstance  => TextureToken    ),
    ( mat    in self.materials : MaterialInstance => MaterialToken   ),
    ( mesh   in self.meshes    : MeshInstance     => MeshToken       ),
    ( obj    in self.objects   : ObjectInstance   => ObjectToken     ),
}
