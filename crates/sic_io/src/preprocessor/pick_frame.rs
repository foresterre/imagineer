use crate::errors::SicIoError;
use crate::preprocessor::Preprocess;
use sic_core::{SicImage, image};

pub struct PickFramePreprocessor {
    image_format: image::ImageFormat,
}

impl PickFramePreprocessor {
    pub fn new(image_format: image::ImageFormat) -> Self {
        Self { image_format }
    }
}

impl Preprocess for PickFramePreprocessor {
    type Err = SicIoError;

    fn preprocess(&self, image: SicImage) -> Result<SicImage, Self::Err> {
        match image {
            SicImage::Animated(animated) if self.image_format != image::ImageFormat::Gif => {
                eprintln!(
                    "WARN: Unable to encode animated image buffer with format '{:?}': encoding first frame only",
                    self.image_format
                );
                let image = animated.try_into_static_image(0)?;
                Ok(SicImage::Static(image))
            }
            ok => Ok(ok),
        }
    }
}
