#![deny(clippy::all)]

#[macro_use]
extern crate strum_macros;

#[cfg(test)]
#[macro_use]
extern crate parameterized;

use crate::errors::SicCliOpsError;
use crate::operations::OperationId;
use sic_image_engine::engine::Instr;
use strum::VariantNames;

pub mod errors;
pub mod operations;

pub type TResult<T> = Result<T, SicCliOpsError>;

/// Parses cli image operation definitions to image engine image operations.
/// This parser however doesn't replace Clap, and specifically its validator.
/// For example, `--flip-horizontal 0` will not be allowed
/// by Clap's validator. The function below however does allow it, since we parse
/// only the amount of arguments we expect to receive, in this case 0.
/// Since we can rely on Clap, we left the added complexity out here.  
pub fn create_image_ops<I: IntoIterator<Item = String>>(iter: I) -> TResult<Vec<Instr>> {
    let mut iter = iter.into_iter();

    let size = iter.size_hint().1.unwrap_or(128);

    let mut ast: Vec<Instr> = Vec::with_capacity(size);

    while let Some(ref program_argument) = iter.next() {
        if program_argument.starts_with("--")
            && OperationId::VARIANTS.contains(&&program_argument[2..])
        {
            let operation = OperationId::try_from_name(&program_argument[2..])?;
            let inputs = take_n(&mut iter, operation)?;
            let inputs = inputs.iter().map(|v| v.as_str()).collect::<Vec<&str>>();
            ast.push(operation.create_instruction(inputs)?);
        }
        // else: skip
    }

    Ok(ast)
}

fn take_n<I: Iterator<Item = String>>(
    iter: &mut I,
    operation: OperationId,
) -> TResult<Vec<String>> {
    let mut operation_arguments: Vec<String> = Vec::new();

    for i in 0..operation.takes_number_of_arguments() {
        if let Some(op_arg) = iter.next() {
            operation_arguments.push(op_arg)
        } else {
            return Err(SicCliOpsError::ExpectedArgumentForImageOperation(
                operation.as_str().to_string(),
                i,
            ));
        }
    }

    Ok(operation_arguments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::vec_init_then_push)]
    mod individual_args {
        use super::*;
        use sic_image_engine::engine::EnvItem;
        use sic_image_engine::wrapper::filter_type::FilterTypeWrap;
        use sic_image_engine::wrapper::image_path::ImageFromPath;
        use sic_image_engine::ImgOp;
        use sic_testing::setup_test_image;

        macro_rules! op {
            ($expr:expr) => {
                vec![Instr::Operation($expr)]
            };
        }

        macro_rules! ops {
            ($($expr:expr),*) => {{
                let mut vec = Vec::new();

                $(
                    vec.push(Instr::Operation($expr));
                )*

               vec
            }};
        }

        macro_rules! modifier {
            ($expr:expr) => {
                vec![Instr::EnvAdd($expr)]
            };
        }

        fn interweave(ops: &[&str]) -> Vec<String> {
            ops.iter()
                .map(|f| f.replace('▲', &setup_test_image("aaa.png").to_string_lossy()))
                .collect::<Vec<_>>()
        }

        ide!();

        #[parameterized(
            ops = {
                vec!["--blur", "1.0"],
                vec!["--brighten", "-1"],
                vec!["--contrast", "1.0"],
                vec!["--crop", "0", "1", "2", "3"],
                vec!["--diff", "▲"],
                vec!["--filter3x3", "1.0", "1.0", "1.0", "-1.0", "-1.0", "-1.0", "0.0", "0.0", "0.0"],
                vec!["--flip-horizontal"],
                vec!["--flip-vertical"],
                vec!["--grayscale"],
                vec!["--hue-rotate", "-1"],
                vec!["--invert"],
                vec!["--resize", "1", "1"],
                vec!["--preserve-aspect-ratio", "true"],
                vec!["--sampling-filter", "catmullrom"],
                vec!["--sampling-filter", "gaussian"],
                vec!["--sampling-filter", "lanczos3"],
                vec!["--sampling-filter", "nearest"],
                vec!["--sampling-filter", "triangle"],
                vec!["--rotate90"],
                vec!["--rotate180"],
                vec!["--rotate270"],
                vec!["--unsharpen", "-1.0", "-1"],
            },
            expected = {
                op![ImgOp::Blur(1.0)],
                op![ImgOp::Brighten(-1)],
                op![ImgOp::Contrast(1.0)],
                op![ImgOp::Crop((0, 1, 2, 3))],
                op![ImgOp::Diff(ImageFromPath::new(setup_test_image("aaa.png")))],
                op![ImgOp::Filter3x3([1.0, 1.0, 1.0, -1.0, -1.0, -1.0, 0.0, 0.0, 0.0])],
                op![ImgOp::FlipHorizontal],
                op![ImgOp::FlipVertical],
                op![ImgOp::Grayscale],
                op![ImgOp::HueRotate(-1)],
                op![ImgOp::Invert],
                op![ImgOp::Resize((1, 1))],
                modifier![EnvItem::PreserveAspectRatio(true)],
                modifier![EnvItem::CustomSamplingFilter(FilterTypeWrap::try_from_str("catmullrom").unwrap())],
                modifier![EnvItem::CustomSamplingFilter(FilterTypeWrap::try_from_str("gaussian").unwrap())],
                modifier![EnvItem::CustomSamplingFilter(FilterTypeWrap::try_from_str("lanczos3").unwrap())],
                modifier![EnvItem::CustomSamplingFilter(FilterTypeWrap::try_from_str("nearest").unwrap())],
                modifier![EnvItem::CustomSamplingFilter(FilterTypeWrap::try_from_str("triangle").unwrap())],
                op![ImgOp::Rotate90],
                op![ImgOp::Rotate180],
                op![ImgOp::Rotate270],
                op![ImgOp::Unsharpen((-1.0, -1))],
            },
        )]
        fn create_image_ops_t_sunny(ops: Vec<&str>, expected: Vec<Instr>) {
            let result = create_image_ops(interweave(&ops));
            assert_eq!(result.unwrap(), expected);
        }

        mod imageproc_ops_tests {
            use super::*;
            use sic_core::image::Rgba;
            use sic_image_engine::wrapper::draw_text_inner::DrawTextInner;
            use sic_image_engine::wrapper::font_options::{FontOptions, FontScale};
            use std::path::PathBuf;

            ide!();

            #[parameterized(
                ops = {
                    vec!["--draw-text", "my text", "coord(0, 1)", "rgba(10, 10, 255, 255)", "size(16.0)", r#"font("resources/font/Lato-Regular.ttf")"#],
                    vec!["--draw-text", "my text", "coord(0, 1)", "rgba(10, 10, 255, 255)", "size(16.0)", r#"font("resources/font/Lato-Regular()".ttf")"#],
                },
                expected = {
                    op![ImgOp::DrawText(DrawTextInner::new("my text".to_string(),
                        (0, 1),
                        FontOptions::new(
                        PathBuf::from("resources/font/Lato-Regular.ttf".to_string()),
                        Rgba([10, 10, 255, 255]),
                        FontScale::Uniform(16.0))))],
                    op![ImgOp::DrawText(DrawTextInner::new("my text".to_string(),
                        (0, 1),
                        FontOptions::new(
                        PathBuf::from("resources/font/Lato-Regular()\".ttf".to_string()),
                        Rgba([10, 10, 255, 255]),
                        FontScale::Uniform(16.0))))]
                }
            )]
            fn create_image_ops_t_sunny_imageproc_ops(ops: Vec<&str>, expected: Vec<Instr>) {
                let result = create_image_ops(interweave(&ops));

                assert_eq!(result.unwrap(), expected);
            }
        }

        #[test]
        fn combined() {
            let input = vec![
                "--blur",
                "1.0",
                "--brighten",
                "-1",
                "--contrast",
                "1.0",
                "--crop",
                "0",
                "1",
                "2",
                "3",
                "--diff",
                &setup_test_image("aaa.png").to_string_lossy(),
                "--filter3x3",
                "1.0",
                "1.0",
                "1.0",
                "-1.0",
                "-1.0",
                "-1.0",
                "0.0",
                "0.0",
                "0.0",
                "--flip-horizontal",
            ]
            .iter()
            .map(|v| (*v).to_string())
            .collect::<Vec<_>>();

            let expected = ops![
                ImgOp::Blur(1.0),
                ImgOp::Brighten(-1),
                ImgOp::Contrast(1.0),
                ImgOp::Crop((0, 1, 2, 3)),
                ImgOp::Diff(ImageFromPath::new(setup_test_image("aaa.png"))),
                ImgOp::Filter3x3([1.0, 1.0, 1.0, -1.0, -1.0, -1.0, 0.0, 0.0, 0.0]),
                ImgOp::FlipHorizontal
            ];

            assert_eq!(create_image_ops(input).unwrap(), expected);
        }

        #[parameterized(
            ops = {
                vec!["--blur", "A"],
                vec!["--brighten", "-1.0"],
                vec!["--contrast", ""],
                vec!["--crop", "--crop", "0", "1", "2", "3"],
                vec!["--diff"],
                vec!["--filter3x3", "[", "1.0", "1.0", "1.0", "-1.0", "-1.0", "-1.0", "0.0", "0.0", "0.0", "]"],
                vec!["--hue-rotate", "-100.8"],
                vec!["--resize", "1", "1", "--crop"],
                vec!["--preserve-aspect-ratio", "yes"],
                vec!["--sampling-filter", "tri"],
                vec!["--sampling-filter", ""],
                vec!["--unsharpen", "-1.0", "-1.0"],
            }
        )]
        fn create_image_ops_t_expected_failure(ops: Vec<&str>) {
            let result = create_image_ops(interweave(&ops));
            assert!(result.is_err());
        }
    }
}
