use crate::core::types::{Colour, Number};
use crate::shared::math::Lerp;
use derivative::Derivative;
use getset::{CopyGetters, Getters};
use ndarray::{ArcArray, Ix2, Shape};
use std::ops::{Deref, DerefMut};

#[derive(CopyGetters, Getters, Derivative, Clone)]
#[derivative(Debug)]
pub struct Image<Col = Colour> {
    #[get_copy = "pub"]
    width: usize,
    #[get_copy = "pub"]
    height: usize,
    #[derivative(Debug = "ignore")]
    #[get = "pub"]
    data: ArcArray<Col, Ix2>,
}

// region Constructors

impl<Col: Clone + Default> Image<Col> {
    /// Creates a new image with the specified dimensions, and the default pixel value
    pub fn new_blank(width: usize, height: usize) -> Self {
        Self::new(ArcArray::from_elem(Shape::from(Ix2(width, height)), Default::default()))
    }
}

impl<Col: Clone> Image<Col> {
    /// Creates a new image with the specified dimensions, and the given fill pixel value
    pub fn new_filled(width: usize, height: usize, fill: Col) -> Self {
        Self::new(ArcArray::from_elem(Shape::from(Ix2(width, height)), fill))
    }
}

impl<Col> Image<Col> {
    pub fn new(data: impl Into<ArcArray<Col, Ix2>>) -> Self {
        let data = data.into();
        let (width, height) = data.dim();

        Self { width, height, data }
    }

    /// Creates an image from the image's dimensions, using the given function to calculate pixel values
    pub fn from_fn(width: usize, height: usize, mut func: impl FnMut(usize, usize) -> Col) -> Self {
        Self::new(ArcArray::from_shape_fn(
            Shape::from(Ix2(width, height)),
            move |(x, y)| func(x, y),
        ))
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

        Self::new(
            // `NDarray` and `image` seem to have different row/column ordering, so swap the axes to compensate
            // else our image is sideways
            ArcArray::from_shape_vec(Shape::from(Ix2(width, height)), data)
                .expect("array creation failed")
                .reversed_axes(),
        )
    }
}

// endregion From<> for crate `image`

// region Pixel Accessors

impl<Col> Image<Col> {
    fn bilinear_coords(&self, val: Number, max: usize) -> (usize, usize, Number) {
        let floor = val.floor().clamp(0., (max - 1) as _);
        let ceil = val.ceil().clamp(0., (max - 1) as _);
        let frac = val - floor;

        (floor as _, ceil as _, frac)
    }

    pub fn get_bilinear(&self, px: Number, py: Number) -> Col
    where
        Col: Lerp<Number> + Clone,
    {
        let (x1, x2, xl) = self.bilinear_coords(px, self.width);
        let (y1, y2, yl) = self.bilinear_coords(py, self.height);
        let [c11, c12, c21, c22] = [(x1, y1), (x1, y2), (x2, y1), (x2, y2)].map(|c| self[c].clone());

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
    type Target = ArcArray<Col, Ix2>;

    fn deref(&self) -> &Self::Target { &self.data }
}
impl<Col> DerefMut for Image<Col> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.data }
}

// endregion Deref

// TODO: Parallel iteration?
