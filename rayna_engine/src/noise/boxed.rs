use crate::core::types::Number;
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct BoxedNoise<const D: usize> {
    pub inner: std::sync::Arc<dyn noise::NoiseFn<Number, D> + Sync + Send>,
}

impl<const N: usize> Debug for BoxedNoise<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { f.debug_struct("BoxedNoise").finish_non_exhaustive() }
}

impl<const D: usize> super::Noise<D> for BoxedNoise<D> {
    fn value(&self, coords: &[Number; D]) -> Number { self.inner.get(*coords) }
}
