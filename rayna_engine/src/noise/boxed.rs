use crate::core::types::Number;
use noise::NoiseFn;
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
pub struct BoxedNoise<const N: usize> {
    pub inner: Box<dyn NoiseFn<Number, N>>,
}

impl<const N: usize> Debug for BoxedNoise<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { f.debug_struct("BoxedNoise").finish_non_exhaustive() }
}
