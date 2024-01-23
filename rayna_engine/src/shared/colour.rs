use itertools::Itertools;
use rayna_shared::def::types::Number;
use std::array;
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
    #[inline]
    pub fn map(&self, op: impl Fn(Number) -> Number) -> Self { self.0.map(op).into() }
    #[inline]
    pub fn map2(&self, other: &Self, mut op: impl FnMut(Number, Number) -> Number) -> Self {
        array::from_fn(|i| op(self[i], other[i])).into()
    }
    #[inline]
    pub fn map_mut(&mut self, op: impl Fn(&mut Number)) { self.0.iter_mut().for_each(op) }
    #[inline]
    pub fn map2_mut(&mut self, other: &Self, mut op: impl FnMut(&mut Number, Number)) {
        self.0.iter_mut().zip_eq(other.0).for_each(|(s, o)| op(s, o))
    }
}

/// Helper macro to provide implementations of operator traits
///
/// The function should take in an owned `Self`-type reference.
///
/// I would use the [auto_ops]/[impl_ops] crates, but they don't support const generics, so roll my own
macro_rules! impl_op {
    (impl $($operator:ident)::+ : fn $fn_name:ident ($a:ident, $b:ident) $body:block) => {
        impl_op!(@inner impl $($operator)::+ : fn $fn_name ($a:  Colour<N>, $b:  Colour<N>) -> Colour<N> $body);
        impl_op!(@inner impl $($operator)::+ : fn $fn_name ($a:  Colour<N>, $b: &Colour<N>) -> Colour<N> $body);
        impl_op!(@inner impl $($operator)::+ : fn $fn_name ($a: &Colour<N>, $b:  Colour<N>) -> Colour<N> $body);
        impl_op!(@inner impl $($operator)::+ : fn $fn_name ($a: &Colour<N>, $b: &Colour<N>) -> Colour<N> $body);
    };

    (@inner impl $($operator:ident)::+ : fn $fn_name:ident ($a:ident: $lhs:ty, $b:ident : $rhs:ty) -> $out:ty $body:block) => {
        impl<const N: usize> $($operator)::+<$rhs> for $lhs {
            type Output = $out;

            fn $fn_name(self, rhs: $rhs) -> Self::Output {
                // Cloning is the easiest way to ensure that we get a owned value, from either a reference or owned val
                let ($a, $b) = (self.clone(), rhs.clone());
                $body
            }
        }
    };
}

/// See [impl_op]
macro_rules! impl_op_assign {
    (impl $($operator:ident)::+ : fn $fn_name:ident ($a:ident, $b:ident) $body:block) => {
        impl_op_assign!(@inner impl $($operator)::+ : fn $fn_name ($a:  Colour<N>, $b:  Colour<N>) $body);
        impl_op_assign!(@inner impl $($operator)::+ : fn $fn_name ($a:  Colour<N>, $b: &Colour<N>) $body);
    };

    (@inner impl $($operator:ident)::+ : fn $fn_name:ident ($a:ident: $lhs:ty, $b:ident : $rhs:ty) $body:block) => {
        impl<const N: usize> $($operator)::+<$rhs> for $lhs {
            fn $fn_name(&mut self, rhs: $rhs) {
                // Cloning is the easiest way to ensure that we get a owned value, from either a reference or owned val
                let (mut $a, $b) = (self.clone(), rhs.clone());
                $body;
                *self = $a;
            }
        }
    };
}

impl_op!(impl core::ops::Add : fn add(a, b) { Colour::map2(&a, &b, Number::add) });
impl_op!(impl core::ops::Sub : fn sub(a, b) { Colour::map2(&a, &b, Number::sub) });
impl_op!(impl core::ops::Mul : fn mul(a, b) { Colour::map2(&a, &b, Number::mul) });
impl_op!(impl core::ops::Div : fn div(a, b) { Colour::map2(&a, &b, Number::div) });

impl_op_assign!(impl core::ops::AddAssign : fn add_assign(a, b) { Colour::map2_mut(&mut a, &b, Number::add_assign) });
impl_op_assign!(impl core::ops::SubAssign : fn sub_assign(a, b) { Colour::map2_mut(&mut a, &b, Number::sub_assign) });
impl_op_assign!(impl core::ops::MulAssign : fn mul_assign(a, b) { Colour::map2_mut(&mut a, &b, Number::mul_assign) });
impl_op_assign!(impl core::ops::DivAssign : fn div_assign(a, b) { Colour::map2_mut(&mut a, &b, Number::div_assign) });

// endregion
