#![allow(clippy::new_without_default)]
use crate::errors::SicImageEngineError;
use sic_core::SicImage;

pub mod blur;
pub mod brighten;
pub mod contrast;
pub mod crop;
pub mod diff;

pub mod draw_text;
pub mod filter3x3;
pub mod flip_horizontal;
pub mod flip_vertical;
pub mod grayscale;
pub mod horizontal_gradient;
pub mod hue_rotate;
pub mod invert;
pub mod overlay;
pub mod resize;
pub mod rotate180;
pub mod rotate270;
pub mod rotate90;

pub mod threshold;
pub mod unsharpen;
pub mod vertical_gradient;

pub trait ImageOperation {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError>;
}
