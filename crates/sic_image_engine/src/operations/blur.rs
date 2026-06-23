use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;

pub struct Blur {
    sigma: f32,
}

impl Blur {
    pub fn new(sigma: f32) -> Self {
        Self { sigma }
    }
}

impl ImageOperation for Blur {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        if self.sigma <= 0.0 {
            return Err(SicImageEngineError::BlurSigmaNotPositive(self.sigma));
        }

        *image = image.blur(self.sigma);

        Ok(())
    }
}
