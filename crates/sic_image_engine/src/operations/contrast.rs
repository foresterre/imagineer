use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;

pub struct Contrast {
    contrast: f32,
}

impl Contrast {
    pub fn new(contrast: f32) -> Self {
        Self { contrast }
    }
}

impl ImageOperation for Contrast {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        *image = image.adjust_contrast(self.contrast);

        Ok(())
    }
}
