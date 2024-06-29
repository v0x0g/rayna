use std::ops::DerefMut as _;

use egui::{Color32, ColorImage};
use puffin::{profile_function, profile_scope};
use rayna_engine::core::types::*;
use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};

pub trait ImageExt {
    /// Converts the image outputted by the renderer into an egui-appropriate one.
    /// Also converts from linear space to SRGB space
    fn to_egui(self) -> ColorImage;
}

impl ImageExt for Image {
    fn to_egui(mut self) -> ColorImage {
        profile_function!();

        // TODO: I may be doing something wrong here,
        //  maybe should be using `ecolor::rgba::Rgba` not `ecolor::ecolor32::Color32`.
        //  Apparently it's an
        {
            profile_scope!("correct_gamma");
            const GAMMA: Channel = 2.2;
            const INV_GAMMA: Channel = 1.0 / GAMMA;

            // Gamma correction is per-channel, not per-pixel
            self.deref_mut().into_par_iter().for_each(|c| *c = c.powf(INV_GAMMA));
        }
        // TODO: Pool the images?
        let mut output = {
            profile_scope!("alloc_output");
            ColorImage {
                size: [self.width(), self.height()],
                // I hope the compiler optimizes this
                pixels: vec![Color32::default(); self.len()],
            }
        };

        // Convert each pixel into array of u8 channels and write to output
        {
            profile_scope!("convert_channels_u8");
            self.indexed_iter().for_each(|((x, y), col)| {
                let [r, g, b] = col.0.map(|c| (c * 255.0) as u8);
                output[(x, y)] = Color32::from_rgb(r, g, b)
            });
        };

        output
    }
}
