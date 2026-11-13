use crate::TResult;
use crate::errors::{InternalErrorSource, SicCliOpsError};
use std::str::FromStr;

/// The enumeration of all supported operations.
#[derive(
    Debug, Copy, Clone, Hash, Eq, PartialEq, IntoStaticStr, EnumIter, EnumString, VariantNames,
)]
#[strum(serialize_all = "kebab_case")]
pub enum OperationId {
    // image operations
    Blur,
    Brighten,
    Contrast,
    Crop,
    Diff,
    Dither,
    DitherQuant,
    DrawText,
    Filter3x3,
    FlipHorizontal,
    FlipVertical,
    Grayscale,
    HueRotate,
    HorizontalGradient,
    IndexColors,
    IndexColorsQuant,
    Invert,
    Overlay,
    Resize,
    Rotate90,
    Rotate180,
    Rotate270,
    Threshold,
    Unsharpen,
    VerticalGradient,

    // modifiers
    PreserveAspectRatio,
    SamplingFilter,
}

impl OperationId {
    pub fn variants() -> &'static [&'static str] {
        use strum::VariantNames;
        OperationId::VARIANTS
    }

    /// A string representation for each operation.
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    pub fn try_from_name(input: &str) -> TResult<Self> {
        OperationId::from_str(input)
            .map_err(|_err| SicCliOpsError::InternalError(InternalErrorSource::NoMatchingOperator))
    }

    /// Provides the number of arguments an operation takes.
    /// Used to unify arguments together.
    /// E.g. (without accounting for the requirement of having incremental indices as well),
    ///     say we receive for resize the values 10, 20, 100 and 100. With the number of values we know
    ///     that each resize operation takes two arguments, not four. So it could be that there are
    ///     two operations, namely `resize 10 20` and `resize 100 100`. We do need to take some other
    ///     conditions into account, but they are not relevant for this particular method =).
    pub fn takes_number_of_arguments(self) -> usize {
        match self {
            // image operations
            OperationId::Blur => 1,
            OperationId::Brighten => 1,
            OperationId::Contrast => 1,
            OperationId::Crop => 4,
            OperationId::Diff => 1,
            OperationId::Dither => 0,
            OperationId::DitherQuant => 2,
            OperationId::DrawText => 5,
            OperationId::Filter3x3 => 9,
            OperationId::FlipHorizontal => 0,
            OperationId::FlipVertical => 0,
            OperationId::Grayscale => 0,
            OperationId::HueRotate => 1,
            OperationId::HorizontalGradient => 2,
            OperationId::IndexColors => 0,
            OperationId::IndexColorsQuant => 2,
            OperationId::Invert => 0,
            OperationId::Overlay => 3,
            OperationId::Resize => 2,
            OperationId::Rotate90 => 0,
            OperationId::Rotate180 => 0,
            OperationId::Rotate270 => 0,
            OperationId::Threshold => 0,
            OperationId::Unsharpen => 2,
            OperationId::VerticalGradient => 2,

            // image operation modifiers
            OperationId::PreserveAspectRatio => 1,
            OperationId::SamplingFilter => 1,
        }
    }
}
