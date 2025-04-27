use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use crate::wrapper::overlay::OverlayInputs;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use sic_core::image::{DynamicImage, imageops};
use sic_core::{SicImage, image};
use std::convert::TryFrom;

pub struct Overlay<'overlay> {
    inputs: &'overlay OverlayInputs,
}

impl<'overlay> Overlay<'overlay> {
    pub fn new(inputs: &'overlay OverlayInputs) -> Self {
        Self { inputs }
    }
}

impl ImageOperation for Overlay<'_> {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => overlay_static(image, self.inputs),
            SicImage::Animated(image) => overlay_animated_image(image.frames_mut(), self.inputs),
        }
    }
}

fn overlay_animated_image(
    frames: &mut [image::Frame],
    inputs: &OverlayInputs,
) -> Result<(), SicImageEngineError> {
    // Open matching image
    let overlay_image = inputs.image_path().open_image()?;
    let (x, y) = inputs.position();

    match overlay_image {
        SicImage::Static(image) => overlay_animated_with_static(frames, &image, x, y),
        SicImage::Animated(other) => overlay_animated_with_animated(frames, other.frames(), x, y),
    }

    Ok(())
}

fn overlay_animated_with_animated(
    frames: &mut [image::Frame],
    other: &[image::Frame],
    x: i64,
    y: i64,
) {
    frames.par_iter_mut().zip(other).for_each(|(lhs, rhs)| {
        imageops::overlay(lhs.buffer_mut(), rhs.buffer(), x, y);
    });
}

fn overlay_animated_with_static(frames: &mut [image::Frame], other: &DynamicImage, x: i64, y: i64) {
    frames.par_iter_mut().for_each(|frame| {
        imageops::overlay(frame.buffer_mut(), other, x, y);
    });
}

fn overlay_static(
    image: &mut DynamicImage,
    overlay: &OverlayInputs,
) -> Result<(), SicImageEngineError> {
    let overlay_image = overlay.image_path().open_image()?;
    let overlay_image = DynamicImage::try_from(overlay_image)?;

    let (x, y) = overlay.position();
    imageops::overlay(image, &overlay_image, x, y);

    Ok(())
}
