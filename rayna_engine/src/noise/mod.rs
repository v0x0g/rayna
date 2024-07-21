use crate::core::types::Number;
use crate::shared::token::generate_component_token;

pub mod boxed;

#[enum_dispatch::enum_dispatch]
#[doc(notable_trait)]
pub trait Noise<const D: usize>: crate::shared::ComponentRequirements {
    fn value(&self, coords: &[Number; D]) -> Number;
}

#[derive(Clone, Debug)]
#[enum_dispatch::enum_dispatch(Noise<D>)]
pub enum NoiseInstance<const D: usize> {}

generate_component_token!(NoiseToken for NoiseInstance);
