// NOTE: I adapted this based off the `noise` crate's gradient

use crate::core::math::Lerp;
use crate::core::types::Number;
use getset::{CopyGetters, Setters};
use smallvec::SmallVec;
use std::{fmt::Debug, ops::Deref};

#[derive(Clone, Copy, Debug, Default, CopyGetters)]
#[get_copy = "pub"]
pub struct GradientPoint<T: Copy + Debug> {
    pos: Number,
    value: T,
}

#[derive(Clone, Copy, Debug, Default, CopyGetters, Setters)]
#[getset(get_copy = "pub", set = "pub")]
pub struct GradientDomain {
    min: Number,
    max: Number,
}

#[derive(Clone, Debug, Default)]
pub struct Gradient<T: Copy + Debug> {
    // Store a small number of points inline
    points: SmallVec<[GradientPoint<T>; 8]>,
    domain: GradientDomain,
}

impl<T: Copy + Debug> Gradient<T> {
    pub fn domain(&self) -> GradientDomain {
        self.domain
    }
    pub fn points(&self) -> &[GradientPoint<T>] {
        self.points.deref()
    }
}

impl<T: Copy + Debug> Gradient<T> {
    pub fn new() -> Self {
        Self {
            points: SmallVec::new(),
            domain: GradientDomain { min: 0.0, max: 0.0 },
        }
    }

    pub fn add_gradient_point(mut self, pos: Number, value: T) -> Self {
        let new_point = GradientPoint { pos, value };

        // first check to see if the position is within the domain of the gradient. if the position
        // is not within the domain, expand the domain and add the GradientPoint
        if self.domain.min > pos {
            self.domain.set_min(pos);
            // since the new position is the smallest value, insert it at the beginning of the
            // gradient
            self.points.insert(0, new_point);
        } else if self.domain.max < pos {
            self.domain.set_max(pos);

            // since the new position is at the largest value, insert it at the end of the gradient
            self.points.push(new_point)
        } else if !self // new point must be somewhere inside the existing domain. Check to see if
            // it doesn't exist already
            .points
            .iter()
            .any(|&x| (x.pos - pos).abs() < Number::EPSILON)
        {
            // it doesn't, so find the correct position to insert the new
            // control point.
            let insertion_point = self.find_insertion_point(pos);

            // add the new control point at the correct position.
            self.points.insert(insertion_point, new_point);
        }

        self
    }

    fn find_insertion_point(&self, pos: Number) -> usize {
        self.points
            .iter()
            .position(|x| x.pos >= pos)
            .unwrap_or(self.points.len())
    }

    pub fn get(&self, pos: Number) -> T
    where
        T: Default
            + Clone
            + std::ops::Add<T, Output = T>
            + std::ops::Sub<T, Output = T>
            + std::ops::Mul<Number, Output = T>,
    {
        // If there are no colors in the gradient, return black
        if self.points.is_empty() {
            T::default()
        } else {
            if pos < self.domain.min {
                self.points.first().unwrap().value.clone()
            } else if pos > self.domain.max {
                self.points.last().unwrap().value.clone()
            } else {
                for points in self.points.windows(2) {
                    if (points[0].pos <= pos) && (points[1].pos > pos) {
                        // Compute the alpha value used for linear interpolation
                        let alpha = (pos - points[0].pos) / (points[1].pos - points[0].pos);

                        // Now perform the interpolation and return.
                        return Lerp::lerp(points[0].value.clone(), points[1].value.clone(), alpha);
                    }
                }
                // Checked was `min < pos < max`, so there must be some two points it is between
                unreachable!("no gradient points found for position {pos}");
            }
        }
    }
}

impl<T: Copy + Debug> IntoIterator for Gradient<T> {
    type Item = GradientPoint<T>;
    type IntoIter = smallvec::IntoIter<[GradientPoint<T>; 8]>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.into_iter()
    }
}
