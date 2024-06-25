use std::ops::{Add, Div};

use crate::core::{colour::ColourRgb, image::Image, types::Number};

/// A wrapper around an [`Image`] that stores [`AccumulationValue`]s instead of pixels
///
/// Has convenience methods for working with accumulated samples easier. Not all pixels
/// need to be sampled evenly - sample counts can be unique per pixel.
#[derive(Debug, Clone, Default)]
pub struct AccumulationBuffer<C = ColourRgb> {
    inner: Option<Image<AccumulationValue<C>>>,
    counter: usize,
}

/// Wrapper struct storing the accumulated colour value for a single pixel
///
/// # Notes
/// Currently this uses a simple mean algorithm, but in the future this might change
/// to something more advanced, to provide better noise reduction.
#[derive(Debug, Clone, Copy, Default)]
pub struct AccumulationValue<C = ColourRgb> {
    /// Sum of all samples
    sum: C,
    /// Mean of all samples
    mean: C,
    /// Counter for how many frames have been accumulated
    accum: Number,
}

impl<C: Add<Output = C> + Div<Number, Output = C> + Clone> AccumulationValue<C> {
    /// Inserts a sample with a weighting of one
    pub fn insert_sample(&mut self, sample: C) -> C { self.insert_sample_weighted(sample, 1.0) }

    /// Inserts a sample with a given weight
    ///
    /// This can be used e.g. for importance sampling
    pub fn insert_sample_weighted(&mut self, sample: C, weight: Number) -> C {
        self.sum = C::add(self.sum.clone(), sample);
        self.accum += weight;
        self.mean = self.sum.clone() / self.accum.clone();
        self.get()
    }

    /// Gets the overall accumulated colour value
    pub fn get(&self) -> C { self.mean.clone() }
}

impl<C: Default + Clone> AccumulationBuffer<C> {
    ///
    /// This ensures the given image exists and has correct dimensions. If the image dimensions changed
    /// then the image is cleared.
    pub fn new_frame(&mut self, [w, h]: [usize; 2]) -> &mut Image<AccumulationValue<C>> {
        self.counter += 1;
        // Doesn't exist
        if self.inner.is_none() {
            return self.inner.insert(Image::new_blank(w, h));
        }
        // SAFETY: If `None` then would have returned above
        let img = self.inner.as_mut().unwrap();
        // Needs resize
        if img.width() != w || img.height() != h {
            *img = Image::new_blank(w, h);
        }
        img
    }

    /// Clears the buffer, removing any accumulation that was stored
    pub fn clear(&mut self) {
        self.inner.as_mut().map(|img| img.fill(AccumulationValue::default()));
        self.counter = 0;
    }

    /// Returns the number of frames that make up this buffer.
    ///
    /// This is the number of times that [`Self::new_frame`] has been called, so it
    /// might be different to the per-pixel accumulation counters.
    pub fn frame_count(&self) -> usize { self.counter }
}
