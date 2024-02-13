//! This module contains helper

use crate::core::types::Number;
use std::ops::{Add, Div, Mul, Sub};
use std::simd::{LaneCount, Simd, SupportedLaneCount};

/// A helper struct that wraps an array of SIMD vectors, as a multidimensional
/// numeric vector.
#[derive(Copy, Clone, Debug)]
pub struct SimdVector<const L: usize, const N: usize>(pub [Simd<Number, L>; N])
where
    LaneCount<L>: SupportedLaneCount;

// TODO: See if there's a way to combine using SIMD vectors and Glamour vectors
//  Without having to rewrite the math functions for SIMD

impl<const L: usize, const N: usize> From<[Simd<Number, L>; N]> for SimdVector<L, N>
where
    LaneCount<L>: SupportedLaneCount,
{
    fn from(value: [Simd<Number, L>; N]) -> Self { Self(value) }
}
impl<const L: usize, const N: usize> Add for SimdVector<L, N>
where
    LaneCount<L>: SupportedLaneCount,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output { Self(std::array::from_fn(|i| self.0[i] + rhs.0[i])) }
}

impl<const L: usize, const N: usize> Sub for SimdVector<L, N>
where
    LaneCount<L>: SupportedLaneCount,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output { Self(std::array::from_fn(|i| self.0[i] - rhs.0[i])) }
}

impl<const L: usize, const N: usize> Mul for SimdVector<L, N>
where
    LaneCount<L>: SupportedLaneCount,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output { Self(std::array::from_fn(|i| self.0[i] * rhs.0[i])) }
}

impl<const L: usize, const N: usize> Div for SimdVector<L, N>
where
    LaneCount<L>: SupportedLaneCount,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output { Self(std::array::from_fn(|i| self.0[i] / rhs.0[i])) }
}

impl<const L: usize, const N: usize> SimdVector<L, N>
where
    LaneCount<L>: SupportedLaneCount,
{
    #[inline(always)]
    pub fn dot(a: Self, b: Self) -> Simd<Number, L> { std::iter::zip(a.0, b.0).map(|(a, b)| a * b).sum() }
}

impl<const L: usize> SimdVector<L, 3>
where
    LaneCount<L>: SupportedLaneCount,
{
    #[inline(always)]
    pub fn cross(Self([ax, ay, az]): SimdVector<L, 3>, Self([bx, by, bz]): SimdVector<L, 3>) -> SimdVector<L, 3> {
        Self([(ay * bz) - (by * az), (az * bx) - (bz * ax), (ax * by) - (bx * ay)])
    }
}

pub struct SimdConstants<const L: usize>
where
    LaneCount<L>: SupportedLaneCount;

impl<const L: usize> SimdConstants<L>
where
    LaneCount<L>: SupportedLaneCount,
{
    pub const ZERO: Simd<Number, L> = Simd::from_array([0.; L]);
    pub const ONE: Simd<Number, L> = Simd::from_array([1.; L]);
    pub const POS_INFINITY: Simd<Number, L> = Simd::from_array([Number::INFINITY; L]);
    pub const NEG_INFINITY: Simd<Number, L> = Simd::from_array([Number::NEG_INFINITY; L]);
}
