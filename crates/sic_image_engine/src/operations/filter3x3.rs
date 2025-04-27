use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::{SicImage, image};

pub struct Filter3x3<'kernel> {
    kernel: &'kernel [f32; 9],
}

impl<'kernel> Filter3x3<'kernel> {
    pub fn new(kernel: &'kernel [f32; 9]) -> Self {
        Self { kernel }
    }
}

impl ImageOperation for Filter3x3<'_> {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => *image = image.filter3x3(self.kernel),
            SicImage::Animated(image) => filter3x3_animated_image(image.frames_mut(), self.kernel),
        }

        Ok(())
    }
}

fn filter3x3_animated_image(frames: &mut [image::Frame], kernel: &[f32; 9]) {
    frames.par_iter_mut().for_each(|frame| {
        *frame.buffer_mut() = image::imageops::filter3x3(frame.buffer_mut(), kernel);
    });
}
