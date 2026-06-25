use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;

pub struct Rotate180;

impl Rotate180 {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for Rotate180 {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        *image = image.rotate180();

        Ok(())
    }
}
