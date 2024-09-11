use std::fmt::Debug;

/// A simple marker trait that enforces a few other traits we need
/// in the engine
// TODO: Add a requirement for `valuable::Valuable`
pub trait Component: Debug + Send + Sync {}
impl<T: Debug + Send + Sync> Component for T {}
