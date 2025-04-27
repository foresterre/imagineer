use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::{SicImage, image};

pub struct Blur {
    sigma: f32,
}

impl Blur {
    pub fn new(sigma: f32) -> Self {
        Self { sigma }
    }
}

impl ImageOperation for Blur {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => *image = image.blur(self.sigma),
            SicImage::Animated(image) => blur_animated_image(image.frames_mut(), self.sigma),
        }

        Ok(())
    }
}

fn blur_animated_image(frames: &mut [image::Frame], sigma: f32) {
    frames.par_iter_mut().for_each(|frame| {
        *frame.buffer_mut() = image::imageops::blur(frame.buffer_mut(), sigma);
    });
}
