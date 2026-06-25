use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::color_quant::NeuQuant;
use sic_core::image::DynamicImage;
use sic_core::image::imageops::colorops::dither;

const MIN_COLORS: u32 = 64;
const SAMPLE_FACTOR_RANGE: std::ops::RangeInclusive<u32> = 1..=30;

pub struct DitherColor {
    colors: u32,
    sample_factor: u32,
}

impl DitherColor {
    pub fn new(colors: u32, sample_factor: u32) -> Self {
        Self {
            colors,
            sample_factor,
        }
    }
}

impl ImageOperation for DitherColor {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        if self.colors < MIN_COLORS {
            return Err(SicImageEngineError::DitherColorsOutOfRange(self.colors));
        }

        if !SAMPLE_FACTOR_RANGE.contains(&self.sample_factor) {
            return Err(SicImageEngineError::DitherSampleFactorOutOfRange(
                self.sample_factor,
            ));
        }

        let mut buffer = image.to_rgba8();
        // NeuQuant trains a palette on the image's own pixels.
        let quant = NeuQuant::new(
            self.sample_factor as i32,
            self.colors as usize,
            buffer.as_raw(),
        );
        dither(&mut buffer, &quant);
        *image = DynamicImage::ImageRgba8(buffer);

        Ok(())
    }
}
