use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::{SicImage, image};

pub struct Rotate90;

impl Rotate90 {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for Rotate90 {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => *image = image.rotate90(),
            SicImage::Animated(image) => rotate90_animated_image(image.frames_mut()),
        }

        Ok(())
    }
}

fn rotate90_animated_image(frames: &mut [image::Frame]) {
    frames.par_iter_mut().for_each(|frame| {
        *frame.buffer_mut() = image::imageops::rotate90(frame.buffer_mut());
    });
}
