use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;

pub struct Unsharpen {
    sigma: f32,
    threshold: i32,
}

impl Unsharpen {
    pub fn new(sigma: f32, threshold: i32) -> Self {
        Self { sigma, threshold }
    }
}

impl ImageOperation for Unsharpen {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        if self.sigma <= 0.0 {
            return Err(SicImageEngineError::BlurSigmaNotPositive(self.sigma));
        }

        *image = image.unsharpen(self.sigma, self.threshold);

        Ok(())
    }
}
