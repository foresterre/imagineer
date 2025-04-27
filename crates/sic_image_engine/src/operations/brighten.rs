use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::{SicImage, image};

pub struct Brighten {
    amount: i32,
}

impl Brighten {
    pub fn new(amount: i32) -> Self {
        Self { amount }
    }
}

impl ImageOperation for Brighten {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => *image = image.brighten(self.amount),
            SicImage::Animated(image) => brighten_animated_image(image.frames_mut(), self.amount),
        }

        Ok(())
    }
}

fn brighten_animated_image(frames: &mut [image::Frame], amount: i32) {
    frames.par_iter_mut().for_each(|frame| {
        *frame.buffer_mut() = image::imageops::brighten(frame.buffer_mut(), amount);
    });
}
