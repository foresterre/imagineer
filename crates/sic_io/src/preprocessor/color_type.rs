use crate::encode::dynamic::DynamicImageFormat;
use crate::errors::SicIoError;
use crate::preprocessor::Preprocess;
use sic_core::SicImage;
use sic_core::image::DynamicImage;
use sic_core::image::codecs::pnm::PnmSubtype;

pub struct ColorTypePreprocessor {
    format: DynamicImageFormat,
}

impl ColorTypePreprocessor {
    pub fn new(format: DynamicImageFormat) -> Self {
        Self { format }
    }
}

impl Preprocess for ColorTypePreprocessor {
    type Err = SicIoError;

    fn preprocess(&self, image: SicImage) -> Result<SicImage, SicIoError> {
        match image {
            // JPEG
            // - Jpeg encoder only supports L8 or Rgb8.
            // https://github.com/image-rs/image/blob/d71046727387b627e63effe7c56790bd355ec5ba/src/codecs/jpeg/encoder.rs#L454
            //
            // GIF
            // - Gif image::Frame only accepts RgbaImage
            //
            // PNM
            // - PNM Bitmap encoder supports: L8 or L1
            // - PNM Graymap encoder supports: L8
            // - PNM Pixmap encoder supports: Rgb8
            // https://github.com/image-rs/image/blob/d71046727387b627e63effe7c56790bd355ec5ba/src/codecs/pnm/encoder.rs
            SicImage::Static(image) => match self.format {
                DynamicImageFormat::Farbfeld => Ok(to_rgba16(image)),
                DynamicImageFormat::Jpeg => Ok(to_rgb8(image)),
                DynamicImageFormat::Gif => Ok(to_rgba8(image)),
                DynamicImageFormat::Pnm { subtype } => match subtype {
                    PnmSubtype::Bitmap(_) => Ok(to_l8(image)),
                    PnmSubtype::Graymap(_) => Ok(to_l8(image)),
                    PnmSubtype::Pixmap(_) => Ok(to_rgb8(image)),
                    _ => Ok(SicImage::Static(image)),
                },
                _ => Ok(SicImage::Static(image)),
            },
            SicImage::Animated(image) => Ok(SicImage::Animated(image)),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ColorTypeAdjustment {
    // Usually the default
    #[default]
    Enabled,
    Disabled,
}

impl From<bool> for ColorTypeAdjustment {
    fn from(value: bool) -> Self {
        if value {
            ColorTypeAdjustment::Enabled
        } else {
            ColorTypeAdjustment::Disabled
        }
    }
}

impl ColorTypeAdjustment {
    pub fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled)
    }
}

fn to_l8(image: DynamicImage) -> SicImage {
    if let ok @ DynamicImage::ImageLuma8(_) = image {
        SicImage::Static(ok)
    } else {
        SicImage::Static(DynamicImage::ImageLuma8(image.to_luma8()))
    }
}

fn to_rgb8(image: DynamicImage) -> SicImage {
    if let ok @ DynamicImage::ImageRgb8(_) = image {
        SicImage::Static(ok)
    } else {
        SicImage::Static(DynamicImage::ImageRgb8(image.to_rgb8()))
    }
}

fn to_rgba8(image: DynamicImage) -> SicImage {
    if let ok @ DynamicImage::ImageRgb8(_) = image {
        SicImage::Static(ok)
    } else {
        SicImage::Static(DynamicImage::ImageRgba8(image.to_rgba8()))
    }
}

fn to_rgba16(image: DynamicImage) -> SicImage {
    if let ok @ DynamicImage::ImageRgba16(_) = image {
        SicImage::Static(ok)
    } else {
        SicImage::Static(DynamicImage::ImageRgba16(image.to_rgba16()))
    }
}
