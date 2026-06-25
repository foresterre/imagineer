use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;

pub struct Brighten {
    amount: i32,
}

impl Brighten {
    pub fn new(amount: i32) -> Self {
        Self { amount }
    }
}

impl ImageOperation for Brighten {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        *image = image.brighten(self.amount);

        Ok(())
    }
}
