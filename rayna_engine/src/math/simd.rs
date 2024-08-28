//! This module contains helper

use crate::core::types::Number;
use std::ops::{Add, Div, Mul, Sub};

/// A helper struct that wraps an array of SIMD vectors, as a multidimensional
/// numeric vector.
///
/// ```
/// Self([
///     [x_1, x_2, x_3, x_4, .., x_len],
///     [y_1. y_2. y_3, y_4, .., y_len],
///     [z_1, z_2, z_3, z_4, .., y_len],
///     .. // dim times
/// ])
/// ```
#[derive(Copy, Clone, Debug)]
pub struct SimdVector<const DIM: usize, const LEN: usize>(pub [[Number; LEN]; DIM]);

// TODO: See if there's a way to combine using SIMD vectors and Glamour vectors
//  Without having to rewrite the math functions for SIMD

impl<const DIM: usize, const LEN: usize> From<[[Number; LEN]; DIM]> for SimdVector<DIM, LEN> {
    fn from(value: [[Number; LEN]; DIM]) -> Self {
        Self(value)
    }
}
impl<const DIM: usize, const LEN: usize> Add for SimdVector<DIM, LEN> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(std::array::from_fn(|n| (self.0[n], rhs.0[n])).map(|(ln, rn)| std::array::from_fn(|i| ln[i] + rn[i])))
    }
}

impl<const DIM: usize, const LEN: usize> Sub for SimdVector<DIM, LEN> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(std::array::from_fn(|n| (self.0[n], rhs.0[n])).map(|(ln, rn)| std::array::from_fn(|i| ln[i] - rn[i])))
    }
}

impl<const DIM: usize, const LEN: usize> Mul for SimdVector<DIM, LEN> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(std::array::from_fn(|n| (self.0[n], rhs.0[n])).map(|(ln, rn)| std::array::from_fn(|i| ln[i] * rn[i])))
    }
}

impl<const DIM: usize, const LEN: usize> Div for SimdVector<DIM, LEN> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(std::array::from_fn(|n| (self.0[n], rhs.0[n])).map(|(ln, rn)| std::array::from_fn(|i| ln[i] / rn[i])))
    }
}

impl<const DIM: usize, const LEN: usize> SimdVector<DIM, LEN> {
    /// Dot Product
    #[inline(always)]
    pub fn dot(l: Self, r: Self) -> SimdVector<1, LEN> {
        SimdVector([std::array::from_fn(|i| {
            std::iter::zip(l.0, r.0).map(|(ln, rn)| ln[i] * rn[i]).sum()
        })])
    }
}

impl<const LEN: usize> SimdVector<3, LEN> {
    /// Cross product
    ///
    /// Only implemented for `DIM = 3`
    #[inline(always)]
    pub fn cross(Self([ax, ay, az]): SimdVector<3, LEN>, Self([bx, by, bz]): SimdVector<3, LEN>) -> SimdVector<3, LEN> {
        Self([
            std::array::from_fn(|n| (ay[n] * bz[n]) - (by[n] * az[n])),
            std::array::from_fn(|n| (az[n] * bx[n]) - (bz[n] * ax[n])),
            std::array::from_fn(|n| (ax[n] * by[n]) - (bx[n] * ay[n])),
        ])
    }
}

impl<const DIM: usize, const LEN: usize> SimdVector<DIM, LEN> {
    pub const ZERO: SimdVector<DIM, LEN> = Self([[0.; LEN]; DIM]);
    pub const ONE: SimdVector<DIM, LEN> = Self([[1.; LEN]; DIM]);
    pub const POS_INFINITY: SimdVector<DIM, LEN> = Self([[Number::INFINITY; LEN]; DIM]);
    pub const NEG_INFINITY: SimdVector<DIM, LEN> = Self([[Number::NEG_INFINITY; LEN]; DIM]);
}
