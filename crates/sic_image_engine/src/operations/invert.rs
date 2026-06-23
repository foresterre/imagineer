use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;

pub struct Invert;

impl Invert {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for Invert {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        image.invert();

        Ok(())
    }
}
