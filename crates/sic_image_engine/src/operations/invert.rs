use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::{SicImage, image};

pub struct Invert;

impl Invert {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for Invert {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => image.invert(),
            SicImage::Animated(image) => invert_animated_image(image.frames_mut()),
        }

        Ok(())
    }
}

fn invert_animated_image(frames: &mut [image::Frame]) {
    frames.par_iter_mut().for_each(|frame| {
        image::imageops::invert(frame.buffer_mut());
    });
}
