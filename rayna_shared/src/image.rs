use derivative::Derivative;
use getset::{CopyGetters, Getters};
use num_integer::Integer;
use rayna_shared::colour::ColourRgb;
use rayna_shared::def::types::Channel;
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[derive(CopyGetters, Getters, Derivative)]
#[derivative(Debug)]
pub struct Image<Col = ColourRgb> {
    #[get_copy = "pub"]
    width: usize,
    #[get_copy = "pub"]
    height: usize,
    #[get_copy = "pub"]
    len: usize,
    #[derivative(Debug = "ignore")]
    #[get = "pub"]
    data: Box<[Col]>,
}

// region Constructors

impl<Col: Clone + Default> Image<Col> {
    /// Creates a new image with the specified dimensions, and the default pixel value
    pub fn new_blank(width: usize, height: usize) -> Self {
        let mut data = vec![];
        data.resize(width * height, Default::default());
        Self::new_from(width, height, data)
    }
}
impl<Col: Clone> Image<Col> {
    /// Creates a new image with the specified dimensions, and the given fill pixel value
    pub fn new_filled(width: usize, height: usize, fill: Col) -> Self {
        let mut data = vec![];
        data.resize(width * height, fill);
        Self::new_from(width, height, data)
    }
}

impl<Col> Image<Col> {
    /// Creates an image from the image's dimensions, and a slice of pixels
    ///
    /// # Panics
    /// The length of the `data` must be equal to the number of pixels `width * height`.
    pub fn new_from(width: usize, height: usize, data: impl Into<Box<[Col]>>) -> Self {
        let data = data.into();
        let len = width * height;
        assert_eq!(data.len(), len, "number of pixels does not match dimensions");

        Self {
            width,
            height,
            data,
            len,
        }
    }
}

// regions Constructors

// region Pixel Accessors

impl<Col> Image<Col> {
    fn compress_index(&self, x: usize, y: usize) -> usize { x + (y * self.width) }

    fn decompress_index(&self, n: usize) -> (usize, usize) {
        let (y, x) = usize::div_rem(&n, &self.width);
        (x, y)
    }
}

impl<Col> Index<usize> for Image<Col> {
    type Output = Col;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len, "invalid pixel index {} for len {}", index, self.len);
        &self.data[index]
    }
}

impl<Col> IndexMut<usize> for Image<Col> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len, "invalid pixel index {} for len {}", index, self.len);
        &mut self.data[index]
    }
}

impl<Col> Index<(usize, usize)> for Image<Col> {
    type Output = Col;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        assert!(
            x < self.width && y < self.height,
            "invalid pixel index ({}, {}) for dims ({},{})",
            x,
            y,
            self.width,
            self.height
        );
        self.index(self.compress_index(x, y))
    }
}

impl<Col> IndexMut<(usize, usize)> for Image<Col> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        assert!(
            x < self.width && y < self.height,
            "invalid pixel index ({}, {}) for dims ({},{})",
            x,
            y,
            self.width,
            self.height
        );
        self.index_mut(self.compress_index(x, y))
    }
}

// endregion Pixel Accessors

// region Deref

impl<Col> Deref for Image<Col> {
    type Target = [Col];

    fn deref(&self) -> &Self::Target { self.data.deref() }
}
impl<Col> DerefMut for Image<Col> {
    fn deref_mut(&mut self) -> &mut Self::Target { self.data.deref_mut() }
}

// endregion Deref

// region Iteration

/// An enumerated iterator over the pixels of an image.
///
/// Will iterate the pixels row-by-row, returning the position of the pixel as well
///
/// # Returns
/// Each value returned will be `(x, y, colour)`
pub struct ImageIterator<Col> {
    pixels: std::vec::IntoIter<Col>,
    x: usize,
    y: usize,
    width: usize,
}

impl<Col> ImageIterator<Col> {
    pub fn new(image: Image<Col>) -> Self {
        Self {
            width: image.width,
            x: 0,
            y: 0,
            pixels: image.data.into_vec().into_iter(),
        }
    }
}

impl<Col> Iterator for ImageIterator<Col> {
    type Item = (usize, usize, Col);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;
        self.pixels.next().map(|p| (x, y, p))
    }
}

impl<Col> IntoIterator for Image<Col> {
    type Item = (usize, usize, Col);
    type IntoIter = ImageIterator<Col>;

    fn into_iter(self) -> Self::IntoIter { ImageIterator::new(self) }
}

// endregion Iteration
