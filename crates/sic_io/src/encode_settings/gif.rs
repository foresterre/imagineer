use crate::errors::{FormatError, SicIoError};
use sic_core::image;

#[derive(Clone, Copy, Debug, Default)]
pub enum RepeatAnimation {
    Finite(u16),
    #[default]
    Infinite,
    Never,
}

impl RepeatAnimation {
    pub fn try_from_str(input: &str) -> Result<Self, SicIoError> {
        match input {
            "infinite" => Ok(Self::Infinite),
            "never" => Ok(Self::Never),
            elsy => elsy
                .parse::<u16>()
                .map(Self::Finite)
                .map_err(|_| SicIoError::FormatError(FormatError::GIFRepeatInvalidValue)),
        }
    }
}

impl From<RepeatAnimation> for image::codecs::gif::Repeat {
    fn from(value: RepeatAnimation) -> Self {
        match value {
            RepeatAnimation::Finite(v) => image::codecs::gif::Repeat::Finite(v),
            RepeatAnimation::Infinite => image::codecs::gif::Repeat::Infinite,
            RepeatAnimation::Never => image::codecs::gif::Repeat::Finite(0),
        }
    }
}
