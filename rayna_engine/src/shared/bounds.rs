use std::cmp::Ordering;
use std::collections::Bound;
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

impl<T: PartialOrd> RangeBounds<T> for Bounds<T> {
    fn start_bound(&self) -> Bound<&T> {
        match self {
            Self::Full(r) => r.start_bound(),
            Self::Inclusive(r) => r.start_bound(),
            Self::To(r) => r.start_bound(),
            Self::ToInclusive(r) => r.start_bound(),
            Self::From(r) => r.start_bound(),
            Self::Normal(r) => r.start_bound(),
        }
    }
    fn end_bound(&self) -> Bound<&T> {
        match self {
            Self::Full(r) => r.end_bound(),
            Self::Inclusive(r) => r.end_bound(),
            Self::To(r) => r.end_bound(),
            Self::ToInclusive(r) => r.end_bound(),
            Self::From(r) => r.end_bound(),
            Self::Normal(r) => r.end_bound(),
        }
    }
}

impl<T: PartialOrd> Bounds<T> {
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

    /// Checks if the given bounds overlap with self
    pub fn bounds_overlap(&self, other: &Self) -> bool {
        let self_lower = self.start_bound();
        let self_upper = self.end_bound();
        let other_lower = other.start_bound();
        let other_upper = other.end_bound();

        // Find the largest (total) lowest bound, aka the lower bound that's inside both bounds
        // This is equivalent to finding `max(self_lower, other_lower)`
        let lower = match (self_lower, other_lower) {
            (Bound::Unbounded, Bound::Unbounded) => Bound::Unbounded,
            (Bound::Unbounded, b) | (b, Bound::Unbounded) => b,
            (Bound::Included(a), Bound::Included(b)) => match T::partial_cmp(a, b) {
                // a < b
                Some(Ordering::Less) => Bound::Included(b),
                // a >= b
                Some(Ordering::Greater) | Some(Ordering::Equal) => Bound::Included(a),
                // ???
                None => panic!("couldn't compare bounds a and b"),
            },
            (Bound::Excluded(a), Bound::Excluded(b)) => match T::partial_cmp(a, b) {
                // a < b
                Some(Ordering::Less) => Bound::Excluded(b),
                // a >= b
                Some(Ordering::Greater) | Some(Ordering::Equal) => Bound::Excluded(a),
                // ???
                None => panic!("couldn't compare bounds a and b"),
            },
            (Bound::Included(a), Bound::Excluded(b)) => match T::partial_cmp(a, b) {
                // a <= b
                Some(Ordering::Less) | Some(Ordering::Equal) => Bound::Excluded(b),
                // a > b
                Some(Ordering::Greater) => Bound::Included(a),
                // ???
                None => panic!("couldn't compare bounds a and b"),
            },
            (Bound::Excluded(a), Bound::Included(b)) => match T::partial_cmp(a, b) {
                // a <= b
                Some(Ordering::Less) | Some(Ordering::Equal) => Bound::Included(b),
                // a > b
                Some(Ordering::Greater) => Bound::Excluded(a),
                // ???
                None => panic!("couldn't compare bounds a and b"),
            },
        };

        // Find the smallest (total) upper bound, aka the upper bound that's inside both bounds
        // This is equivalent to finding `min(self_upper, other_upper)`
        let upper = match (self_upper, other_upper) {
            (Bound::Unbounded, Bound::Unbounded) => Bound::Unbounded,
            (Bound::Unbounded, b) | (b, Bound::Unbounded) => b,
            (Bound::Included(a), Bound::Included(b)) => match T::partial_cmp(a, b) {
                // a < b
                Some(Ordering::Less) => Bound::Included(a),
                // a >= b
                Some(Ordering::Greater) | Some(Ordering::Equal) => Bound::Included(b),
                // ???
                None => panic!("couldn't compare bounds a and b"),
            },
            (Bound::Excluded(a), Bound::Excluded(b)) => match T::partial_cmp(a, b) {
                // a < b
                Some(Ordering::Less) => Bound::Excluded(a),
                // a >= b
                Some(Ordering::Greater) | Some(Ordering::Equal) => Bound::Excluded(b),
                // ???
                None => panic!("couldn't compare bounds a and b"),
            },
            (Bound::Included(a), Bound::Excluded(b)) => match T::partial_cmp(a, b) {
                // a <= b
                Some(Ordering::Less) | Some(Ordering::Equal) => Bound::Excluded(a),
                // a > b
                Some(Ordering::Greater) => Bound::Included(b),
                // ???
                None => panic!("couldn't compare bounds a and b"),
            },
            (Bound::Excluded(a), Bound::Included(b)) => match T::partial_cmp(a, b) {
                // a <= b
                Some(Ordering::Less) | Some(Ordering::Equal) => Bound::Included(a),
                // a > b
                Some(Ordering::Greater) => Bound::Excluded(b),
                // ???
                None => panic!("couldn't compare bounds a and b"),
            },
        };

        match (lower, upper) {
            // Completely infinite range, or infinite on one side
            (Bound::Unbounded, _) | (_, Bound::Unbounded) => true,
            (Bound::Included(low), Bound::Included(up)) => match T::partial_cmp(low, up) {
                // low <= up
                Some(Ordering::Less) | Some(Ordering::Equal) => true,
                // low > up
                Some(Ordering::Greater) => false,
                // ???
                None => panic!("couldn't compare bounds low and up"),
            },
            (Bound::Excluded(low), Bound::Excluded(up)) => match T::partial_cmp(low, up) {
                // low < up
                Some(Ordering::Less) => Bound::Excluded(low),
                // low >= up
                Some(Ordering::Greater) | Some(Ordering::Equal) => Bound::Excluded(up),
                // ???
                None => panic!("couldn't compare bounds low and up"),
            },
            (Bound::Included(low), Bound::Excluded(up)) => match T::partial_cmp(low, up) {
                // low <= up
                Some(Ordering::Less) | Some(Ordering::Equal) => Bound::Excluded(low),
                // low > up
                Some(Ordering::Greater) => Bound::Included(up),
                // ???
                None => panic!("couldn't compare bounds low and up"),
            },
            (Bound::Excluded(low), Bound::Included(up)) => match T::partial_cmp(low, up) {
                // low <= up
                Some(Ordering::Less) | Some(Ordering::Equal) => Bound::Included(low),
                // low > up
                Some(Ordering::Greater) => Bound::Excluded(up),
                // ???
                None => panic!("couldn't compare bounds low and up"),
            },
        }
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
