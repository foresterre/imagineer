use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;
use sic_core::image::imageops::colorops::{BiLevel, index_colors};

pub struct IndexColors;

impl IndexColors {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for IndexColors {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        let buffer = image.to_luma8();
        // The result holds the palette index of each pixel, not the palette color itself.
        let indices = index_colors(&buffer, &BiLevel);
        *image = DynamicImage::ImageLuma8(indices);

        Ok(())
    }
}
