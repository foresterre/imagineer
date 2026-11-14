#![deny(clippy::all)]

#[cfg(test)]
#[macro_use]
extern crate parameterized;

use crate::errors::SicParserError;
use sic_image_engine::engine::Instr;

pub mod ast;
mod builtins;
pub mod cli_args;
pub mod errors;
mod parser;
mod resolve;
pub mod types;

pub fn parse_script(script: &str) -> Result<Vec<Instr>, SicParserError> {
    let program = parser::parse_program(script)?;

    Ok(resolve::resolve(&program)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sic_core::image::Rgba;
    use sic_core::image::imageops::FilterType;
    use sic_image_engine::ImgOp;
    use sic_image_engine::engine::{EnvItem, ItemName};
    use sic_image_engine::wrapper::draw_text_inner::DrawTextInner;
    use sic_image_engine::wrapper::filter_type::FilterTypeWrap;
    use sic_image_engine::wrapper::font_options::{FontOptions, FontScale};
    use sic_image_engine::wrapper::gradient_input::GradientInput;
    use sic_image_engine::wrapper::image_path::ImageFromPath;
    use sic_image_engine::wrapper::overlay::OverlayInputs;
    use std::path::PathBuf;

    fn op(op: ImgOp) -> Instr {
        Instr::Operation(op)
    }

    fn filter(filter: FilterType) -> Instr {
        Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(filter)))
    }

    fn diff(path: &str) -> Vec<Instr> {
        vec![op(ImgOp::Diff(ImageFromPath::new(path.into())))]
    }

    fn overlay(path: &str, x: i64, y: i64) -> Vec<Instr> {
        vec![op(ImgOp::Overlay(OverlayInputs::new(
            ImageFromPath::new(path.into()),
            (x, y),
        )))]
    }

    fn blur_brighten() -> Vec<Instr> {
        vec![op(ImgOp::Blur(1.0)), op(ImgOp::Brighten(2))]
    }

    fn flips_resize_blur() -> Vec<Instr> {
        vec![
            op(ImgOp::FlipHorizontal),
            op(ImgOp::FlipVertical),
            op(ImgOp::Resize((100, 200))),
            op(ImgOp::Blur(10.0)),
        ]
    }

    fn gradient(from: [u8; 4], to: [u8; 4]) -> GradientInput {
        GradientInput::new((Rgba(from), Rgba(to)))
    }

    ide!();

    #[parameterized(
        input = {
            "",
            "blur 15; flip-vertical",
            "blur 1;\nbrighten 2",
            "blur 1;\nbrighten 2;",
            "blur 1;\nbrighten 2;    ",
            "blur 1; brighten 2",
            "blur 1;brighten 2",
            "blur 1; brighten 2;   ",
            "blur 1;brighten 2;",
            "BLUR 1; Brighten 2",
            "threshold",
            "threshold;flip-horizontal",
            "dither",
            "dither-quant 64 10",
            "dither-quant 64 10;flip-horizontal",
            "index-colors",
            "index-colors-quant 64 10",
            "index-colors-quant 64 10;flip-horizontal",
            "blur 15;",
            "blur 15.0;",
            "blur -15.0;",
            "contrast 15;",
            "contrast 15.8;",
            "brighten 3579;",
            "brighten -3579;",
            "crop 1 2 3 4;",
            "crop 1 1 1 1;",
            "crop 0 0 0 0;",
            "crop 0 0 0 4294967295",
            "crop 0 0 | 10 10",
            "flip-horizontal;",
            "flip-vertical;",
            "grayscale",
            "hue-rotate 3579;",
            "hue-rotate -3579;",
            "invert;",
            "resize 99 88;",
            "rotate90;",
            "rotate180;",
            "rotate270;",
            "unsharpen 99 88;",
            "unsharpen 99.0 88;",
            "unsharpen -99.0 -88;",
            "blur 10;flip-horizontal;flip-vertical;resize 100 200;",
            "flip-horizontal; flip-vertical; resize 100 200; blur 10;",
            "flip-horizontal    ; flip-vertical   ;      resize 100 200; blur 10;",
            "flip-horizontal;\nflip-vertical;\nresize 100 200;\nblur 10;",
            "flip-horizontal; flip-vertical; resize 100 200; blur 10",
            "flip-horizontal; flip-vertical;  resize 100 200;\nblur 10",
            "flip-horizontal;\r\n\tflip-vertical\n;resize 100 200 ;\tblur 10",
        },
        expected = {
            vec![],
            vec![op(ImgOp::Blur(15.0)), op(ImgOp::FlipVertical)],
            blur_brighten(),
            blur_brighten(),
            blur_brighten(),
            blur_brighten(),
            blur_brighten(),
            blur_brighten(),
            blur_brighten(),
            blur_brighten(),
            vec![op(ImgOp::Threshold)],
            vec![op(ImgOp::Threshold), op(ImgOp::FlipHorizontal)],
            vec![op(ImgOp::Dither)],
            vec![op(ImgOp::DitherQuant((64, 10)))],
            vec![op(ImgOp::DitherQuant((64, 10))), op(ImgOp::FlipHorizontal)],
            vec![op(ImgOp::IndexColors)],
            vec![op(ImgOp::IndexColorsQuant((64, 10)))],
            vec![op(ImgOp::IndexColorsQuant((64, 10))), op(ImgOp::FlipHorizontal)],
            vec![op(ImgOp::Blur(15.0))],
            vec![op(ImgOp::Blur(15.0))],
            vec![op(ImgOp::Blur(-15.0))],
            vec![op(ImgOp::Contrast(15.0))],
            vec![op(ImgOp::Contrast(15.8))],
            vec![op(ImgOp::Brighten(3579))],
            vec![op(ImgOp::Brighten(-3579))],
            vec![op(ImgOp::Crop((1, 2, 3, 4)))],
            vec![op(ImgOp::Crop((1, 1, 1, 1)))],
            vec![op(ImgOp::Crop((0, 0, 0, 0)))],
            vec![op(ImgOp::Crop((0, 0, 0, u32::MAX)))],
            vec![op(ImgOp::Crop((0, 0, 10, 10)))],
            vec![op(ImgOp::FlipHorizontal)],
            vec![op(ImgOp::FlipVertical)],
            vec![op(ImgOp::Grayscale)],
            vec![op(ImgOp::HueRotate(3579))],
            vec![op(ImgOp::HueRotate(-3579))],
            vec![op(ImgOp::Invert)],
            vec![op(ImgOp::Resize((99, 88)))],
            vec![op(ImgOp::Rotate90)],
            vec![op(ImgOp::Rotate180)],
            vec![op(ImgOp::Rotate270)],
            vec![op(ImgOp::Unsharpen((99.0, 88)))],
            vec![op(ImgOp::Unsharpen((99.0, 88)))],
            vec![op(ImgOp::Unsharpen((-99.0, -88)))],
            vec![op(ImgOp::Blur(10.0)), op(ImgOp::FlipHorizontal), op(ImgOp::FlipVertical), op(ImgOp::Resize((100, 200)))],
            flips_resize_blur(),
            flips_resize_blur(),
            flips_resize_blur(),
            flips_resize_blur(),
            flips_resize_blur(),
            flips_resize_blur(),
        }
    )]
    fn operations(input: &str, expected: Vec<Instr>) {
        assert_eq!(parse_script(input).unwrap(), expected);
    }

    #[parameterized(
        input = {
            "blur 15 28;",
            "blur 15.7.; flipv",
            "blur 4 blur 3",
            "blur 4\nblur 3",
            "blur 4 blur 3;",
            "blur1; brighten 2",
            "blur 15.;",
            "blur true",
            "blur \"1\"",
            "blur rgba(1, 2, 3, 4)",
            "blr 1",
            "contrast 15.;",
            "brighten 1.5",
            "crop -1 -1 -1 -1;",
            "crop -1 0 0 0;",
            "crop 0 -1 0 0;",
            "crop 0 0 -1 0;",
            "crop 0 0 0 -1;",
            "crop 0 0 0 4294967296",
            "340282200000000000000000000000000000000.0;",
            "flip_horizontal;",
            "flip_vertical;",
            "flip-horizontal 1",
            "unsharpen -99.0 -88.0;",
            "resize 100",
        }
    )]
    fn rejects(input: &str) {
        assert!(parse_script(input).is_err(), "{input:?} should be rejected");
    }

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
            "diff \"it's.jpg\";",
        },
        expected = {
            diff("/my/path/input.jpg"),
            diff("input.jpg"),
            diff("C:/Users/Some Name/input.jpg"),
            diff("C:\\Users\\Some Name\\input.jpg"),
            diff("/my/path/input.jpg"),
            diff("input.jpg"),
            diff("C:/Users/Some Name/input.jpg"),
            diff("C:\\Users\\Some Name\\input.jpg"),
            diff("it's.jpg"),
        }
    )]
    fn diff_ok(input: &str, expected: Vec<Instr>) {
        assert_eq!(parse_script(input).unwrap(), expected);
    }

    #[parameterized(
        input = {
            "diff \"/my/path/input.\"jpg\";",
            "diff '/my/path/input.jpg'';",
            "diff input.jpg",
            "diff input",
        }
    )]
    fn diff_err(input: &str) {
        assert!(parse_script(input).is_err(), "{input:?} should be rejected");
    }

    #[parameterized(
        input = {
            "filter3x3 0 0.1 0.2 | 1.3 1.4 1.5 | 2.6 2.7 2.8",
            "filter3x3 0 0 0 | 1 1 1 | 2 2 2",
            "filter3x3 0 0 0 1 1 1 2 2 3.0",
            "filter3x3 0 0 0 1 1 1 2 2 3.0;",
            "filter3x3 0 0 0 1 1 1 | 2 2 3.0",
            "filter3x3 0 0 0 | 1 1 1 2 2 3.0",
        },
        expected = {
            [0.0, 0.1, 0.2, 1.3, 1.4, 1.5, 2.6, 2.7, 2.8],
            [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0],
            [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0],
            [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0],
            [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0],
            [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0],
        }
    )]
    fn filter3x3_ok(input: &str, expected: [f32; 9]) {
        assert_eq!(
            parse_script(input).unwrap(),
            vec![op(ImgOp::Filter3x3(expected))]
        );
    }

    #[test]
    fn filter3x3_twice() {
        assert_eq!(
            parse_script(
                "filter3x3 1.9 2 3 | 4 5.9 6 | 7 8 9.9;\nfilter3x3 10.9 2 3 4 11.9 6 7 8 12.9"
            )
            .unwrap(),
            vec![
                op(ImgOp::Filter3x3([
                    1.9, 2.0, 3.0, 4.0, 5.9, 6.0, 7.0, 8.0, 9.9
                ])),
                op(ImgOp::Filter3x3([
                    10.9, 2.0, 3.0, 4.0, 11.9, 6.0, 7.0, 8.0, 12.9
                ])),
            ]
        );
    }

    #[parameterized(
        input = {
            "filter3x3 0 0 0 | 1 1 1 | 2 2 2 |",
            "filter3x3\n0 0 0\n1 1 1\n2 2 3.0;",
            "filter3x3 0 0 0\t1 1 1\t2 2 3;",
            "filter3x3\n\t0 0 0\n\t1 1 1\n\t2 2 3",
            "filter3x3 0 0.9 0 | 1 1.1 1|2.0 2 2",
            "filter3x3 0 0.9 0 | 1 1.1 1 |2.0 2 2",
            "filter3x3 0 0.9 0 | 1 1.1 1| 2.0 2 2",
            "filter3x3 0 0.9 0 | 1 1.1 1 | 2.0 2 2|",
            "filter3x3 0 0.9 0 | 1 1.1 1 | 2.0 2 2 | ",
            "filter3x3 | 0 0.9 0 | 1 1.1 1 | 2.0 2 2",
            "filter3x3 0 0.9 0 | | 1 1.1 1 | 2.0 2 2",
            "filter3x3 0 0.9 0 | 1 1.1 1 999 | 2.0 2 2",
            "filter3x3 0 0.9 0 | 1 2.2 3 | 2.0 2 2 | 0 1 2",
            "filter3x3 0 0.9 0 | 1 2.2 3",
        }
    )]
    fn filter3x3_err(input: &str) {
        assert!(parse_script(input).is_err(), "{input:?} should be rejected");
    }

    #[test]
    fn gradients() {
        assert_eq!(
            parse_script(
                "horizontal-gradient rgba(10, 10, 255, 255) rgba(255, 255, 255, 255);\
                 vertical-gradient RGBA(10,10,255,255) rgba( 255 , 255 , 255 , 255 )"
            )
            .unwrap(),
            vec![
                op(ImgOp::HorizontalGradient(gradient(
                    [10, 10, 255, 255],
                    [255, 255, 255, 255]
                ))),
                op(ImgOp::VerticalGradient(gradient(
                    [10, 10, 255, 255],
                    [255, 255, 255, 255]
                ))),
            ]
        );
    }

    #[parameterized(
        input = {
            "horizontal-gradient rgba(10, 10, 255, 255)",
            "vertical-gradient rgba(10, 10, 255, 255)",
            "vertical-gradient rgba(10, 10, 256, 255) rgba(0, 0, 0, 0)",
            "vertical-gradient rgba(10, 10, 255) rgba(0, 0, 0, 0)",
            "vertical-gradient rgb(10, 10, 255, 255) rgba(0, 0, 0, 0)",
            "vertical-gradient coord(0, 0) rgba(0, 0, 0, 0)",
        }
    )]
    fn gradient_err(input: &str) {
        assert!(parse_script(input).is_err(), "{input:?} should be rejected");
    }

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
        expected = {
            overlay("/my/path/input.jpg", 0, 0),
            overlay("/my/path/input.jpg", 10, 5),
            overlay("input.jpg", 0, 0),
            overlay("C:/Users/Some Name/input.jpg", 0, 0),
            overlay("C:\\Users\\Some Name\\input.jpg", 0, 0),
            overlay("/my/path/input.jpg", 0, 0),
            overlay("input.jpg", 0, 0),
            overlay("C:/Users/Some Name/input.jpg", 0, 0),
            overlay("C:\\Users\\Some Name\\input.jpg", 0, 0),
        }
    )]
    fn overlay_ok(input: &str, expected: Vec<Instr>) {
        assert_eq!(parse_script(input).unwrap(), expected);
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
    fn overlay_err(input: &str) {
        assert!(parse_script(input).is_err(), "{input:?} should be rejected");
    }

    #[parameterized(
        input = {
            "set sampling-filter CatmullRom;",
            "set sampling-filter GAUSSIAN;",
            "set sampling-filter Lanczos3;",
            "set sampling-filter nearest;",
            "set sampling-filter triangle;",
            "SET Sampling-Filter cubic",
        },
        expected = {
            filter(FilterType::CatmullRom),
            filter(FilterType::Gaussian),
            filter(FilterType::Lanczos3),
            filter(FilterType::Nearest),
            filter(FilterType::Triangle),
            filter(FilterType::CatmullRom),
        }
    )]
    fn set_sampling_filter(input: &str, expected: Instr) {
        assert_eq!(parse_script(input).unwrap(), vec![expected]);
    }

    #[test]
    fn set_sampling_filter_with_resize() {
        assert_eq!(
            parse_script("set   sampling-filter   GAUSSIAN;\nresize 100 200").unwrap(),
            vec![filter(FilterType::Gaussian), op(ImgOp::Resize((100, 200)))]
        );
    }

    #[test]
    fn set_and_del_sampling_filter() {
        assert_eq!(
            parse_script(
                "set sampling-filter catmullrom;\
                 set sampling-filter gaussian;\
                 del sampling-filter;\
                 del sampling-filter;"
            )
            .unwrap(),
            vec![
                filter(FilterType::CatmullRom),
                filter(FilterType::Gaussian),
                Instr::EnvRemove(ItemName::CustomSamplingFilter),
                Instr::EnvRemove(ItemName::CustomSamplingFilter),
            ]
        );
    }

    #[test]
    fn preserve_aspect_ratio() {
        assert_eq!(
            parse_script(
                "set preserve-aspect-ratio true; set preserve-aspect-ratio false;\n\
                 del preserve-aspect-ratio; resize 100 200"
            )
            .unwrap(),
            vec![
                Instr::EnvAdd(EnvItem::PreserveAspectRatio(true)),
                Instr::EnvAdd(EnvItem::PreserveAspectRatio(false)),
                Instr::EnvRemove(ItemName::PreserveAspectRatio),
                op(ImgOp::Resize((100, 200))),
            ]
        );
    }

    #[parameterized(
        input = {
            "set resize preserve-aspect-ratio true;\nresize 100 200",
            "set preserve-aspect-ratio yes",
            "set preserve-aspect-ratio",
            "set sampling-filter tri",
            "set sampling-filter 'nearest'",
            "set colors 1",
            "del colors",
            "del sampling-filter nearest",
        }
    )]
    fn options_err(input: &str) {
        assert!(parse_script(input).is_err(), "{input:?} should be rejected");
    }

    #[test]
    fn draw_text() {
        let expected = vec![op(ImgOp::DrawText(DrawTextInner::new(
            "my text".to_string(),
            (0, 1),
            FontOptions::new(
                PathBuf::from("resources/font/Lato-Regular.ttf"),
                Rgba([10, 10, 255, 255]),
                FontScale::Uniform(16.0),
            ),
        )))];

        assert_eq!(
            parse_script(
                r#"draw-text "my text" coord(0,1) rgba(10, 10, 255, 255) size(16.0) font("resources/font/Lato-Regular.ttf");"#
            )
            .unwrap(),
            expected
        );
    }

    #[parameterized(
        input = {
            r#"draw-text "my text" coord(0, 1) size(16.0) rgba(10, 10, 255, 255) font("f.ttf");"#,
            r#"draw-text "my text" coord(0, 1) rgba(10, 10, 255, 255) size(16.0)"#,
            r#"draw-text "my text" coord(0.5, 1) rgba(10, 10, 255, 255) size(16.0) font("f.ttf")"#,
            r#"draw-text my-text coord(0, 1) rgba(10, 10, 255, 255) size(16.0) font("f.ttf")"#,
            r#"draw-text "my text" coord(0, 1) rgba(10, 10, 255, 255) size(16.0) font(f)"#,
        }
    )]
    fn draw_text_err(input: &str) {
        assert!(parse_script(input).is_err(), "{input:?} should be rejected");
    }

    #[test]
    fn error_messages() {
        let message = |input: &str| {
            let err = parse_script(input).unwrap_err();
            let mut message = err.to_string();
            let mut source = std::error::Error::source(&err);
            while let Some(cause) = source {
                message.push_str(&format!(": {cause}"));
                source = cause.source();
            }
            message
        };

        assert_eq!(message("blr 1"), "unknown operation `blr`");
        assert_eq!(
            message("resize 10"),
            "wrong number of arguments for `resize`: expected 2, found 1"
        );
        assert_eq!(
            message("resize 10 -1"),
            "invalid argument `height` for `resize`: -1 doesn't fit in a whole number of 0 or more"
        );
        assert_eq!(
            message("vertical-gradient rgba(0, 0, 0, 0) rgba(0, 300, 0, 0)"),
            "invalid argument `to` for `vertical-gradient`: invalid argument `g` for `rgba`: 300 doesn't fit in a byte (0 to 255)"
        );
        assert_eq!(
            message(r#"draw-text "t" coord(0, 1) size(16) rgba(0, 0, 0, 0) font("f.ttf")"#),
            "invalid argument `color` for `draw-text`: expected a color, rgba(r, g, b, a), found size(16)"
        );
        assert_eq!(
            message("set preserve-aspect-ratio yes"),
            "invalid value for option `preserve-aspect-ratio`: expected true or false, found yes"
        );
    }
}
