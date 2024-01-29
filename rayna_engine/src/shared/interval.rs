use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/// Represents a interval of values. There may/not be a `start` and/or `end` bound.
///
/// # Requirements
/// It is a logic error for `start > end`. This requirement may not necessarily be enforced due to performance reasons,
/// and is considered UB.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Interval<T> {
    pub start: Option<T>,
    pub end: Option<T>,
}

impl<T> From<RangeFull> for Interval<T> {
    fn from(_value: RangeFull) -> Self { Self { start: None, end: None } }
}
impl<T> From<RangeInclusive<T>> for Interval<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        let (min, max) = value.into_inner();
        Self {
            start: Some(min),
            end: Some(max),
        }
    }
}
impl<T> From<RangeTo<T>> for Interval<T> {
    fn from(value: RangeTo<T>) -> Self {
        Self {
            start: None,
            end: Some(value.end),
        }
    }
}
impl<T> From<RangeToInclusive<T>> for Interval<T> {
    fn from(value: RangeToInclusive<T>) -> Self {
        Self {
            start: None,
            end: Some(value.end),
        }
    }
}
impl<T> From<RangeFrom<T>> for Interval<T> {
    fn from(value: RangeFrom<T>) -> Self {
        Self {
            start: Some(value.start),
            end: None,
        }
    }
}
impl<T> From<Range<T>> for Interval<T> {
    fn from(value: Range<T>) -> Self {
        Self {
            start: Some(value.start),
            end: Some(value.end),
        }
    }
}

impl<T> Interval<T> {
    pub const FULL: Self = Self { start: None, end: None };
}

impl<T: PartialOrd> Interval<T> {
    /// Checks if the given range `min..max` overlaps with the bounds (`self`)
    pub fn range_overlaps(&self, min: &T, max: &T) -> bool {
        return match self {
            Self { start: None, end: None } => true,
            Self {
                start: Some(start),
                end: Some(end),
            } => {
                let low = if min > start { min } else { start };
                let high = if max < end { max } else { end };
                low <= high
            }
            Self {
                start: None,
                end: Some(end),
            } => {
                let high = if max < &end { max } else { &end };
                min <= high
            }
            Self {
                start: Some(start),
                end: None,
            } => {
                let low = if min > &start { min } else { &start };
                low <= max
            }
        };
    }

    /// Checks if the given bounds overlap with self
    pub fn bounds_overlap(self, other: Self) -> bool {
        // Calculate overlap and check it's valid
        // If the bounds overlap, then lower must be <= upper
        return match self & other {
            Self { start: None, .. } | Self { end: None, .. } => true,
            Self {
                start: Some(start),
                end: Some(end),
            } => start <= end,
        };
    }

    pub fn contains(&self, item: &T) -> bool {
        match self {
            Self {
                start: Some(start),
                end: Some(end),
            } => start <= item && item <= end,
            Self {
                start: Some(start),
                end: None,
            } => start <= item,
            Self {
                start: None,
                end: Some(end),
            } => item <= end,
            Self { start: None, end: None } => true,
        }
    }
}

impl<T: PartialOrd> std::ops::BitAnd for Interval<T> {
    type Output = Interval<T>;

    fn bitand(self, other: Self) -> Self::Output {
        // `lower`: Find the largest (total) lowest bound, aka the lower bound that's inside both bounds
        // `upper`: Find the smallest (total) upper bound, aka the upper bound that's inside both bounds
        // This is equivalent to finding `lower = max(self_lower, other_lower), upper = min(self_upper, other_upper)`

        let start = match (self.start, other.start) {
            (None, start) | (start, None) => start,
            (Some(a), Some(b)) => {
                match T::partial_cmp(&a, &b).expect("can't compare bounds") {
                    // a < b
                    Ordering::Less => Some(b),
                    // a >= b
                    Ordering::Greater | Ordering::Equal => Some(a),
                }
            }
        };

        let end = match (self.end, other.end) {
            (None, start) | (start, None) => start,
            (Some(a), Some(b)) => {
                match T::partial_cmp(&a, &b).expect("can't compare bounds") {
                    // a < b
                    Ordering::Less => Some(a),
                    // a >= b
                    Ordering::Greater | Ordering::Equal => Some(b),
                }
            }
        };

        Self { start, end }
    }
}

impl<T: Display> Display for Interval<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(start) = &self.start {
            write!(f, "{start}")?;
        }
        write!(f, "..")?;
        if let Some(end) = &self.end {
            write!(f, "{end}")?
        }
        Ok(())

        // match self {
        //     Self { start:None, end: None} => write!(f, ".."),
        //     Self { start: Some(start), end: Some(end)} => write!(f, "{}..={}", start, end),
        //     Self { start: None, end:Some(end) } => write!(f, "..{}", end),
        //     Self { start: Some(start), end:None } => write!(f, "{}..", start),
        // }
    }
}

impl<T> Interval<T> {
    pub fn with_start(self, start: Option<T>) -> Self { Self { start, ..self } }
    pub fn with_some_start(self, start: T) -> Self {
        Self {
            start: Some(start),
            ..self
        }
    }
}
