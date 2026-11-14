use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::{image::DynamicImage, imageproc};

pub struct EqualizeHistogram;

impl EqualizeHistogram {
    pub fn new() -> Self {
        Self
    }
}

impl ImageOperation for EqualizeHistogram {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        let mut gray = image.to_luma8();
        imageproc::contrast::equalize_histogram_mut(&mut gray);
        *image = DynamicImage::ImageLuma8(gray);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sic_core::image::{ColorType, GrayImage, Rgba, RgbaImage};

    #[test]
    fn spreads_intensities_over_full_range() {
        let gray = GrayImage::from_raw(2, 2, vec![0, 0, 100, 200]).unwrap();
        let mut image = DynamicImage::ImageLuma8(gray);

        EqualizeHistogram::new().apply_to_frame(&mut image).unwrap();

        assert_eq!(image.as_bytes(), &[127, 127, 191, 255]);
    }

    #[test]
    fn outputs_grayscale() {
        let mut image =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(3, 2, Rgba([10, 20, 30, 40])));

        EqualizeHistogram::new().apply_to_frame(&mut image).unwrap();

        assert_eq!(image.color(), ColorType::L8);
        assert_eq!(image.width(), 3);
        assert_eq!(image.height(), 2);
    }

    #[test]
    fn empty_image() {
        let mut image = DynamicImage::new_rgb8(0, 0);

        EqualizeHistogram::new().apply_to_frame(&mut image).unwrap();

        assert_eq!(image.color(), ColorType::L8);
        assert!(image.as_bytes().is_empty());
    }
}
