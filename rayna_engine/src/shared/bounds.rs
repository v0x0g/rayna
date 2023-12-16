use std::fmt::{Display, Formatter, Pointer};
use std::ops::{
    Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Bounds<T> {
    Full(RangeFull),
    Inclusive(RangeInclusive<T>),
    To(RangeTo<T>),
    ToInclusive(RangeToInclusive<T>),
    From(RangeFrom<T>),
    Normal(Range<T>),
}

impl<T> From<RangeFull> for Bounds<T> {
    fn from(value: RangeFull) -> Self {
        Self::Full(value)
    }
}
impl<T> From<RangeInclusive<T>> for Bounds<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        Self::Inclusive(value)
    }
}
impl<T> From<RangeTo<T>> for Bounds<T> {
    fn from(value: RangeTo<T>) -> Self {
        Self::To(value)
    }
}
impl<T> From<RangeToInclusive<T>> for Bounds<T> {
    fn from(value: RangeToInclusive<T>) -> Self {
        Self::ToInclusive(value)
    }
}
impl<T> From<RangeFrom<T>> for Bounds<T> {
    fn from(value: RangeFrom<T>) -> Self {
        Self::From(value)
    }
}
impl<T> From<Range<T>> for Bounds<T> {
    fn from(value: Range<T>) -> Self {
        Self::Normal(value)
    }
}

impl<T: PartialOrd> Bounds<T> {
    pub fn contains(&self, val: &T) -> bool {
        match self {
            Self::Full(r) => r.contains(val),
            Bounds::Inclusive(r) => r.contains(val),
            Bounds::To(r) => r.contains(val),
            Bounds::ToInclusive(r) => r.contains(val),
            Bounds::From(r) => r.contains(val),
            Bounds::Normal(r) => r.contains(val),
        }
    }
}

impl<T: Display> Display for Bounds<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full(_) => write!(f, ".."),
            Bounds::Inclusive(r) => write!(f, "{}..={}", r.start(), r.end()),
            Bounds::To(r) => write!(f, "..{}", r.end),
            Bounds::ToInclusive(r) => write!(f, "..={}", r.end),
            Bounds::From(r) => write!(f, "{}..", r.start),
            Bounds::Normal(r) => write!(f, "{}..{}", r.start, r.end),
        }
    }
}
