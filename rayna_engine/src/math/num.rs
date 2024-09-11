use core::ops::{Add, Mul, Sub};

pub trait Lerp<Frac>: Add<Self, Output = Self> + Sub<Self, Output = Self> + Mul<Frac, Output = Self> + Sized {
    fn lerp(a: Self, b: Self, t: Frac) -> Self;
}

impl<Frac, T: Add<Self, Output = Self> + Sub<Self, Output = Self> + Mul<Frac, Output = Self> + Sized + Clone> Lerp<Frac>
    for T
{
    fn lerp(a: Self, b: Self, t: Frac) -> Self {
        a.clone() + (b - a) * t
    }
}
