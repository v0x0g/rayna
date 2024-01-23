use crate::shared::impl_utils::{impl_op, impl_op_assign};
use itertools::Itertools;
use rayna_shared::def::types::Number;
use std::array;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Colour<const N: usize>([Number; N]);

pub type ColourRgb = Colour<3>;

impl<const N: usize> Colour<N> {
    /// How many channels there are, for this colour.
    /// RGB is 3 channels.
    pub const CHANNEL_COUNT: usize = N;
}

// region Constructors

impl<const N: usize> Colour<N> {
    pub const fn new(val: [Number; N]) -> Self { Self(val) }
}

// endregion Constructors

// region RGB Impl

impl const From<(Number, Number, Number)> for ColourRgb {
    fn from(val: (Number, Number, Number)) -> Self { Self::new(val.into()) }
}
impl const From<ColourRgb> for (Number, Number, Number) {
    //noinspection RsLiveness - `r,g,b` are used
    fn from(ColourRgb { 0: [r, g, b] }: ColourRgb) -> Self { (r, g, b) }
}

//poop

// endregion RGB Impl

// region To/From impls

impl<const N: usize> const From<[Number; N]> for Colour<N> {
    fn from(val: [Number; N]) -> Self { Self::new(val) }
}
impl<const N: usize> const From<Colour<N>> for [Number; N] {
    //noinspection RsLiveness - `val` is used
    fn from(Colour::<N> { 0: val }: Colour<N>) -> Self { val }
}

// endregion To/From impls

// region Iterating/Indexing

impl<const N: usize> const IntoIterator for Colour<N> {
    type Item = Number;
    type IntoIter = array::IntoIter<Number, N>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<const N: usize> const Deref for Colour<N> {
    type Target = [Number; N];

    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<const N: usize> const DerefMut for Colour<N> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<const N: usize> const Index<usize> for Colour<N> {
    type Output = Number;

    fn index(&self, index: usize) -> &Self::Output { &self.0[index] }
}
impl<const N: usize> const IndexMut<usize> for Colour<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output { &mut self.0[index] }
}

// endregion Iterating/Indexing

// region Operators

impl<const N: usize> Colour<N> {
    /// Maps each element of the colour with the given closure, and returns the new colour
    #[inline]
    pub fn map(&self, op: impl Fn(Number) -> Number) -> Self { self.0.map(op).into() }
    /// Maps each element of the colour with the given closure, with the element of another, and returns the new colour.
    #[inline]
    pub fn map2(&self, other: &Self, mut op: impl FnMut(Number, Number) -> Number) -> Self {
        array::from_fn(|i| op(self[i], other[i])).into()
    }

    /// Same as [Self::map], but acts in_place
    #[inline]
    pub fn map_assign(&mut self, op: impl Fn(&mut Number)) { self.0.iter_mut().for_each(op) }
    /// Same as [Self::map2], but acts in_place
    #[inline]
    pub fn map2_assign(&mut self, other: &Self, mut op: impl FnMut(&mut Number, Number)) {
        self.0.iter_mut().zip_eq(other.0).for_each(|(s, o)| op(s, o))
    }
}

// Basic maths operators
impl_op!(impl {<const N: usize>} core::ops::Add : fn add(a: Colour<N>, b:Colour<N>) -> Colour<N> { Colour::map2(&a, &b, Number::add) });
impl_op!(impl {<const N: usize>} core::ops::Sub : fn sub(a: Colour<N>, b:Colour<N>) -> Colour<N> { Colour::map2(&a, &b, Number::sub) });
impl_op!(impl {<const N: usize>} core::ops::Mul : fn mul(a: Colour<N>, b:Colour<N>) -> Colour<N> { Colour::map2(&a, &b, Number::mul) });
impl_op!(impl {<const N: usize>} core::ops::Div : fn div(a: Colour<N>, b:Colour<N>) -> Colour<N> { Colour::map2(&a, &b, Number::div) });

// Shift left/right rotates the channels left/right by `n` places.
impl_op!(impl {<const N: usize>} core::ops::Shl : fn shl(col: Colour<N>, shift: usize) -> Colour<N> { col.0.rotate_left(shift); col });
impl_op!(impl {<const N: usize>} core::ops::Shr : fn shr(col: Colour<N>, shift: usize) -> Colour<N> { col.0.rotate_right(shift); col });

impl_op_assign!(impl {<const N: usize>} core::ops::AddAssign : fn add_assign(a: Colour<N>, b: Colour<N>) { Colour::map2_assign(&mut a, &b, Number::add_assign) });
impl_op_assign!(impl {<const N: usize>} core::ops::SubAssign : fn sub_assign(a: Colour<N>, b: Colour<N>) { Colour::map2_assign(&mut a, &b, Number::sub_assign) });
impl_op_assign!(impl {<const N: usize>} core::ops::MulAssign : fn mul_assign(a: Colour<N>, b: Colour<N>) { Colour::map2_assign(&mut a, &b, Number::mul_assign) });
impl_op_assign!(impl {<const N: usize>} core::ops::DivAssign : fn div_assign(a: Colour<N>, b: Colour<N>) { Colour::map2_assign(&mut a, &b, Number::div_assign) });

impl_op_assign!(impl {<const N: usize>} core::ops::ShlAssign : fn shl_assign(col: Colour<N>, shift: usize) { col.0.rotate_left(shift) });
impl_op_assign!(impl {<const N: usize>} core::ops::ShrAssign : fn shr_assign(col: Colour<N>, shift: usize) { col.0.rotate_right(shift) });

// endregion

impl<const N: usize> Hash for Colour<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.0.len());
        for c in self.0 {
            Number::to_ne_bytes(c).hash(state)
        }
    }
}
