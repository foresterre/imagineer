use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::image::buffer::ConvertBuffer;
use sic_core::{SicImage, image};

pub struct Grayscale;

impl Grayscale {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for Grayscale {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => *image = image.grayscale(),
            SicImage::Animated(image) => grayscale_animated_image(image.frames_mut()),
        }

        Ok(())
    }
}

fn grayscale_animated_image(frames: &mut [image::Frame]) {
    frames.par_iter_mut().for_each(|frame| {
        *frame.buffer_mut() = image::imageops::grayscale(frame.buffer_mut()).convert();
    });
}
