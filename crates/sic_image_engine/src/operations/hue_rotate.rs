use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;

pub struct HueRotate {
    degree: i32,
}

impl HueRotate {
    pub fn new(degree: i32) -> Self {
        Self { degree }
    }
}

impl ImageOperation for HueRotate {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        *image = image.huerotate(self.degree);

        Ok(())
    }
}
