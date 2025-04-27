#![deny(clippy::all)]

//! This crate contains a re-export of the image crate and a few fundamental data types which all
//! or almost all sic components interact with. The re-export of the image crate makes sure all
//! components depend on the same version of the image crate, which is required for binary
//! compatibility.

/// The re-export of image ensures all crates use the same version.
pub use {ab_glyph, image, imageproc};

use image::DynamicImage;
use std::convert::TryFrom;

mod animated;
mod errors;

pub use animated::AnimatedImage;

pub use errors::SicCoreError;

/// The fundamental image data structure in `imagineer`.
/// An image can either be animated, in which case it consists of a collection of `image::Frame` frames,
/// or static, in which case it's represented as an `image::DynamicImage`.
#[derive(Clone, Debug)]
pub enum SicImage {
    Animated(AnimatedImage),
    Static(DynamicImage),
}

// Should not be used outside of tests, as it doesn't support animated images
#[doc(hidden)]
impl AsRef<DynamicImage> for SicImage {
    fn as_ref(&self) -> &DynamicImage {
        match self {
            Self::Animated(_) => unimplemented!(),
            Self::Static(image) => image,
        }
    }
}

impl From<DynamicImage> for SicImage {
    fn from(item: DynamicImage) -> Self {
        Self::Static(item)
    }
}

impl TryFrom<SicImage> for DynamicImage {
    type Error = SicCoreError;

    fn try_from(value: SicImage) -> Result<Self, Self::Error> {
        match value {
            SicImage::Static(image) => Ok(image),
            _ => Err(SicCoreError::RequiresStaticImage),
        }
    }
}
