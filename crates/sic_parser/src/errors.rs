use sic_image_engine::errors::SicImageEngineError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SicParserError {
    #[error("unable to parse filter type: {0}")]
    FilterTypeError(SicImageEngineError),

    #[error("unable to parse script: {0}")]
    ParseError(String),

    #[error("parsing failed: operation doesn't exist")]
    UnknownOperationError,

    #[error("unable to parse value '{0}'")]
    ValueParsingError(String),
}
