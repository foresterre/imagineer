//! The rule parser module has a goal to parse pairs/span from Pest data structures to image operations.

use super::Rule;
use crate::errors::{OperationParamError, SicParserError};
use crate::value_parser::ParseInputsFromIter;
use pest::iterators::{Pair, Pairs};
use sic_image_engine::engine::{EnvItem, Instr, ItemName};

use sic_image_engine::ImgOp;
use sic_image_engine::wrapper::draw_text_inner::DrawTextInner;
use sic_image_engine::wrapper::filter_type::FilterTypeWrap;
use sic_image_engine::wrapper::image_path::ImageFromPath;
use sic_image_engine::wrapper::overlay::OverlayInputs;

// This function parses statements provided as a single 'script' to an image operations program.
// An image operations program is currently a linear list of image operations which are applied
// in a left-to-right order.
// Operations are parsed from Pairs and Rules, which are provided by the Pest parser library.
//
// In the event of any parse failure, an error shall be returned.
// The error currently usually contains into_inner(), to provide detailed information about the
// origins of the parsing rejection.
//
// FIXME: When the user facing errors will be reworked, the providing of or the how to providing of-
//        the into_inner() parsing details should be reconsidered
pub fn parse_image_operations(pairs: Pairs<'_, Rule>) -> Result<Vec<Instr>, SicParserError> {
    pairs
        .filter(|pair| pair.as_rule() != Rule::EOI)
        .map(|pair| match pair.as_rule() {
            Rule::blur => Blur(pair),
            Rule::brighten => Brighten(pair),
            Rule::contrast => Contrast(pair),
            Rule::crop => Crop(pair),
            Rule::diff => Diff(
                pair.into_inner()
                    .next()
                    .ok_or(SicParserError::NoInnerString)?,
            ),

            Rule::draw_text => Ok(parse_draw_text(pair)?),
            Rule::filter3x3 => Filter3x3(pair),
            Rule::flip_horizontal => Ok(Instr::Operation(ImgOp::FlipHorizontal)),
            Rule::flip_vertical => Ok(Instr::Operation(ImgOp::FlipVertical)),
            Rule::grayscale => Ok(Instr::Operation(ImgOp::Grayscale)),
            Rule::huerotate => HueRotate(pair),
            Rule::horizontal_gradient => Ok(parse_horizontal_gradient(pair)?),
            Rule::invert => Ok(Instr::Operation(ImgOp::Invert)),
            Rule::overlay => parse_overlay(pair),
            Rule::resize => Resize(pair),
            Rule::rotate90 => Ok(Instr::Operation(ImgOp::Rotate90)),
            Rule::rotate180 => Ok(Instr::Operation(ImgOp::Rotate180)),
            Rule::rotate270 => Ok(Instr::Operation(ImgOp::Rotate270)),
            Rule::unsharpen => Unsharpen(pair),
            Rule::setopt => {
                parse_set_environment(pair.into_inner().next().ok_or({
                    SicParserError::OperationError(OperationParamError::SetEnvironment)
                })?)
            }

            Rule::threshold => Ok(Instr::Operation(ImgOp::Threshold)),
            Rule::vertical_gradient => Ok(parse_vertical_gradient(pair)?),
            // this is called 'del' for users
            Rule::unsetopt => parse_unset_environment(pair.into_inner().next().ok_or({
                SicParserError::OperationError(OperationParamError::UnsetEnvironment)
            })?),
            _ => Err(SicParserError::UnknownOperationError),
        })
        .collect::<Result<Vec<_>, SicParserError>>()
}

macro_rules! parse_primitive_from_pair {
    ($pair:expr, $ty:ty) => {{
        let inner = $pair.into_inner();
        let ty: Result<$ty, SicParserError> =
            ParseInputsFromIter::parse(inner.map(|pair| pair.as_str()));
        ty
    }};
}

macro_rules! parse_op_from_pair {
    ($what_op:tt, $inner_ty:ty) => {
        #[allow(non_snake_case)]
        fn $what_op(pair: Pair<'_, Rule>) -> Result<Instr, SicParserError> {
            let val = parse_primitive_from_pair!(pair, $inner_ty)?;
            let stmt = Instr::Operation(ImgOp::$what_op(val));
            Ok(stmt)
        }
    };
}

parse_op_from_pair!(Blur, f32);
parse_op_from_pair!(Brighten, i32);
parse_op_from_pair!(Contrast, f32);
parse_op_from_pair!(Crop, (u32, u32, u32, u32));
parse_op_from_pair!(Diff, ImageFromPath);
parse_op_from_pair!(HueRotate, i32);
parse_op_from_pair!(Resize, (u32, u32));
parse_op_from_pair!(Unsharpen, (f32, i32));
parse_op_from_pair!(Filter3x3, [f32; 9]);

macro_rules! parse_setenv_from_pair {
    ($env_item:tt, $ty:ty) => {
        #[allow(non_snake_case)]
        fn $env_item(pair: Pair<'_, Rule>) -> Result<Instr, SicParserError> {
            let inner = pair.into_inner().skip(1);

            let arg: Result<$ty, SicParserError> =
                ParseInputsFromIter::parse(inner.map(|pair| pair.as_str()));

            let stmt = Instr::EnvAdd(EnvItem::$env_item(arg?));
            Ok(stmt)
        }
    };
}

parse_setenv_from_pair!(CustomSamplingFilter, FilterTypeWrap);
parse_setenv_from_pair!(PreserveAspectRatio, bool);

fn parse_set_environment(pair: Pair<'_, Rule>) -> Result<Instr, SicParserError> {
    let environment_item = match pair.as_rule() {
        Rule::set_resize_sampling_filter => CustomSamplingFilter(pair)?,
        Rule::set_resize_preserve_aspect_ratio => PreserveAspectRatio(pair)?,
        _ => {
            return Err(SicParserError::OperationError(
                OperationParamError::SetEnvironmentElement(format!("{}", pair)),
            ));
        }
    };

    Ok(environment_item)
}

fn parse_unset_environment(pair: Pair<'_, Rule>) -> Result<Instr, SicParserError> {
    let environment_item = match pair.as_rule() {
        Rule::env_resize_sampling_filter_name => ItemName::CustomSamplingFilter,
        Rule::env_resize_preserve_aspect_ratio_name => ItemName::PreserveAspectRatio,
        _ => {
            return Err(SicParserError::OperationError(
                OperationParamError::UnsetEnvironmentElement(format!("{}", pair)),
            ));
        }
    };

    Ok(Instr::EnvRemove(environment_item))
}

fn parse_overlay(pair: Pair<'_, Rule>) -> Result<Instr, SicParserError> {
    let mut pairs = pair.into_inner();

    let image_path = parse_primitive_from_pair!(
        pairs.next().ok_or(SicParserError::NoInnerString)?,
        ImageFromPath
    )?;

    let x = pairs
        .next()
        .ok_or_else(|| SicParserError::ExpectedValue("uint".to_string()))?;
    let y = pairs
        .next()
        .ok_or_else(|| SicParserError::ExpectedValue("uint".to_string()))?;

    let position: (i64, i64) = ParseInputsFromIter::parse(&[x.as_str(), y.as_str()])?;

    Ok(Instr::Operation(ImgOp::Overlay(OverlayInputs::new(
        image_path, position,
    ))))
}

// expected pair with inner pairs:
// - rule: 'string_unicode'; represents: text to draw
// - rule: 'named_value'; which: rgba(r, g, b, a) with r,g,b,a =: u8; represents: color of the text
// - rule: 'named_value'; which: size(s) with s =: u32; represents: size of the text
// - rule: 'named_value'; which: font(f) with f =: string (->into path); represents: which font file to use
fn parse_draw_text(pair: Pair<'_, Rule>) -> Result<Instr, SicParserError> {
    use crate::named_value::parse_named_value;
    use sic_core::image::Rgba;
    use sic_image_engine::wrapper::font_options::{FontOptions, FontScale};

    let mut pairs = pair.into_inner();

    // text
    let text_pair = pairs
        .next()
        .ok_or_else(|| SicParserError::ExpectedValue(String::from("String")))?
        .into_inner()
        .next()
        .ok_or_else(|| SicParserError::ExpectedValue(String::from("String")))?
        .as_str();

    let coord = pairs.next().ok_or_else(|| {
        SicParserError::ExpectedNamedValue(String::from("coord(x: NatNum, y: NatNum)"))
    })?;

    let coord = parse_named_value(coord).map_err(SicParserError::NamedValueParsingError)?;

    let color = pairs.next().ok_or_else(|| {
        SicParserError::ExpectedNamedValue(String::from("rgba(r: Byte, g: Byte, b: Byte, a: Byte)"))
    })?;

    let color = parse_named_value(color).map_err(SicParserError::NamedValueParsingError)?;

    let size = pairs
        .next()
        .ok_or_else(|| SicParserError::ExpectedNamedValue(String::from("size(v: Float)")))?;

    let size = parse_named_value(size).map_err(SicParserError::NamedValueParsingError)?;

    let font_file = pairs.next().ok_or_else(|| {
        SicParserError::ExpectedNamedValue(String::from("font(font_path: String)"))
    })?;

    let font_file = parse_named_value(font_file).map_err(SicParserError::NamedValueParsingError)?;

    Ok(Instr::Operation(ImgOp::DrawText(DrawTextInner::new(
        text_pair.to_string(),
        (coord.extract_coord()).map_err(SicParserError::NamedValueParsingError)?,
        FontOptions::new(
            font_file
                .extract_font()
                .map_err(SicParserError::NamedValueParsingError)?,
            Rgba(
                color
                    .extract_rgba()
                    .map_err(SicParserError::NamedValueParsingError)?,
            ),
            FontScale::Uniform(
                size.extract_size()
                    .map_err(SicParserError::NamedValueParsingError)?,
            ),
        ),
    ))))
}

macro_rules! def_parse_gradient {
    ($variant_name:ident, $variant_op:ident) => {
        fn $variant_name(pair: Pair<'_, Rule>) -> Result<Instr, SicParserError> {
            use crate::named_value::parse_named_value;
            use sic_core::image::Rgba;
            use sic_image_engine::wrapper::gradient_input::GradientInput;

            let mut pairs = pair.into_inner();

            let color = pairs.next().ok_or_else(|| {
                SicParserError::ExpectedNamedValue(String::from(
                    "rgba(r: Byte, g: Byte, b: Byte, a: Byte)",
                ))
            })?;
            let color = parse_named_value(color).map_err(SicParserError::NamedValueParsingError)?;

            let color2 = pairs.next().ok_or_else(|| {
                SicParserError::ExpectedNamedValue(String::from(
                    "rgba(r: Byte, g: Byte, b: Byte, a: Byte)",
                ))
            })?;
            let color2 =
                parse_named_value(color2).map_err(SicParserError::NamedValueParsingError)?;
            Ok(Instr::Operation(ImgOp::$variant_op(GradientInput::new((
                Rgba(
                    color
                        .extract_rgba()
                        .map_err(SicParserError::NamedValueParsingError)?,
                ),
                Rgba(
                    color2
                        .extract_rgba()
                        .map_err(SicParserError::NamedValueParsingError)?,
                ),
            )))))
        }
    };
}

def_parse_gradient!(parse_horizontal_gradient, HorizontalGradient);
def_parse_gradient!(parse_vertical_gradient, VerticalGradient);

#[cfg(test)]
mod tests {
    use crate::SICParser;
    use pest::Parser;
    use sic_core::image::imageops::FilterType;
    use sic_image_engine::engine::EnvItem;

    use super::*;

    #[test]
    fn test_parse_next_line_versions_fin_with_eoi() {
        let pairs = SICParser::parse(Rule::main, "blur 1;\nbrighten 2")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::Operation(ImgOp::Blur(1.0)),
                Instr::Operation(ImgOp::Brighten(2))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]

    fn test_parse_threshold_only() {
        let pairs =
            SICParser::parse(Rule::main, "threshold").unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![Instr::Operation(ImgOp::Threshold)],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]

    fn test_parse_threshold_with_flip() {
        let pairs = SICParser::parse(Rule::main, "threshold;flip-horizontal")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::Operation(ImgOp::Threshold),
                Instr::Operation(ImgOp::FlipHorizontal)
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_next_line_versions_fin_with_sep_eoi() {
        let pairs = SICParser::parse(Rule::main, "blur 1;\nbrighten 2;")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::Operation(ImgOp::Blur(1.0)),
                Instr::Operation(ImgOp::Brighten(2))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_next_line_versions_fin_with_sep_with_trailing_spaces_eoi() {
        let pairs = SICParser::parse(Rule::main, "blur 1;\nbrighten 2;    ")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::Operation(ImgOp::Blur(1.0)),
                Instr::Operation(ImgOp::Brighten(2))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_single_line_versions_fin_with_eoi() {
        let pairs = SICParser::parse(Rule::main, "blur 1; brighten 2")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::Operation(ImgOp::Blur(1.0)),
                Instr::Operation(ImgOp::Brighten(2))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_single_line_versions_fin_with_eoi_2() {
        let pairs = SICParser::parse(Rule::main, "blur 1;brighten 2")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::Operation(ImgOp::Blur(1.0)),
                Instr::Operation(ImgOp::Brighten(2))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_single_line_versions_fin_with_sep_with_trailing_spaces_eoi() {
        let pairs = SICParser::parse(Rule::main, "blur 1; brighten 2;   ")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::Operation(ImgOp::Blur(1.0)),
                Instr::Operation(ImgOp::Brighten(2))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_single_line_versions_require_sep() {
        SICParser::parse(Rule::main, "blur 4 blur 3").unwrap_or_else(|e| panic!("error: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_parse_single_line_versions_require_sep_2() {
        SICParser::parse(Rule::main, "blur 4\nblur 3").unwrap_or_else(|e| panic!("error: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_parse_single_line_versions_require_sep_3() {
        SICParser::parse(Rule::main, "blur 4 blur 3;").unwrap_or_else(|e| panic!("error: {:?}", e));
    }

    #[test]
    fn test_parse_single_line_versions_fin_with_sep_eoi() {
        let pairs = SICParser::parse(Rule::main, "blur 1;brighten 2;")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::Operation(ImgOp::Blur(1.0)),
                Instr::Operation(ImgOp::Brighten(2))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_require_space_between_operation_id_and_value() {
        SICParser::parse(Rule::main, "blur1; brighten 2")
            .unwrap_or_else(|e| panic!("error: {:?}", e));
    }

    #[test]
    fn test_blur_with_int_accept() {
        let pairs = SICParser::parse(Rule::main, "blur 15;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Blur(15.0))],
            parse_image_operations(pairs).unwrap()
        );
    }

    // related issue: https://github.com/foresterre/imagineer/issues/130
    #[test]
    fn test_blur_with_fp_accept() {
        let pairs = SICParser::parse(Rule::main, "blur 15.0;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Blur(15.0))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_blur_with_fp_neg_accept() {
        let pairs = SICParser::parse(Rule::main, "blur -15.0;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Blur(-15.0))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_blur_with_fp_reject() {
        SICParser::parse(Rule::main, "blur 15.;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    fn test_crop_in_order_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "crop 1 2 3 4;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Crop((1, 2, 3, 4)))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_crop_ones_parse_correct() {
        // Here we don't check that rX > lX and rY > lY
        // We only check that the values are uint and in range (u32)
        // lX = upper left X coordinate
        // lY = upper left Y coordinate
        // rX = bottom right X coordinate
        // rY = bottom right Y coordinate

        let pairs = SICParser::parse(Rule::main, "crop 1 1 1 1;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Crop((1, 1, 1, 1)))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_crop_zeros_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "crop 0 0 0 0;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Crop((0, 0, 0, 0)))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_crop_args_negative_parse_err() {
        SICParser::parse(Rule::main, "crop -1 -1 -1 -1;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_crop_arg_negative_p1_parse_err() {
        SICParser::parse(Rule::main, "crop -1 0 0 0;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_crop_arg_negative_p2_parse_err() {
        SICParser::parse(Rule::main, "crop 0 -1 0 0;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_crop_arg_negative_p3_parse_err() {
        SICParser::parse(Rule::main, "crop 0 0 -1 0;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_crop_arg_negative_p4_parse_err() {
        SICParser::parse(Rule::main, "crop 0 0 0 -1;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    fn test_crop_arg_to_big_p4_parse_err() {
        // 4294967296 == std::u32::MAX + 1
        let pairs = SICParser::parse(Rule::main, "crop 0 0 0 4294967296")
            .unwrap_or_else(|_| panic!("Unable to parse image operations script."));

        assert!(parse_image_operations(pairs).is_err())
    }

    #[test]
    fn test_crop_arg_just_in_range_p4_parse_ok() {
        // 4294967296 == std::u32::MAX
        let pairs = SICParser::parse(Rule::main, "crop 0 0 0 4294967295")
            .unwrap_or_else(|_| panic!("Unable to parse image operations script."));

        assert_eq!(
            vec![Instr::Operation(ImgOp::Crop((0, 0, 0, u32::MAX,)))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_contrast_single_stmt_int_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "contrast 15;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Contrast(15.0))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_contrast_single_stmt_f32_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "contrast 15.8;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Contrast(15.8))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_contrast_single_stmt_parse_fail_end_in_dot() {
        let pairs = SICParser::parse(Rule::main, "contrast 15.;");
        assert!(pairs.is_err());
    }

    #[cfg(test)]
    mod diff_test {
        use super::*;

        ide!();

        #[parameterized(
            input = {
                "diff \"/my/path/input.jpg\";",
                "diff \"input.jpg\";",
                "diff \"C:/Users/Some Name/input.jpg\";",
                "diff \"C:\\Users\\Some Name\\input.jpg\";",
                "diff '/my/path/input.jpg';",
                "diff 'input.jpg';",
                "diff 'C:/Users/Some Name/input.jpg';",
                "diff 'C:\\Users\\Some Name\\input.jpg';",
            },
            expected_ops = {
                vec![Instr::Operation(ImgOp::Diff(ImageFromPath::new("/my/path/input.jpg".into())))],
                vec![Instr::Operation(ImgOp::Diff(ImageFromPath::new("input.jpg".into())))],
                vec![Instr::Operation(ImgOp::Diff(ImageFromPath::new("C:/Users/Some Name/input.jpg".into())))],
                vec![Instr::Operation(ImgOp::Diff(ImageFromPath::new("C:\\Users\\Some Name\\input.jpg".into())))],
                vec![Instr::Operation(ImgOp::Diff(ImageFromPath::new("/my/path/input.jpg".into())))],
                vec![Instr::Operation(ImgOp::Diff(ImageFromPath::new("input.jpg".into())))],
                vec![Instr::Operation(ImgOp::Diff(ImageFromPath::new("C:/Users/Some Name/input.jpg".into())))],
                vec![Instr::Operation(ImgOp::Diff(ImageFromPath::new("C:\\Users\\Some Name\\input.jpg".into())))],
            }
        )]
        fn test_diff_ok(input: &str, expected_ops: Vec<Instr>) {
            let pairs = SICParser::parse(Rule::main, input)
                .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));

            assert_eq!(parse_image_operations(pairs).unwrap(), expected_ops);
        }

        #[parameterized(
            input = {
                "diff \"/my/path/input.\"jpg\";",
                "diff '/my/path/input.jpg'';",
            }
        )]
        fn test_diff_err(input: &str) {
            let pairs = SICParser::parse(Rule::main, input);
            assert!(pairs.is_err());
        }
    }

    #[test]
    fn test_contrast_single_stmt_parse_fail_max_f32_1() {
        let pairs = SICParser::parse(Rule::main, "340282200000000000000000000000000000000.0;");
        assert!(pairs.is_err());
    }

    #[test]
    fn test_brighten_pos_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "brighten 3579;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Brighten(3579))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_brighten_neg_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "brighten -3579;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Brighten(-3579))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_filter3x3_triplets_f3_with_end_triplet_sep_fail() {
        SICParser::parse(Rule::main, "filter3x3 0 0 0 | 1 1 1 | 2 2 2 |")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    fn test_filter3x3_triplets_f3_no_end_triplet_sep() {
        let pairs = SICParser::parse(
            Rule::main,
            "filter3x3 0 0.1 0.2 | 1.3 1.4 1.5 | 2.6 2.7 2.8",
        )
        .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Filter3x3([
                0.0, 0.1, 0.2, 1.3, 1.4, 1.5, 2.6, 2.7, 2.8
            ]))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_filter3x3_triplets_f3_ensure_f32() {
        let pairs = SICParser::parse(Rule::main, "filter3x3 0 0 0 | 1 1 1 | 2 2 2")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Filter3x3([
                0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0
            ]))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_filter3x3_triplets_f3_no_sep() {
        let pairs = SICParser::parse(Rule::main, "filter3x3 0 0 0 1 1 1 2 2 3.0")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Filter3x3([
                0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0
            ]))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_filter3x3_triplets_f3_end_op_sep() {
        let pairs = SICParser::parse(Rule::main, "filter3x3 0 0 0 1 1 1 2 2 3.0;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Filter3x3([
                0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0
            ]))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_filter3x3_triplets_f3_sep_newline() {
        let pairs = SICParser::parse(Rule::main, "filter3x3\n0 0 0\n1 1 1\n2 2 3.0;");

        assert!(pairs.is_err())
    }

    #[test]
    fn test_filter3x3_triplets_f3_tabbed_spacing() {
        let pairs = SICParser::parse(Rule::main, "filter3x3 0 0 0\t1 1 1\t2 2 3;");

        assert!(pairs.is_err())
    }

    #[test]
    fn test_filter3x3_triplets_f3_indented_newlines() {
        let pairs = SICParser::parse(Rule::main, "filter3x3\n\t0 0 0\n\t1 1 1\n\t2 2 3");

        assert!(pairs.is_err())
    }

    #[test]
    fn test_filter3x3_duo_filter3x3() {
        let pairs = SICParser::parse(
            Rule::main,
            "filter3x3 1.9 2 3 | 4 5.9 6 | 7 8 9.9;\nfilter3x3 10.9 2 3 4 11.9 6 7 8 12.9",
        )
        .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));

        assert_eq!(
            vec![
                Instr::Operation(ImgOp::Filter3x3([
                    1.9, 2.0, 3.0, 4.0, 5.9, 6.0, 7.0, 8.0, 9.9
                ])),
                Instr::Operation(ImgOp::Filter3x3([
                    10.9, 2.0, 3.0, 4.0, 11.9, 6.0, 7.0, 8.0, 12.9
                ])),
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_filter3x3_triplets_f3_require_spacing_on_triplet_sep_1() {
        SICParser::parse(Rule::main, "filter3x3 0 0.9 0 | 1 1.1 1|2.0 2 2")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_filter3x3_triplets_f3_require_spacing_on_triplet_sep_2() {
        SICParser::parse(Rule::main, "filter3x3 0 0.9 0 | 1 1.1 1 |2.0 2 2")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_filter3x3_triplets_f3_require_spacing_on_triplet_sep_3() {
        SICParser::parse(Rule::main, "filter3x3 0 0.9 0 | 1 1.1 1| 2.0 2 2")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_filter3x3_triplets_f3_require_spacing_on_triplet_sep_end_fail_1() {
        SICParser::parse(Rule::main, "filter3x3 0 0.9 0 | 1 1.1 1 | 2.0 2 2|")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_filter3x3_triplets_f3_require_spacing_on_triplet_sep_end_fail_2() {
        SICParser::parse(Rule::main, "filter3x3 0 0.9 0 | 1 1.1 1 | 2.0 2 2 | ")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_filter3x3_triplets_f3_require_all_triplet_sep_1() {
        SICParser::parse(Rule::main, "filter3x3 0 0.9 0 1 1.1 1 | 2.0 2 2 | ")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_filter3x3_triplets_f3_require_all_triplet_sep_2() {
        SICParser::parse(Rule::main, "filter3x3 0 0.9 0 | 1 1.1 1 2.0 2 2 | ")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_filter3x3_insufficient_args() {
        SICParser::parse(Rule::main, "filter3x3 0 0.9 0 | 1 1.1 1 999 | 2.0 2 2 | ")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_filter3x3_insufficient_triplet_count_4() {
        SICParser::parse(Rule::main, "filter3x3 0 0.9 0 | 1 2.2 3 | 2.0 2 2 | 0 1 2")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    #[should_panic]
    fn test_filter3x3_insufficient_triplet_count_2() {
        SICParser::parse(Rule::main, "filter3x3 0 0.9 0 | 1 2.2 3")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
    }

    #[test]
    fn test_flip_horizontal_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "flip-horizontal;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::FlipHorizontal)],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_flip_horizontal_removed() {
        let pairs = SICParser::parse(Rule::main, "flip_horizontal;");

        assert!(pairs.is_err());
    }

    #[test]
    fn test_flip_vertical_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "flip-vertical;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::FlipVertical)],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_flip_vertical_removed() {
        let pairs = SICParser::parse(Rule::main, "flip_vertical;");

        assert!(pairs.is_err());
    }

    #[test]
    fn test_hue_rotate_pos_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "hue-rotate 3579;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::HueRotate(3579))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_hue_rotate_neg_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "hue-rotate -3579;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::HueRotate(-3579))],
            parse_image_operations(pairs).unwrap()
        );
    }

    mod horizontal_gradient_tests {
        use super::*;
        use sic_core::image::Rgba;
        use sic_image_engine::wrapper::gradient_input::GradientInput;

        #[test]
        fn horizontal_gradient_valid_args() {
            let pairs = SICParser::parse(
                Rule::main,
                "horizontal-gradient rgba(10, 10, 255, 255) rgba(255, 255, 255, 255)",
            )
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));

            let expected = vec![Instr::Operation(ImgOp::HorizontalGradient(
                GradientInput::new((Rgba([10, 10, 255, 255]), Rgba([255, 255, 255, 255]))),
            ))];
            let actual = parse_image_operations(pairs).unwrap();

            assert_eq!(actual, expected);
        }

        #[test]
        #[should_panic]
        fn horizontal_gradient_invalid_args() {
            SICParser::parse(Rule::main, "horizontal-gradient rgba(10, 10, 255, 255)")
                .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        }
    }

    mod vertical_gradient_tests {
        use super::*;
        use sic_core::image::Rgba;
        use sic_image_engine::wrapper::gradient_input::GradientInput;

        #[test]
        fn vertical_gradient_valid_args() {
            let pairs = SICParser::parse(
                Rule::main,
                "vertical-gradient rgba(10, 10, 255, 255) rgba(255, 255, 255, 255)",
            )
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));

            let expected = vec![Instr::Operation(ImgOp::VerticalGradient(
                GradientInput::new((Rgba([10, 10, 255, 255]), Rgba([255, 255, 255, 255]))),
            ))];
            let actual = parse_image_operations(pairs).unwrap();

            assert_eq!(actual, expected);
        }

        #[test]
        #[should_panic]
        fn vertical_gradient_invalid_args() {
            SICParser::parse(Rule::main, "vertical-gradient rgba(10, 10, 255, 255)")
                .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        }
    }

    #[test]
    fn test_invert_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "invert;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Invert)],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[cfg(test)]
    mod overlay_test {
        use super::*;

        ide!();

        #[parameterized(
            input = {
                "overlay \"/my/path/input.jpg\" 0 0;",
                "overlay \"/my/path/input.jpg\" 10 5;",
                "overlay \"input.jpg\" 0 0;",
                "overlay \"C:/Users/Some Name/input.jpg\" 0 0;",
                "overlay \"C:\\Users\\Some Name\\input.jpg\" 0 0;",
                "overlay '/my/path/input.jpg' 0 0;",
                "overlay 'input.jpg' 0 0;",
                "overlay 'C:/Users/Some Name/input.jpg' 0 0;",
                "overlay 'C:\\Users\\Some Name\\input.jpg' 0 0;",
            },
            expected_ops = {
                vec![Instr::Operation(ImgOp::Overlay(OverlayInputs::new(ImageFromPath::new("/my/path/input.jpg".into()), (0, 0))))],
                vec![Instr::Operation(ImgOp::Overlay(OverlayInputs::new(ImageFromPath::new("/my/path/input.jpg".into()), (10, 5))))],
                vec![Instr::Operation(ImgOp::Overlay(OverlayInputs::new(ImageFromPath::new("input.jpg".into()), (0, 0))))],
                vec![Instr::Operation(ImgOp::Overlay(OverlayInputs::new(ImageFromPath::new("C:/Users/Some Name/input.jpg".into()), (0, 0))))],
                vec![Instr::Operation(ImgOp::Overlay(OverlayInputs::new(ImageFromPath::new("C:\\Users\\Some Name\\input.jpg".into()), (0, 0))))],
                vec![Instr::Operation(ImgOp::Overlay(OverlayInputs::new(ImageFromPath::new("/my/path/input.jpg".into()), (0, 0))))],
                vec![Instr::Operation(ImgOp::Overlay(OverlayInputs::new(ImageFromPath::new("input.jpg".into()), (0, 0))))],
                vec![Instr::Operation(ImgOp::Overlay(OverlayInputs::new(ImageFromPath::new("C:/Users/Some Name/input.jpg".into()), (0, 0))))],
                vec![Instr::Operation(ImgOp::Overlay(OverlayInputs::new(ImageFromPath::new("C:\\Users\\Some Name\\input.jpg".into()), (0, 0))))],
            }
        )]
        fn test_overlay_ok(input: &str, expected_ops: Vec<Instr>) {
            let pairs = SICParser::parse(Rule::main, input)
                .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));

            assert_eq!(parse_image_operations(pairs).unwrap(), expected_ops);
        }

        #[parameterized(
            input = {
                "overlay \"/my/path/input.jpg\"",
                "overlay '/my/path/input.jpg' (0, 0);",
                "overlay '/my/path/input.jpg' 0, 0;",
                "overlay '/my/path/input.jpg' -1 0;",
                "overlay '/my/path/input.jpg' 0 -1;",
            }
        )]
        fn test_overlay_err(input: &str) {
            let pairs = SICParser::parse(Rule::main, input);
            assert!(pairs.is_err());
        }
    }

    #[test]
    fn test_resize_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "resize 99 88;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Resize((99, 88)))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_rotate90_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "rotate90;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Rotate90)],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_rotate180_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "rotate180;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Rotate180)],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_rotate270_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "rotate270;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Rotate270)],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_unsharpen_single_stmt_parse_correct_ints() {
        let pairs = SICParser::parse(Rule::main, "unsharpen 99 88;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Unsharpen((99.0, 88)))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_unsharpen_single_stmt_parse_correct_fp_int() {
        let pairs = SICParser::parse(Rule::main, "unsharpen 99.0 88;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Unsharpen((99.0, 88)))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_unsharpen_single_stmt_parse_correct_fp_int_neg() {
        let pairs = SICParser::parse(Rule::main, "unsharpen -99.0 -88;")
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![Instr::Operation(ImgOp::Unsharpen((-99.0, -88)))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_unsharpen_single_stmt_parse_correct_fp_fp_fail() {
        let pairs = SICParser::parse(Rule::main, "unsharpen -99.0 -88.0;");
        assert!(pairs.is_err());
    }

    #[test]
    fn test_multi_stmt_parse_correct() {
        let pairs = SICParser::parse(
            Rule::main,
            "blur 10;flip-horizontal;flip-vertical;resize 100 200;",
        )
        .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![
                Instr::Operation(ImgOp::Blur(10.0)),
                Instr::Operation(ImgOp::FlipHorizontal),
                Instr::Operation(ImgOp::FlipVertical),
                Instr::Operation(ImgOp::Resize((100, 200)))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_multi_stmt_parse_diff_order_correct() {
        let pairs = SICParser::parse(
            Rule::main,
            "flip-horizontal;flip-vertical;resize 100 200;blur 10;",
        )
        .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![
                Instr::Operation(ImgOp::FlipHorizontal),
                Instr::Operation(ImgOp::FlipVertical),
                Instr::Operation(ImgOp::Resize((100, 200))),
                Instr::Operation(ImgOp::Blur(10.0))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_multi_whitespace() {
        let pairs = SICParser::parse(
            Rule::main,
            "flip-horizontal; flip-vertical; resize 100 200; blur 10;",
        )
        .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![
                Instr::Operation(ImgOp::FlipHorizontal),
                Instr::Operation(ImgOp::FlipVertical),
                Instr::Operation(ImgOp::Resize((100, 200))),
                Instr::Operation(ImgOp::Blur(10.0))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_multi_whitespace_2() {
        let pairs = SICParser::parse(
            Rule::main,
            "flip-horizontal    ; flip-vertical   ;      resize 100 200; blur 10;",
        )
        .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![
                Instr::Operation(ImgOp::FlipHorizontal),
                Instr::Operation(ImgOp::FlipVertical),
                Instr::Operation(ImgOp::Resize((100, 200))),
                Instr::Operation(ImgOp::Blur(10.0))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_multi_whitespace_3() {
        let pairs = SICParser::parse(
            Rule::main,
            "flip-horizontal;\nflip-vertical;\nresize 100 200;\nblur 10;",
        )
        .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![
                Instr::Operation(ImgOp::FlipHorizontal),
                Instr::Operation(ImgOp::FlipVertical),
                Instr::Operation(ImgOp::Resize((100, 200))),
                Instr::Operation(ImgOp::Blur(10.0))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_multi_should_no_longer_end_with_sep() {
        let pairs = SICParser::parse(
            Rule::main,
            "flip-horizontal; flip-vertical; resize 100 200; blur 10",
        )
        .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![
                Instr::Operation(ImgOp::FlipHorizontal),
                Instr::Operation(ImgOp::FlipVertical),
                Instr::Operation(ImgOp::Resize((100, 200))),
                Instr::Operation(ImgOp::Blur(10.0))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_multi_sep() {
        let pairs = SICParser::parse(
            Rule::main,
            "flip-horizontal; flip-vertical;  resize 100 200;\nblur 10",
        )
        .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));
        assert_eq!(
            vec![
                Instr::Operation(ImgOp::FlipHorizontal),
                Instr::Operation(ImgOp::FlipVertical),
                Instr::Operation(ImgOp::Resize((100, 200))),
                Instr::Operation(ImgOp::Blur(10.0))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_setopt_resize_sampling_filter_catmullrom() {
        let pairs = SICParser::parse(Rule::main, "set sampling-filter CatmullRom;")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![Instr::EnvAdd(EnvItem::CustomSamplingFilter(
                FilterTypeWrap::new(FilterType::CatmullRom)
            ))],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_setopt_resize_sampling_filter_gaussian() {
        let pairs = SICParser::parse(Rule::main, "set sampling-filter GAUSSIAN;")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![Instr::EnvAdd(EnvItem::CustomSamplingFilter(
                FilterTypeWrap::new(FilterType::Gaussian)
            )),],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_setopt_resize_sampling_filter_lanczos3() {
        let pairs = SICParser::parse(Rule::main, "set sampling-filter Lanczos3;")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![Instr::EnvAdd(EnvItem::CustomSamplingFilter(
                FilterTypeWrap::new(FilterType::Lanczos3)
            )),],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_setopt_resize_sampling_filter_nearest() {
        let pairs = SICParser::parse(Rule::main, "set sampling-filter nearest;")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![Instr::EnvAdd(EnvItem::CustomSamplingFilter(
                FilterTypeWrap::new(FilterType::Nearest)
            )),],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_setopt_resize_sampling_filter_triangle() {
        let pairs = SICParser::parse(Rule::main, "set sampling-filter triangle;")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![Instr::EnvAdd(EnvItem::CustomSamplingFilter(
                FilterTypeWrap::new(FilterType::Triangle)
            )),],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_setopt_resize_sampling_filter_with_resize() {
        let pairs = SICParser::parse(
            Rule::main,
            "set   sampling-filter   GAUSSIAN;\nresize 100 200",
        )
        .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
                    FilterType::Gaussian
                ))),
                Instr::Operation(ImgOp::Resize((100, 200)))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_setopt_resize_sampling_filter_multi() {
        let pairs = SICParser::parse(
            Rule::main,
            "set sampling-filter catmullrom;\
             set sampling-filter gaussian;\
             set sampling-filter lanczos3;\
             set sampling-filter nearest;\
             set sampling-filter triangle;",
        )
        .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
                    FilterType::CatmullRom
                ))),
                Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
                    FilterType::Gaussian
                ))),
                Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
                    FilterType::Lanczos3
                ))),
                Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
                    FilterType::Nearest
                ))),
                Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
                    FilterType::Triangle
                ))),
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_setopt_resize_preserve_aspect_ratio_t() {
        let pairs = SICParser::parse(
            Rule::main,
            "set preserve-aspect-ratio true;\nresize 100 200",
        )
        .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::EnvAdd(EnvItem::PreserveAspectRatio(true)),
                Instr::Operation(ImgOp::Resize((100, 200)))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_setopt_resize_preserve_aspect_ratio_f() {
        let pairs = SICParser::parse(
            Rule::main,
            "set preserve-aspect-ratio false;\nresize 100 200",
        )
        .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::EnvAdd(EnvItem::PreserveAspectRatio(false)),
                Instr::Operation(ImgOp::Resize((100, 200)))
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_setopt_resize_preserve_aspect_ratio_no_value() {
        SICParser::parse(
            Rule::main,
            "set resize preserve-aspect-ratio true;\nresize 100 200",
        )
        .unwrap_or_else(|e| panic!("error: {:?}", e));
    }

    #[test]
    fn test_parse_delopt_resize_sampling_filter_single() {
        let pairs = SICParser::parse(Rule::main, "del sampling-filter;")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![Instr::EnvRemove(ItemName::CustomSamplingFilter),],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_set_and_del_opt_resize_sampling_filter_multi() {
        let pairs = SICParser::parse(
            Rule::main,
            "set sampling-filter catmullrom;\
             set sampling-filter gaussian;\
             del sampling-filter;\
             del sampling-filter;",
        )
        .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![
                Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
                    FilterType::CatmullRom
                ))),
                Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
                    FilterType::Gaussian
                ))),
                Instr::EnvRemove(ItemName::CustomSamplingFilter),
                Instr::EnvRemove(ItemName::CustomSamplingFilter),
            ],
            parse_image_operations(pairs).unwrap()
        );
    }

    #[test]
    fn test_parse_delopt_resize_preserve_aspect_ratio_single() {
        let pairs = SICParser::parse(Rule::main, "del preserve-aspect-ratio;")
            .unwrap_or_else(|e| panic!("error: {:?}", e));

        assert_eq!(
            vec![Instr::EnvRemove(ItemName::PreserveAspectRatio),],
            parse_image_operations(pairs).unwrap()
        );
    }

    mod imageproc_ops_tests {
        use super::*;
        use sic_core::image::Rgba;
        use sic_image_engine::wrapper::font_options::{FontOptions, FontScale};
        use std::path::PathBuf;

        #[test]
        fn draw_text() {
            let pairs = SICParser::parse(
                Rule::main,
                r#"draw-text "my text" coord(0,1) rgba(10, 10, 255, 255) size(16.0) font("resources/font/Lato-Regular.ttf");"#,
            )
            .unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));

            let font_file = PathBuf::from("resources/font/Lato-Regular.ttf".to_string());
            let font_options = FontOptions::new(
                font_file,
                Rgba([255, 255, 0, 255]),
                FontScale::Uniform(16.0),
            );

            let expected = vec![Instr::Operation(ImgOp::DrawText(DrawTextInner::new(
                "my text".to_string(),
                (0, 1),
                font_options,
            )))];
            let actual = parse_image_operations(pairs).unwrap();

            assert_eq!(actual, expected);
        }

        #[test]
        fn draw_text_ordering() {
            let pairs = SICParser::parse(
                Rule::main,
                r#"draw-text "my text" coord(0, 1) size(16.0) rgba(10, 10, 255, 255) font("resources/font/Lato-Regular.ttf");"#,
            ).unwrap_or_else(|e| panic!("Unable to parse image operations script: {:?}", e));

            let actual = parse_image_operations(pairs);

            assert!(actual.is_err());
        }
    }
}
