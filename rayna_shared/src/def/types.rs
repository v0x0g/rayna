use image::{ImageBuffer, Rgb};

pub type Channel = f32;
pub type Pixel = Rgb<Channel>;
pub type ImgBuf = ImageBuffer<Pixel, Vec<Channel>>;

pub type Number = f32;
pub type Angle = glamour::Angle<Number>;
pub type Vector2 = glamour::Vector2<Number>;
pub type Vector3 = glamour::Vector3<Number>;
pub type Vector4 = glamour::Vector4<Number>;
pub type Point2 = glamour::Point2<Number>;
pub type Point3 = glamour::Point3<Number>;
pub type Point4 = glamour::Point4<Number>;
pub type Matrix4 = glamour::Matrix4<Number>;
pub type Transform3 = glamour::Transform3<Number, Number>;
