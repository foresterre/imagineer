use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;

pub struct Grayscale;

impl Grayscale {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for Grayscale {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        *image = image.grayscale();

        Ok(())
    }
}
