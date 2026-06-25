use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;
use sic_core::image::imageops::colorops::{BiLevel, dither};

pub struct Dither;

impl Dither {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for Dither {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        let mut buffer = image.to_luma8();
        dither(&mut buffer, &BiLevel);
        *image = DynamicImage::ImageLuma8(buffer);

        Ok(())
    }
}
