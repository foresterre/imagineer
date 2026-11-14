use crate::types::Type;
use sic_image_engine::errors::SicImageEngineError;
use std::fmt::Display;
use thiserror::Error;
use winnow::error::{ContextError, StrContext};

#[derive(Debug, Error)]
pub enum SicParserError {
    #[error(transparent)]
    Syntax(#[from] SyntaxError),

    #[error(transparent)]
    Resolve(#[from] ResolveError),
}

#[derive(Debug, Error)]
#[error("syntax error at line {line}, column {column}: {message}")]
pub struct SyntaxError {
    line: usize,
    column: usize,
    message: String,
}

impl SyntaxError {
    pub fn new(input: &str, offset: usize, error: &ContextError) -> Self {
        let (before, after) = input.split_at_checked(offset).unwrap_or((input, ""));

        let line = before.matches('\n').count() + 1;
        let column = before
            .rsplit('\n')
            .next()
            .map_or(0, |line| line.chars().count())
            + 1;

        let label = error.context().find_map(|context| match context {
            StrContext::Label(label) => Some(*label),
            _ => None,
        });
        let expected = error
            .context()
            .filter_map(|context| match context {
                StrContext::Expected(value) => Some(value.to_string()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let found = after
            .chars()
            .next()
            .map_or_else(|| "end of input".to_string(), |c| format!("{c:?}"));

        let mut message = String::new();
        if let Some(label) = label {
            message.push_str(&format!("invalid {label}: "));
        }
        if !expected.is_empty() {
            message.push_str(&format!("expected {}, ", expected.join(" or ")));
        }
        message.push_str(&format!("found {found}"));

        Self {
            line,
            column,
            message,
        }
    }
}

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("unknown operation `{name}`")]
    UnknownOperation { name: String },

    #[error("unknown option `{name}`, expected `sampling-filter` or `preserve-aspect-ratio`")]
    UnknownOption { name: String },

    #[error("wrong number of arguments for `{function}`: expected {expected}, found {found}")]
    ArgumentCount {
        function: String,
        expected: usize,
        found: usize,
    },

    #[error("invalid argument `{param}` for `{function}`")]
    Argument {
        function: String,
        param: &'static str,
        source: Box<ArgumentError>,
    },

    #[error("invalid value for option `{option}`")]
    OptionValue {
        option: String,
        source: Box<ArgumentError>,
    },
}

#[derive(Debug, Error)]
pub enum ArgumentError {
    #[error("expected {expected}, found {found}")]
    Mismatch { expected: Type, found: String },

    #[error("{lexeme} doesn't fit in {expected}")]
    OutOfRange { expected: Type, lexeme: String },

    #[error(transparent)]
    FilterType(SicImageEngineError),

    #[error(transparent)]
    Syntax(#[from] SyntaxError),

    #[error(transparent)]
    InCall(Box<ResolveError>),
}

impl ArgumentError {
    pub fn mismatch(expected: Type, found: impl Display) -> Self {
        Self::Mismatch {
            expected,
            found: found.to_string(),
        }
    }
}

impl From<ResolveError> for ArgumentError {
    fn from(error: ResolveError) -> Self {
        Self::InCall(Box::new(error))
    }
}
