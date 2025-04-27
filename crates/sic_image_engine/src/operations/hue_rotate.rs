use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::{SicImage, image};

pub struct HueRotate {
    degree: i32,
}

impl HueRotate {
    pub fn new(degree: i32) -> Self {
        Self { degree }
    }
}

impl ImageOperation for HueRotate {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => *image = image.huerotate(self.degree),
            SicImage::Animated(image) => hue_rotate_animated_image(image.frames_mut(), self.degree),
        }

        Ok(())
    }
}

fn hue_rotate_animated_image(frames: &mut [image::Frame], degree: i32) {
    frames.par_iter_mut().for_each(|frame| {
        *frame.buffer_mut() = image::imageops::huerotate(frame.buffer_mut(), degree);
    });
}
