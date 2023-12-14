use std::ops::{
    Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};
use Bound::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Bounds<T> {
    Full(RangeFull),
    Inclusive(RangeInclusive<T>),
    To(RangeTo<T>),
    ToInclusive(RangeToInclusive<T>),
    From(RangeFrom<T>),
    Normal(Range<T>),
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
