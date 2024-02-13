//! This module contains helper

use crate::core::types::Number;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

/// A helper struct
pub struct SimdMath<const N: usize>
where
    LaneCount<N>: SupportedLaneCount;

impl<const N: usize> SimdMath<N>
where
    LaneCount<N>: SupportedLaneCount,
{
    pub const ZERO: Simd<Number, N> = Simd::from_array([0.; N]);
    pub const ONE: Simd<Number, N> = Simd::from_array([1.; N]);
    pub const POS_INFINITY: Simd<Number, N> = Simd::from_array([Number::INFINITY; N]);
    pub const NEG_INFINITY: Simd<Number, N> = Simd::from_array([Number::NEG_INFINITY; N]);

    #[inline(always)]
    pub fn simd_multi_cross(
        [ax, ay, az]: [Simd<Number, N>; 3],
        [bx, by, bz]: [Simd<Number, N>; 3],
    ) -> [Simd<Number, N>; 3] {
        [(ay * bz) - (by * az), (az * bx) - (bz * ax), (ax * by) - (bx * ay)]
    }

    #[inline(always)]
    pub fn simd_multi_dot([ax, ay, az]: [Simd<Number, N>; 3], [bx, by, bz]: [Simd<Number, N>; 3]) -> Simd<Number, N> {
        (ax * bx) + (ay * by) + (az * bz)
    }

    #[inline(always)]
    pub fn simd_multi_sub(
        [ax, ay, az]: [Simd<Number, N>; 3],
        [bx, by, bz]: [Simd<Number, N>; 3],
    ) -> [Simd<Number, N>; 3] {
        [ax - bx, ay - by, az - bz]
    }
}
