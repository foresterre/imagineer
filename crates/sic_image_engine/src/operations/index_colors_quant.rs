use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::color_quant::NeuQuant;
use sic_core::image::DynamicImage;
use sic_core::image::imageops::colorops::index_colors;

const MIN_COLORS: u32 = 64;
const SAMPLE_FACTOR_RANGE: std::ops::RangeInclusive<u32> = 1..=30;

pub struct IndexColorsQuant {
    colors: u32,
    sample_factor: u32,
}

impl IndexColorsQuant {
    pub fn new(colors: u32, sample_factor: u32) -> Self {
        Self {
            colors,
            sample_factor,
        }
    }
}

impl ImageOperation for IndexColorsQuant {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        if self.colors < MIN_COLORS {
            return Err(SicImageEngineError::IndexColorsQuantColorsOutOfRange(
                self.colors,
            ));
        }

        if !SAMPLE_FACTOR_RANGE.contains(&self.sample_factor) {
            return Err(SicImageEngineError::IndexColorsQuantSampleFactorOutOfRange(
                self.sample_factor,
            ));
        }

        let buffer = image.to_rgba8();
        // NeuQuant trains a palette on the image's own pixels.
        let quant = NeuQuant::new(
            self.sample_factor as i32,
            self.colors as usize,
            buffer.as_raw(),
        );
        // The result holds the palette index of each pixel, not the palette color itself.
        let indices = index_colors(&buffer, &quant);
        *image = DynamicImage::ImageLuma8(indices);

        Ok(())
    }
}
