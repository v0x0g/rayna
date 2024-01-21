use auto_ops::*;
use rayna_shared::def::types::Number;
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Colour<const N: usize = 3>([Number; N]);

impl<const N: usize> Colour<N> {
    /// How many channels there are, for this colour.
    /// RGB is 3 channels.
    pub const CHANNEL_COUNT: usize = N;
}

// region Constructors

impl<const N: usize> Colour<N> {
    pub fn new(val: impl Into<[Number; N]>) -> Self { Self(val.into()) }
}

// endregion Constructors

// region RGB Impl

impl From<(Number, Number, Number)> for Colour<3> {
    fn from(val: (Number, Number, Number)) -> Self { Self::new(val) }
}
impl From<Colour<3>> for (Number, Number, Number) {
    //noinspection RsLiveness - `r,g,b` are used
    fn from(Colour::<3> { 0: [r, g, b] }: Colour) -> Self { (r, g, b) }
}

// endregion RGB Impl

// region To/From impls

impl<const N: usize> From<[Number; N]> for Colour<N> {
    fn from(val: [Number; N]) -> Self { Self::new(val) }
}
impl<const N: usize> From<Colour<N>> for [Number; N] {
    //noinspection RsLiveness - `val` is used
    fn from(Colour::<N> { 0: val }: Colour<N>) -> Self { val }
}

// endregion To/From impls

// region Iterating/Indexing

impl<const N: usize> IntoIterator for Colour<N> {
    type Item = Number;
    type IntoIter = std::array::IntoIter<Number, N>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<const N: usize> Deref for Colour<N> {
    type Target = [Number; N];

    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<const N: usize> DerefMut for Colour<N> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<const N: usize> Index<usize> for Colour<N> {
    type Output = Number;

    fn index(&self, index: usize) -> &Self::Output { &self.0[index] }
}
impl<const N: usize> IndexMut<usize> for Colour<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output { &mut self.0[index] }
}

// endregion Iterating/Indexing

// region Operators

impl Colour {
    pub fn map(&self, op: impl FnMut(Number) -> Number) -> Self { self.0.map(op).into() }
    pub fn map2(&self, other: &Self, mut op: impl FnMut(Number, Number) -> Number) -> Self {
        // SAFETY: Both colours are constant arrays, so this is fine to `.unwrap()`
        std::array::from_fn(|i| op(self[i], other[i])).into()
    }
}

// TODO: Make this use the type parameter `N` on `Colour`
#[rustfmt::skip] impl_op_ex!(+ |a: &Colour, b: &Colour| -> Colour { Colour::map2(a, b, Number::add) });
#[rustfmt::skip] impl_op_ex!(- |a: &Colour, b: &Colour| -> Colour { Colour::map2(a, b, Number::sub) });
#[rustfmt::skip] impl_op_ex!(* |a: &Colour, b: &Colour| -> Colour { Colour::map2(a, b, Number::mul) });
#[rustfmt::skip] impl_op_ex!(/ |a: &Colour, b: &Colour| -> Colour { Colour::map2(a, b, Number::div) });

// endregion
