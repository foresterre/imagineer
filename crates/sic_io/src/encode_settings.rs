use crate::encode_settings::gif::RepeatAnimation;
use crate::encode_settings::jpeg::JpegQuality;
use sic_core::image;

pub mod gif;
pub mod jpeg;
pub mod pnm;

pub struct EncodeSettings {
    pub pnm_sample_encoding: image::codecs::pnm::SampleEncoding,
    pub jpeg_quality: JpegQuality,
    pub repeat_animation: RepeatAnimation,
}

impl Default for EncodeSettings {
    fn default() -> Self {
        Self {
            pnm_sample_encoding: image::codecs::pnm::SampleEncoding::Binary,
            jpeg_quality: JpegQuality::default(),
            repeat_animation: RepeatAnimation::default(),
        }
    }
}
