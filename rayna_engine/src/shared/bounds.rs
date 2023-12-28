use std::fmt::{Display, Formatter};
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
            Self::Inclusive(r) => r.contains(val),
            Self::To(r) => r.contains(val),
            Self::ToInclusive(r) => r.contains(val),
            Self::From(r) => r.contains(val),
            Self::Normal(r) => r.contains(val),
        }
    }

    // TODO: Expand this to cover two full `Bounds<T>` objects, overlapping with each other
    /// Checks if the given range `min..max` overlaps with the bounds (`self`)
    pub fn range_overlaps(&self, min: &T, max: &T) -> bool {
        return match self {
            Self::Full(_) => true,
            Self::Inclusive(r) => {
                let low = if min > r.start() { min } else { r.start() };
                let high = if max < r.end() { max } else { r.end() };
                low <= high
            }
            Self::To(r) => {
                let high = if max < &r.end { max } else { &r.end };
                min < high
            }
            Self::ToInclusive(r) => {
                let high = if max < &r.end { max } else { &r.end };
                min <= high
            }
            Self::From(r) => {
                let low = if min > &r.start { min } else { &r.start };
                low <= max
            }
            Self::Normal(r) => {
                let low = if min > &r.start { min } else { &r.start };
                let high = if max < &r.end { max } else { &r.end };
                low < high
            }
        };
    }
}

impl<T: Display> Display for Bounds<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full(_) => write!(f, ".."),
            Self::Inclusive(r) => write!(f, "{}..={}", r.start(), r.end()),
            Self::To(r) => write!(f, "..{}", r.end),
            Self::ToInclusive(r) => write!(f, "..={}", r.end),
            Self::From(r) => write!(f, "{}..", r.start),
            Self::Normal(r) => write!(f, "{}..{}", r.start, r.end),
        }
    }
}
