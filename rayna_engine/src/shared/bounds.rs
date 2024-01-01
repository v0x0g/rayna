use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Bounds<T> {
    pub start: Option<T>,
    pub end: Option<T>,
}

impl<T> From<RangeFull> for Bounds<T> {
    fn from(_value: RangeFull) -> Self {
        Self {
            start: None,
            end: None,
        }
    }
}
impl<T> From<RangeInclusive<T>> for Bounds<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        let (min, max) = value.into_inner();
        Self {
            start: Some(min),
            end: Some(max),
        }
    }
}
impl<T> From<RangeTo<T>> for Bounds<T> {
    fn from(value: RangeTo<T>) -> Self {
        Self {
            start: None,
            end: Some(value.end),
        }
    }
}
impl<T> From<RangeToInclusive<T>> for Bounds<T> {
    fn from(value: RangeToInclusive<T>) -> Self {
        Self {
            start: None,
            end: Some(value.end),
        }
    }
}
impl<T> From<RangeFrom<T>> for Bounds<T> {
    fn from(value: RangeFrom<T>) -> Self {
        Self {
            start: Some(value.start),
            end: None,
        }
    }
}
impl<T> From<Range<T>> for Bounds<T> {
    fn from(value: Range<T>) -> Self {
        Self {
            start: Some(value.start),
            end: Some(value.end),
        }
    }
}

impl<T> Bounds<T> {
    pub const FULL: Self = Self {
        start: None,
        end: None,
    };
}

impl<T: PartialOrd> Bounds<T> {
    // TODO: Expand this to cover two full `Bounds<T>` objects, overlapping with each other
    /// Checks if the given range `min..max` overlaps with the bounds (`self`)
    pub fn range_overlaps(&self, min: &T, max: &T) -> bool {
        return match self {
            Self {
                start: None,
                end: None,
            } => true,
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
            Self {
                start: None,
                end: None,
            } => true,
        }
    }
}

impl<T: PartialOrd> std::ops::BitAnd for Bounds<T> {
    type Output = Bounds<T>;

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

impl<T: Display> Display for Bounds<T> {
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
