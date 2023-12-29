use std::cmp::Ordering;
use std::collections::Bound;
use std::fmt::{Display, Formatter};
use std::ops::{
    Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Bounds<T> {
    // TODO: Optimise this out and remove the inner value
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

impl<T> Bounds<T> {
    pub fn from_bounds(lower: Bound<T>, upper: Bound<T>) -> Self {
        match (lower, upper) {
            (Bound::Unbounded, Bound::Unbounded) => Self::from(..),
            (Bound::Unbounded, Bound::Included(u)) => Self::from(..=u),
            (Bound::Unbounded, Bound::Excluded(u)) => Self::from(..u),
            (Bound::Included(l), Bound::Unbounded) => Self::from(l..),
            (Bound::Included(l), Bound::Included(u)) => Self::from(l..=u),
            (Bound::Included(l), Bound::Excluded(u)) => Self::from(l..u),
            // TODO: Should these be considered valid bounds?
            (Bound::Excluded(l), Bound::Unbounded) => Self::from(l..),
            (Bound::Excluded(l), Bound::Included(u)) => Self::from(l..=u),
            (Bound::Excluded(l), Bound::Excluded(u)) => Self::from(l..u),
        }
    }

    pub fn to_bounds(self) -> (Bound<T>, Bound<T>) {
        match self {
            Self::Full(..) => (Bound::Unbounded, Bound::Unbounded),
            Self::To(r) => (Bound::Unbounded, Bound::Excluded(r.end)),
            Self::ToInclusive(r) => (Bound::Unbounded, Bound::Included(r.end)),
            Self::From(r) => (Bound::Included(r.start), Bound::Unbounded),
            Self::Inclusive(r) => {
                let (start, end) = r.into_inner();
                (Bound::Included(start), Bound::Included(end))
            }
            Self::Normal(r) => (Bound::Included(r.start), Bound::Excluded(r.end)),
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

    //noinspection DuplicatedCode - it is duplicated but variables are swapped so it's not the same
    /// Checks if the given bounds overlap with self
    pub fn bounds_overlap(self, other: Self) -> bool {
        // Calculate overlap and check it's not empty
        match self | other {
            Self::Full(..) | Self::From(..) | Self::To(..) | Self::ToInclusive(..) => true,
            Self::Inclusive(r) => !r.is_empty(),
            Self::Normal(r) => !r.is_empty(),
        }
    }
}

impl<T: PartialOrd> std::ops::BitOr for Bounds<T> {
    type Output = Bounds<T>;

    fn bitor(self, other: Self) -> Self::Output {
        let (self_lower, self_upper) = self.to_bounds();
        let (other_lower, other_upper) = other.to_bounds();

        // `lower`: Find the largest (total) lowest bound, aka the lower bound that's inside both bounds
        // `upper`: Find the smallest (total) upper bound, aka the upper bound that's inside both bounds
        // This is equivalent to finding `lower = max(self_lower, other_lower), upper = min(self_upper, other_upper)`
        // If the bounds overlap, then lower must be <= upper
        // This does take into account priority of inclusive/exclusive bounds

        let lower_bound = match (self_lower, other_lower) {
            (Bound::Unbounded, Bound::Unbounded) => Bound::Unbounded,

            (Bound::Unbounded, Bound::Included(val)) | (Bound::Included(val), Bound::Unbounded) => {
                Bound::Included(val)
            }

            (Bound::Unbounded, Bound::Excluded(val)) | (Bound::Excluded(val), Bound::Unbounded) => {
                Bound::Excluded(val)
            }

            (Bound::Included(a), Bound::Included(b)) => {
                match T::partial_cmp(&a, &b).expect("can't compare bounds") {
                    // a < b
                    Ordering::Less => Bound::Included(b),
                    // a >= b
                    Ordering::Greater | Ordering::Equal => Bound::Included(a),
                }
            }

            (Bound::Excluded(a), Bound::Excluded(b)) => {
                match T::partial_cmp(&a, &b).expect("can't compare bounds") {
                    // a < b
                    Ordering::Less => Bound::Excluded(b),
                    // a >= b
                    Ordering::Greater | Ordering::Equal => Bound::Excluded(a),
                }
            }

            (Bound::Included(i), Bound::Excluded(e)) | (Bound::Excluded(e), Bound::Included(i)) => {
                match T::partial_cmp(&i, &e).expect("can't compare bounds") {
                    // i <= e
                    Ordering::Less | Ordering::Equal => Bound::Excluded(e),
                    // i > e
                    Ordering::Greater => Bound::Included(i),
                }
            }
        };

        let upper_bound = match (self_upper, other_upper) {
            (Bound::Unbounded, Bound::Unbounded) => Bound::Unbounded,

            (Bound::Unbounded, Bound::Included(val)) | (Bound::Included(val), Bound::Unbounded) => {
                Bound::Included(val)
            }

            (Bound::Unbounded, Bound::Excluded(val)) | (Bound::Excluded(val), Bound::Unbounded) => {
                Bound::Excluded(val)
            }

            (Bound::Included(a), Bound::Included(b)) => {
                match T::partial_cmp(&a, &b).expect("can't compare bounds") {
                    // a < b
                    Ordering::Less => Bound::Included(a),
                    // a >= b
                    Ordering::Greater | Ordering::Equal => Bound::Included(b),
                }
            }

            (Bound::Excluded(a), Bound::Excluded(b)) => {
                match T::partial_cmp(&a, &b).expect("can't compare bounds") {
                    // a < b
                    Ordering::Less => Bound::Excluded(a),
                    // a >= b
                    Ordering::Greater | Ordering::Equal => Bound::Excluded(b),
                }
            }

            (Bound::Included(i), Bound::Excluded(e)) | (Bound::Excluded(e), Bound::Included(i)) => {
                match T::partial_cmp(&i, &e).expect("can't compare bounds") {
                    // i < e
                    Ordering::Less => Bound::Included(i),
                    // i >= e
                    Ordering::Greater | Ordering::Equal => Bound::Excluded(e),
                }
            }
        };

        Bounds::from_bounds(lower_bound, upper_bound)
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
