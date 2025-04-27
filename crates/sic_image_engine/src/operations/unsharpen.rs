use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::{SicImage, image};

pub struct Unsharpen {
    sigma: f32,
    threshold: i32,
}

impl Unsharpen {
    pub fn new(sigma: f32, threshold: i32) -> Self {
        Self { sigma, threshold }
    }
}

impl ImageOperation for Unsharpen {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => *image = image.unsharpen(self.sigma, self.threshold),
            SicImage::Animated(image) => {
                unsharpen_animated_image(image.frames_mut(), self.sigma, self.threshold)
            }
        }

        Ok(())
    }
}

fn unsharpen_animated_image(frames: &mut [image::Frame], sigma: f32, threshold: i32) {
    frames.par_iter_mut().for_each(|frame| {
        *frame.buffer_mut() = image::imageops::unsharpen(frame.buffer_mut(), sigma, threshold);
    });
}
