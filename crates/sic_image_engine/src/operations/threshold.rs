use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::{image::DynamicImage, imageproc};

pub struct Threshold;

impl Threshold {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for Threshold {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        *image = threshold_image(image);

        Ok(())
    }
}

fn threshold_image(img: &mut DynamicImage) -> DynamicImage {
    let gray_image = img.to_luma8();
    let best_threshold = imageproc::contrast::otsu_level(&gray_image);
    let out = imageproc::contrast::threshold(
        &gray_image,
        best_threshold,
        imageproc::contrast::ThresholdType::Truncate,
    );
    DynamicImage::ImageLuma8(out)
}
