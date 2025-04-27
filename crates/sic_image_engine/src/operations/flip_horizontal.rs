use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::{SicImage, image};

pub struct FlipHorizontal;

impl FlipHorizontal {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for FlipHorizontal {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => *image = image.fliph(),
            SicImage::Animated(image) => flip_horizontal_animated_image(image.frames_mut()),
        }

        Ok(())
    }
}

fn flip_horizontal_animated_image(frames: &mut [image::Frame]) {
    frames.par_iter_mut().for_each(|frame| {
        image::imageops::flip_horizontal_in_place(frame.buffer_mut());
    });
}
