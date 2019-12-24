use sic_image_engine::errors::SicImageEngineError;
use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SicParserError {
    #[error("sic parser error > {0}")]
    FilterTypeError(SicImageEngineError),

    #[error("sic parser error > {0}")]
    PestGrammarError(String),

    #[error("{0}")]
    OperationError(OperationParamError),

    #[error("Parse failed: Operation doesn't exist")]
    UnknownOperationError,

    #[error("Unable to parse value '{0}'")]
    ValueParsingError(String),
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum OperationParamError {
    #[error(
        "Unable to parse `set` environment command. Error: expected a single `set` inner element."
    )]
    SetEnvironment,

    #[error("Unable to parse `set` environment command. Error on element: {0}")]
    SetEnvironmentElement(String),

    #[error("Unable to parse 'set resize_sampling_filter' option: {0}")]
    SetResizeSamplingFilter(String),

    #[error(
        "Unable to parse `del` environment command. Error: expected a single `del` inner element."
    )]
    UnsetEnvironment,

    #[error("Unable to parse `del` environment command. Error on element: {0}")]
    UnsetEnvironmentElement(String),
}
