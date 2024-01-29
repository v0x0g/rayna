// use crate::core::image::Image;
// use crate::core::types::Number;
// use crate::mesh::primitive::axis_box::AxisBoxMesh;
// use derivative::Derivative;
// use getset::{CopyGetters, Getters};
// use std::mem::MaybeUninit;
// use std::ops::{Index, IndexMut};
//
// #[derive(Copy, Clone, Debug)]
// pub struct Voxel {
//     pub value: Number,
// }
//
// #[derive(Copy, Clone, Debug)]
// struct VoxelDataInner {
//     pub inner: AxisBoxMesh,
// }
//
// /// A mesh struct that is created from a grid of voxels
// ///
// /// # Transforming
// /// This mesh purposefully does not have any properties for transforming, so you must you a
// /// [ObjectTransform].
// #[derive(CopyGetters, Getters, Derivative, Clone)]
// #[derivative(Debug)]
// pub struct VoxelGridMesh {
//     #[get_copy = "pub"]
//     width: usize,
//     #[get_copy = "pub"]
//     height: usize,
//     #[get_copy = "pub"]
//     depth: usize,
//     /// How many total voxels there are in this [VoxelGridMesh]
//     #[get_copy = "pub"]
//     count: usize,
//     /// The raw data for the grid
//     #[derivative(Debug = "ignore")]
//     #[get = "pub"]
//     data: Box<[Voxel]>,
// }
//
// pub trait GeneratorFunction = Fn([usize; 3]) -> Number;
//
// // region Constructors
//
// impl VoxelGridMesh {
//     pub fn generate(width: usize, height: usize, depth: usize, func: impl GeneratorFunction) -> Self {
//         let count = width * height * depth;
//         // Annoyingly, there doesn't seem to be a way to create a vec/slice from a `fn (...) -> T`, only `fn() -> T`
//         // So do it the manual way with `MaybeUninit`
//         let data = unsafe {
//             let mut data = Box::new_uninit_slice(count);
//             data.iter_mut().enumerate().for_each(|(i, px)| {
//                 let (x, y) = Self::decompress_index_dims(i, [width, height]);
//                 *px = MaybeUninit::new(func(x, y));
//             });
//             Box::<[MaybeUninit<Col>]>::assume_init(data)
//         };
//     }
// }
//
// // endregion Constructors
//
// // region Pixel Accessors
//
// impl<Col> Image<Col> {
//     fn compress_index(&self, x: usize, y: usize) -> usize { x + (y * self.width) }
//
//     fn decompress_index_dims(n: usize, [width, _height]: [usize; 2]) -> (usize, usize) {
//         let (y, x) = usize::div_rem(&n, &width);
//         (x, y)
//     }
// }
//
// impl<Col> Index<usize> for Image<Col> {
//     type Output = Col;
//
//     /// Direct access to the pixel buffer. Don't use this please
//     fn index(&self, index: usize) -> &Self::Output {
//         assert!(index < self.len, "invalid pixel index {} for len {}", index, self.len);
//         &self.data[index]
//     }
// }
//
// impl<Col> IndexMut<usize> for Image<Col> {
//     /// Direct access to the pixel buffer. Don't use this please
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         assert!(index < self.len, "invalid pixel index {} for len {}", index, self.len);
//         &mut self.data[index]
//     }
// }
//
// impl<Col> Index<(usize, usize)> for Image<Col> {
//     type Output = Col;
//
//     fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
//         assert!(
//             x < self.width && y < self.height,
//             "invalid pixel index ({}, {}) for dims ({},{})",
//             x,
//             y,
//             self.width,
//             self.height
//         );
//         self.index(self.compress_index(x, y))
//     }
// }
//
// impl<Col> IndexMut<(usize, usize)> for Image<Col> {
//     fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
//         assert!(
//             x < self.width && y < self.height,
//             "invalid pixel index ({}, {}) for dims ({},{})",
//             x,
//             y,
//             self.width,
//             self.height
//         );
//         self.index_mut(self.compress_index(x, y))
//     }
// }
//
// // endregion Voxel Accessors
