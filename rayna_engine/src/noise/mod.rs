use crate::core::token::generate_component_token;
use crate::core::types::Number;

pub mod boxed;

#[enum_dispatch::enum_dispatch]
pub trait Noise<const D: usize>: crate::core::component::Component {
    fn value(&self, coords: &[Number; D]) -> Number;
}

#[derive(Clone, Debug)]
#[enum_dispatch::enum_dispatch(Noise<D>)]
pub enum NoiseInstance<const D: usize> {
    BoxedNoise(self::boxed::BoxedNoise<D>),
}

generate_component_token!(NoiseToken < {const N: usize} as {N} > for NoiseInstance);
