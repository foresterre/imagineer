#![allow(clippy::new_without_default)]

use crate::errors::SicImageEngineError;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::image::DynamicImage;
use sic_core::{SicImage, image};
use std::mem;

pub mod blur;
pub mod brighten;
pub mod contrast;
pub mod crop;
pub mod diff;

pub mod draw_text;
pub mod filter3x3;
pub mod flip_horizontal;
pub mod flip_vertical;
pub mod grayscale;
pub mod horizontal_gradient;
pub mod hue_rotate;
pub mod invert;
pub mod overlay;
pub mod resize;
pub mod rotate180;
pub mod rotate270;
pub mod rotate90;

pub mod threshold;
pub mod unsharpen;
pub mod vertical_gradient;

pub trait ImageOperation: Sync {
    /// Apply an image operation on a [`DynamicImage`].
    ///
    /// This method enables the default implementation of the [`ImageOperation::apply_operation`].
    /// It is directly applied to static images, since they're internally represented as [`DynamicImage`].
    /// For animated images, we instead iterate over each frame in parallel, and apply the operation
    /// on each individual frame.
    ///
    /// To opt out of the default implementation, override the [`ImageOperation::apply_operation`]
    /// method. For static images, you probably still will want to call `self.apply_to_frame(image)`.
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError>;

    /// This is the actual operation used by the image engine. A [`SicImage`] is provided as argument,
    /// which can be either the static or the animated variant. The static variant is a [`DynamicImage`],
    /// while the animated variant consists of a vec of [`image::Frame`]'s, wrapped into a wrapper
    /// struct named [`AnimatedImage`].
    ///
    /// When overriding this method, for static images, you probably want to call `self.apply_to_frame(image)`.
    ///
    /// [`AnimatedImage`]: sic_core::AnimatedImage
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => self.apply_to_frame(image),
            SicImage::Animated(animated) => apply_operation_to_frames(self, animated.frames_mut()),
        }
    }
}

fn apply_operation_to_frames<O: ImageOperation + ?Sized>(
    operation: &O,
    frames: &mut [image::Frame],
) -> Result<(), SicImageEngineError> {
    frames.par_iter_mut().try_for_each(|frame| {
        let mut image = DynamicImage::ImageRgba8(mem::take(frame.buffer_mut()));
        operation.apply_to_frame(&mut image)?;

        // Free (i.e. no copy) for AnimatedImage frames, since an image::Frame is a `RgbaImage`.
        *frame.buffer_mut() = image.into_rgba8();

        Ok(())
    })
}
