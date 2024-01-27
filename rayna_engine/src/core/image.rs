use crate::core::types::{Colour, Number};
use crate::shared::math::{lerp, Lerp};
use derivative::Derivative;
use getset::{CopyGetters, Getters};
use num_integer::Integer;
use num_traits::FromPrimitive;
use std::mem::MaybeUninit;
use std::ops::{Add, Deref, DerefMut, Index, IndexMut, Mul, Sub};

#[derive(CopyGetters, Getters, Derivative, Clone)]
#[derivative(Debug)]
pub struct Image<Col = Colour> {
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

    /// Creates an image from the image's dimensions, using the given function to calculate pixel values
    pub fn from_fn(width: usize, height: usize, mut func: impl FnMut(usize, usize) -> Col) -> Self {
        let len = width * height;
        // Annoyingly, there doesn't seem to be a way to create a vec/slice from a `fn (usize) -> T`, only `fn() -> T`
        // So do it the manual way with `MaybeUninit`
        let data = unsafe {
            let mut data = Box::new_uninit_slice(len);
            data.iter_mut().enumerate().for_each(|(i, px)| {
                let (x, y) = Self::decompress_index_dims(i, [width, height]);
                *px = MaybeUninit::new(func(x, y));
            });
            Box::<[MaybeUninit<Col>]>::assume_init(data)
        };

        Self {
            width,
            height,
            data,
            len,
        }
    }
}

// endregion Constructors

// region From<> for crate `image`
impl From<image::DynamicImage> for Image<Colour> {
    fn from(img: image::DynamicImage) -> Self {
        // Try convert into appropriate pixel format
        let img = img.into_rgb32f();
        let (width, height, data) = (img.width() as _, img.height() as _, img.into_raw());
        // Have to transmute the data buffer because it's flattened, which we don't want
        let data = unsafe {
            // SAFETY: `Colour` is a wrapper around a `[Channel; Colour::CHANNEL_COUNT]`, so we can safely transmute
            let (ptr, len, cap) = data.into_raw_parts();
            Vec::from_raw_parts(
                ptr as *mut Colour,
                len / Colour::CHANNEL_COUNT,
                cap / Colour::CHANNEL_COUNT,
            )
        };
        Self::new_from(width, height, data)
    }
}
// endregion From<> for crate `image`

// region Pixel Accessors

impl<Col> Image<Col> {
    fn compress_index(&self, x: usize, y: usize) -> usize { x + (y * self.width) }

    fn decompress_index_dims(n: usize, [width, _height]: [usize; 2]) -> (usize, usize) {
        let (y, x) = usize::div_rem(&n, &width);
        (x, y)
    }
}

impl<Col> Index<usize> for Image<Col> {
    type Output = Col;

    /// Direct access to the pixel buffer. Don't use this please
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len, "invalid pixel index {} for len {}", index, self.len);
        &self.data[index]
    }
}

impl<Col> IndexMut<usize> for Image<Col> {
    /// Direct access to the pixel buffer. Don't use this please
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

impl<Col> Image<Col> {
    fn bilinear_coords(&self, val: Number, max: usize) -> (usize, usize, Number) {
        let floor = val.floor().clamp(0., (max - 1) as _);
        let ceil = val.ceil().clamp(0., (max - 1) as _);
        let frac = val - floor;

        (floor as _, ceil as _, frac)
    }

    pub fn get_bilinear(&self, px: Number, py: Number) -> Col
    where
        Col: Lerp<Number>,
    {
        let (x1, x2, xl) = self.bilinear_coords(px, self.width);
        let (y1, y2, yl) = self.bilinear_coords(py, self.height);
        let [c11, c12, c21, c22] = [(x1, y1), (x1, y2), (x2, y1), (x2, y2)].map(|c| self[c]);

        // Interpolate over x-axis
        let cy1/* Y=Y1 */ = Col::lerp(c11, c21, xl);
        let cy2/* Y=Y2 */ = Col::lerp(c12, c22, xl);

        let c = Col::lerp(cy1, cy2, yl);
        c
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

// region Iteration (Owned)

/// An enumerated iterator over the pixels of an owned image.
///
/// Will iterate the pixels row-by-row, returning the position of the pixel as well
///
/// # Returns
/// Each value returned will be `(x, y, colour)`
pub struct ImageIteratorOwned<Col> {
    pixels: std::vec::IntoIter<Col>,
    x: usize,
    y: usize,
    width: usize,
}

impl<Col> ImageIteratorOwned<Col> {
    pub fn new(image: Image<Col>) -> Self {
        Self {
            width: image.width,
            x: 0,
            y: 0,
            pixels: image.data.into_vec().into_iter(),
        }
    }
}

impl<Col> Iterator for ImageIteratorOwned<Col> {
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
    type IntoIter = ImageIteratorOwned<Col>;

    fn into_iter(self) -> Self::IntoIter { ImageIteratorOwned::new(self) }
}

// endregion Iteration (Owned)

// region Iteration (Mut)

/// An enumerated iterator over the pixels of a mutable image reference.
///
/// Will iterate the pixels row-by-row, returning the position of the pixel as well
///
/// # Returns
/// Each value returned will be `(x, y, &mut pixel)`
pub struct ImageIteratorMut<'img, Col> {
    iter: std::slice::IterMut<'img, Col>,
    x: usize,
    y: usize,
    width: usize,
}

impl<'img, Col> ImageIteratorMut<'img, Col> {
    pub fn new(image: &'img mut Image<Col>) -> Self {
        Self {
            x: 0,
            y: 0,
            width: image.width,
            iter: image.iter_mut(),
        }
    }
}

impl<'img, Col> Iterator for ImageIteratorMut<'img, Col> {
    type Item = (usize, usize, &'img mut Col);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;

        self.iter.next().map(|p| (x, y, p))
    }
}

impl<'img, Col> IntoIterator for &'img mut Image<Col> {
    type Item = (usize, usize, &'img mut Col);
    type IntoIter = ImageIteratorMut<'img, Col>;

    fn into_iter(self) -> Self::IntoIter { ImageIteratorMut::new(self) }
}

// endregion Iteration (Mut)

// region Iteration (Ref)

/// An enumerated iterator over the pixels of an image reference.
///
/// Will iterate the pixels row-by-row, returning the position of the pixel as well
///
/// # Returns
/// Each value returned will be `(x, y, &pixel)`
pub struct ImageIterator<'img, Col> {
    image: &'img Image<Col>,
    x: usize,
    y: usize,
}

impl<'img, Col> ImageIterator<'img, Col> {
    pub fn new(image: &'img Image<Col>) -> Self { Self { x: 0, y: 0, image } }
}

impl<'img, Col> Iterator for ImageIterator<'img, Col> {
    type Item = (usize, usize, &'img Col);

    fn next(&mut self) -> Option<Self::Item> {
        // Iteration complete
        if self.y >= self.image.height {
            return None;
        }

        if self.x >= self.image.width {
            self.x = 0;
            self.y += 1;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;

        Some((x, y, &self.image[(x, y)]))
    }
}

impl<'img, Col> IntoIterator for &'img Image<Col> {
    type Item = (usize, usize, &'img Col);
    type IntoIter = ImageIterator<'img, Col>;

    fn into_iter(self) -> Self::IntoIter { ImageIterator::new(self) }
}

// endregion Iteration (Ref)

// TODO: Parallel iteration?
