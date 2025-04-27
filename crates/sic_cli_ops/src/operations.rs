use crate::errors::{InternalErrorSource, SicCliOpsError};
use crate::TResult;
use sic_image_engine::engine::{EnvItem, Instr};
use sic_image_engine::wrapper::filter_type::FilterTypeWrap;
use sic_image_engine::wrapper::image_path::ImageFromPath;
use sic_image_engine::wrapper::overlay::OverlayInputs;
use sic_image_engine::ImgOp;
use sic_parser::errors::SicParserError;
use sic_parser::value_parser::{Describable, ParseInputsFromIter};
use std::fmt::Debug;
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
    DrawText,
    Filter3x3,
    FlipHorizontal,
    FlipVertical,
    Grayscale,
    HueRotate,
    HorizontalGradient,
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
            OperationId::DrawText => 5,
            OperationId::Filter3x3 => 9,
            OperationId::FlipHorizontal => 0,
            OperationId::FlipVertical => 0,
            OperationId::Grayscale => 0,
            OperationId::HueRotate => 1,
            OperationId::HorizontalGradient => 2,
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

macro_rules! parse_inputs_by_type {
    ($iterable:expr, $ty:ty) => {{
        let input: Result<$ty, SicCliOpsError> =
            ParseInputsFromIter::parse($iterable).map_err(|err| {
                SicCliOpsError::UnableToParseValueOfType {
                    err,
                    typ: stringify!($ty).to_string(),
                }
            });
        input
    }};
}

impl OperationId {
    /// Constructs instructions for image operations which are taken as input by the image engine.
    pub fn create_instruction<'a, T>(self, inputs: T) -> Result<Instr, SicCliOpsError>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
    {
        let stmt = match self {
            // image operations
            OperationId::Blur => Instr::Operation(ImgOp::Blur(parse_inputs_by_type!(inputs, f32)?)),
            OperationId::Brighten => {
                Instr::Operation(ImgOp::Brighten(parse_inputs_by_type!(inputs, i32)?))
            }
            OperationId::Contrast => {
                Instr::Operation(ImgOp::Contrast(parse_inputs_by_type!(inputs, f32)?))
            }
            OperationId::Crop => Instr::Operation(ImgOp::Crop(parse_inputs_by_type!(
                inputs,
                (u32, u32, u32, u32)
            )?)),
            OperationId::Diff => {
                Instr::Operation(ImgOp::Diff(parse_inputs_by_type!(inputs, ImageFromPath)?))
            }
            OperationId::DrawText => {
                use sic_image_engine::wrapper::draw_text_inner::DrawTextInner;
                Instr::Operation(ImgOp::DrawText(parse_inputs_by_type!(
                    inputs,
                    DrawTextInner
                )?))
            }
            OperationId::Filter3x3 => {
                Instr::Operation(ImgOp::Filter3x3(parse_inputs_by_type!(inputs, [f32; 9])?))
            }
            OperationId::FlipHorizontal => Instr::Operation(ImgOp::FlipHorizontal),
            OperationId::FlipVertical => Instr::Operation(ImgOp::FlipVertical),
            OperationId::Grayscale => Instr::Operation(ImgOp::Grayscale),
            OperationId::HueRotate => {
                Instr::Operation(ImgOp::HueRotate(parse_inputs_by_type!(inputs, i32)?))
            }
            OperationId::HorizontalGradient => {
                use sic_image_engine::wrapper::gradient_input::GradientInput;
                Instr::Operation(ImgOp::HorizontalGradient(parse_inputs_by_type!(
                    inputs,
                    GradientInput
                )?))
            }
            OperationId::Invert => Instr::Operation(ImgOp::Invert),
            OperationId::Overlay => Instr::Operation(ImgOp::Overlay(parse_inputs_by_type!(
                inputs,
                OverlayInputs
            )?)),
            OperationId::Resize => {
                Instr::Operation(ImgOp::Resize(parse_inputs_by_type!(inputs, (u32, u32))?))
            }
            OperationId::Rotate90 => Instr::Operation(ImgOp::Rotate90),
            OperationId::Rotate180 => Instr::Operation(ImgOp::Rotate180),
            OperationId::Rotate270 => Instr::Operation(ImgOp::Rotate270),
            OperationId::Threshold => Instr::Operation(ImgOp::Threshold),
            OperationId::Unsharpen => {
                Instr::Operation(ImgOp::Unsharpen(parse_inputs_by_type!(inputs, (f32, i32))?))
            }
            OperationId::VerticalGradient => {
                use sic_image_engine::wrapper::gradient_input::GradientInput;
                Instr::Operation(ImgOp::VerticalGradient(parse_inputs_by_type!(
                    inputs,
                    GradientInput
                )?))
            }
            // image operation modifiers
            OperationId::PreserveAspectRatio => Instr::EnvAdd(EnvItem::PreserveAspectRatio(
                parse_inputs_by_type!(inputs, bool)?,
            )),
            OperationId::SamplingFilter => {
                let input = parse_inputs_by_type!(inputs, String)?;
                let filter = FilterTypeWrap::try_from_str(&input)
                    .map_err(SicParserError::FilterTypeError)?;
                Instr::EnvAdd(EnvItem::CustomSamplingFilter(filter))
            }
        };

        Ok(stmt)
    }
}
