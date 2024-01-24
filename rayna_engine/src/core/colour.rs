use crate::core::types::Channel;
use crate::impl_op_assign;
use crate::{forward_fn, impl_op};
use itertools::Itertools;
use std::array;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut, Index, IndexMut};

// TODO: Make this generic over the channel type

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
#[repr(transparent)] // Ensure it's treated as a raw array, so we can transmute safely
pub struct Colour<const N: usize>(pub [Channel; N]);

pub type ColourRgb = Colour<3>;

impl<const N: usize> Colour<N> {
    /// How many channels there are, for this colour.
    /// RGB is 3 channels.
    pub const CHANNEL_COUNT: usize = N;
}

impl<const N: usize> const Default for Colour<N> {
    fn default() -> Self { Self::new([0.; N]) }
}

// region Constructors

impl<const N: usize> Colour<N> {
    pub const fn new(val: [Channel; N]) -> Self { Self(val) }
}

// endregion Constructors

// region RGB Impl

impl const From<(Channel, Channel, Channel)> for ColourRgb {
    fn from(val: (Channel, Channel, Channel)) -> Self { Self::new(val.into()) }
}
impl const From<ColourRgb> for (Channel, Channel, Channel) {
    //noinspection RsLiveness - `r,g,b` are used
    fn from(ColourRgb { 0: [r, g, b] }: ColourRgb) -> Self { (r, g, b) }
}

// endregion RGB Impl

// region Known Colours

impl<const N: usize> Colour<N> {
    pub const BLACK: Self = Self::new([0.; N]);
    pub const WHITE: Self = Self::new([1.; N]);
}
// endregion Known Colours

// region To/From impls

impl<const N: usize> const From<[Channel; N]> for Colour<N> {
    fn from(val: [Channel; N]) -> Self { Self::new(val) }
}
impl<const N: usize> const From<&[Channel]> for Colour<N> {
    /// Converts a slice reference into a colour
    ///
    /// # Panics
    /// Ensure that the slice is at least [N] elements long, otherwise this will cause an assertion failure
    fn from(val: &[Channel]) -> Self {
        assert!(
            val.len() >= N,
            "given slice reference was not long enough ({len} < {n})",
            len = val.len(),
            n = N
        );
        let val: [Channel; N] = val[..N].try_into().expect("couldn't convert slice");
        Self::new(val)
    }
}
impl<const N: usize> const From<Colour<N>> for [Channel; N] {
    //noinspection RsLiveness - `val` is used
    fn from(Colour::<N> { 0: val }: Colour<N>) -> Self { val }
}

// endregion To/From impls

// region Iterating/Indexing

impl<const N: usize> const IntoIterator for Colour<N> {
    type Item = Channel;
    type IntoIter = array::IntoIter<Channel, N>;

    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<const N: usize> const Deref for Colour<N> {
    type Target = [Channel; N];

    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<const N: usize> const DerefMut for Colour<N> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<const N: usize> const Index<usize> for Colour<N> {
    type Output = Channel;

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
    pub fn map(&self, op: impl Fn(Channel) -> Channel) -> Self { self.0.map(op).into() }
    /// Maps each element of the colour with the given closure, with the element of another, and returns the new colour.
    #[inline]
    pub fn map2(&self, other: &Self, mut op: impl FnMut(Channel, Channel) -> Channel) -> Self {
        array::from_fn(|i| op(self[i], other[i])).into()
    }

    /// Same as [Self::map], but acts in_place
    #[inline]
    pub fn map_assign(&mut self, op: impl Fn(&mut Channel)) { self.0.iter_mut().for_each(op) }
    /// Same as [Self::map2], but acts in_place
    #[inline]
    pub fn map2_assign(&mut self, other: &Self, mut op: impl FnMut(&mut Channel, Channel)) {
        self.0.iter_mut().zip_eq(other.0).for_each(|(s, o)| op(s, o))
    }
}

// Basic maths operators
impl_op!(impl {<const N: usize>} core::ops::Add : fn add(a: Colour<N>, b: Colour<N>) -> Colour<N> { Colour::map2(&a, &b, Channel::add) });
impl_op!(impl {<const N: usize>} core::ops::Sub : fn sub(a: Colour<N>, b: Colour<N>) -> Colour<N> { Colour::map2(&a, &b, Channel::sub) });
impl_op!(impl {<const N: usize>} core::ops::Mul : fn mul(a: Colour<N>, b: Colour<N>) -> Colour<N> { Colour::map2(&a, &b, Channel::mul) });
impl_op!(impl {<const N: usize>} core::ops::Div : fn div(a: Colour<N>, b: Colour<N>) -> Colour<N> { Colour::map2(&a, &b, Channel::div) });

impl_op!(impl {<const N: usize>} core::ops::Add : fn add(a: Colour<N>, b: Channel) -> Colour<N> { Colour::map2(&a, &[b; N].into(), Channel::add) });
impl_op!(impl {<const N: usize>} core::ops::Sub : fn sub(a: Colour<N>, b: Channel) -> Colour<N> { Colour::map2(&a, &[b; N].into(), Channel::sub) });
impl_op!(impl {<const N: usize>} core::ops::Mul : fn mul(a: Colour<N>, b: Channel) -> Colour<N> { Colour::map2(&a, &[b; N].into(), Channel::mul) });
impl_op!(impl {<const N: usize>} core::ops::Div : fn div(a: Colour<N>, b: Channel) -> Colour<N> { Colour::map2(&a, &[b; N].into(), Channel::div) });

impl_op_assign!(impl {<const N: usize>} core::ops::AddAssign : fn add_assign(a: Colour<N>, b: Colour<N>) { Colour::map2_assign(&mut a, &b, Channel::add_assign) });
impl_op_assign!(impl {<const N: usize>} core::ops::SubAssign : fn sub_assign(a: Colour<N>, b: Colour<N>) { Colour::map2_assign(&mut a, &b, Channel::sub_assign) });
impl_op_assign!(impl {<const N: usize>} core::ops::MulAssign : fn mul_assign(a: Colour<N>, b: Colour<N>) { Colour::map2_assign(&mut a, &b, Channel::mul_assign) });
impl_op_assign!(impl {<const N: usize>} core::ops::DivAssign : fn div_assign(a: Colour<N>, b: Colour<N>) { Colour::map2_assign(&mut a, &b, Channel::div_assign) });

impl_op_assign!(impl {<const N: usize>} core::ops::AddAssign : fn add_assign(a: Colour<N>, b: Channel) { Colour::map2_assign(&mut a, &[b; N].into(), Channel::add_assign) });
impl_op_assign!(impl {<const N: usize>} core::ops::SubAssign : fn sub_assign(a: Colour<N>, b: Channel) { Colour::map2_assign(&mut a, &[b; N].into(), Channel::sub_assign) });
impl_op_assign!(impl {<const N: usize>} core::ops::MulAssign : fn mul_assign(a: Colour<N>, b: Channel) { Colour::map2_assign(&mut a, &[b; N].into(), Channel::mul_assign) });
impl_op_assign!(impl {<const N: usize>} core::ops::DivAssign : fn div_assign(a: Colour<N>, b: Channel) { Colour::map2_assign(&mut a, &[b; N].into(), Channel::div_assign) });

// Shift left/right rotates the channels left/right by `n` places.
impl_op!(impl {<const N: usize>} core::ops::Shl : fn shl(col: Colour<N>, shift: usize) -> Colour<N> { col.0.rotate_left(shift); col });
impl_op!(impl {<const N: usize>} core::ops::Shr : fn shr(col: Colour<N>, shift: usize) -> Colour<N> { col.0.rotate_right(shift); col });

impl_op_assign!(impl {<const N: usize>} core::ops::ShlAssign : fn shl_assign(col: Colour<N>, shift: usize) { col.0.rotate_left(shift) });
impl_op_assign!(impl {<const N: usize>} core::ops::ShrAssign : fn shr_assign(col: Colour<N>, shift: usize) { col.0.rotate_right(shift) });

// endregion

// region Forwarding Operations

forward_fn! {
    impl {<const N: usize>} Colour<N> {
        abs();
        sqrt();
        recip();

        min(min: Channel);
        max(max: Channel);
        clamp(min: Channel, max: Channel);

        floor();
        ceil();

        exp();
        exp2();
        powf(f: f32);
        powi(n: i32);
    }
}

// endregion

impl<const N: usize> Hash for Colour<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.0.len());
        for c in self.0 {
            Channel::to_ne_bytes(c).hash(state)
        }
    }
}
