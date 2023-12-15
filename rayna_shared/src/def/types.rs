use image::{ImageBuffer, Rgb};

pub type Channel = f32;
pub type Pixel = Rgb<Channel>;
pub type ImgBuf = ImageBuffer<Pixel, Vec<Channel>>;

pub type Number = f64;
pub type Vector = glamour::Vector3<Number>;
pub type Quaternion = glamour::Transform3<Number, Number>;
