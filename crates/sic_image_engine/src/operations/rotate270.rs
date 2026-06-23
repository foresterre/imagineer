use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;

pub struct Rotate270;

impl Rotate270 {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for Rotate270 {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        *image = image.rotate270();

        Ok(())
    }
}
