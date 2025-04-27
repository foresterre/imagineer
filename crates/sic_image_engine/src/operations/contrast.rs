use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::{SicImage, image};

pub struct Contrast {
    contrast: f32,
}

impl Contrast {
    pub fn new(contrast: f32) -> Self {
        Self { contrast }
    }
}

impl ImageOperation for Contrast {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => *image = image.adjust_contrast(self.contrast),
            SicImage::Animated(image) => contrast_animated_image(image.frames_mut(), self.contrast),
        }

        Ok(())
    }
}

fn contrast_animated_image(frames: &mut [image::Frame], contrast: f32) {
    frames.par_iter_mut().for_each(|frame| {
        *frame.buffer_mut() = image::imageops::contrast(frame.buffer_mut(), contrast);
    });
}
