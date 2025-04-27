use crate::encode_settings;
use sic_core::image::ImageError;
use sic_core::{SicCoreError, image};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SicIoError {
    #[error(transparent)]
    SicCoreError(#[from] SicCoreError),

    #[error(transparent)]
    ImageError(#[from] ImageError),

    #[error(transparent)]
    Io(std::io::Error),

    #[error(transparent)]
    Encoding(#[from] EncodingError),

    #[error(transparent)]
    FormatError(FormatError),

    #[error(
        "An input image should be given by providing a path using the input argument or by \
         piping an image to the stdin."
    )]
    NoInputImage,

    #[error(
        "Unable to extract frame {0} from the (animated) image; please use a frame index between 0 and {1}."
    )]
    NoSuchFrame(usize, usize),

    #[error("An animated image was expected, but a static image was given")]
    NotAnAnimatedImage,

    #[error(transparent)]
    UnknownImageFormat(UnknownImageFormatError),
}

#[cfg(test)]
impl PartialEq for SicIoError {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

#[derive(Debug, Error)]
pub enum FormatError {
    #[error(transparent)]
    JPEGQuality(encode_settings::jpeg::JpegQualityError),

    #[error(
        "The GIF repeat value has to be either a positive integer < 65536, 'infinite' or 'never'"
    )]
    GIFRepeatInvalidValue,
}

#[derive(Debug, Error)]
pub enum UnknownImageFormatError {
    #[error("No supported image format found for the file extension of path '{0}'")]
    FileExtension(PathBuf),

    #[error("No supported image format found for the identifier '{0}'")]
    Identifier(String),
}

#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("Unable to encode animated image using '{0:?}' image format")]
    AnimatedImageUnsupported(image::ImageFormat),
}
