use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;

pub struct Rotate90;

impl Rotate90 {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for Rotate90 {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        *image = image.rotate90();

        Ok(())
    }
}
