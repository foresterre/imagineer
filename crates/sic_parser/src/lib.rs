#![deny(clippy::all)]
#![allow(clippy::unnecessary_wraps, clippy::upper_case_acronyms)]

#[cfg(test)]
#[macro_use]
extern crate parameterized;

use crate::errors::SicParserError;
use sic_image_engine::engine::Instr;

pub mod errors;
pub mod parser;

pub fn parse_script(script: &str) -> Result<Vec<Instr>, SicParserError> {
    parser::parse_script(script)
}

#[cfg(test)]
mod tests {
    use sic_image_engine::ImgOp;
    use sic_image_engine::engine::Instr;

    use super::*;

    #[test]
    fn test_too_many_args() {
        let input = "blur 15 28;";
        let parsed = parse_script(input);

        assert!(parsed.is_err());
    }

    #[test]
    fn test_parsed() {
        let input = "blur 15; flip-vertical";
        let parsed = parse_script(input);

        assert!(parsed.is_ok());

        assert_eq!(
            parsed.unwrap(),
            vec![
                Instr::Operation(ImgOp::Blur(15.0)),
                Instr::Operation(ImgOp::FlipVertical)
            ]
        );
    }

    #[test]
    fn test_parsed_fail() {
        let input = "blur 15.7.; flipv";
        let parsed = parse_script(input);

        assert!(parsed.is_err());
    }
}

#[cfg(test)]
mod corpus {
    use super::*;

    ide!();

    // Inputs the parser must accept. Kept from the original grammar's test suite.
    #[parameterized(input = {
        "blur 1;\nbrighten 2",
        "blur 15",
        "blur 15.0",
        "blur -15.0",
        "threshold",
        "threshold;flip-horizontal",
        "dither",
        "dither-quant 64 10",
        "dither-quant 64 10;flip-horizontal",
        "index-colors",
        "index-colors-quant 64 10",
        "index-colors-quant 64 10;flip-horizontal",
        "crop 1 2 3 4",
        "crop 0 0 0 4294967295",
        "contrast 15",
        "contrast 15.8",
        "brighten 3579",
        "brighten -3579",
        "filter3x3 0 0.1 0.2 | 1.3 1.4 1.5 | 2.6 2.7 2.8",
        "filter3x3 0 0 0 | 1 1 1 | 2 2 2",
        "filter3x3 0 0 0 1 1 1 2 2 3.0",
        "filter3x3 0 0 0 1 1 1 2 2 3.0;",
        "flip-horizontal",
        "flip-vertical",
        "grayscale",
        "hue-rotate 3579",
        "hue-rotate -3579",
        "invert",
        "resize 99 88",
        "rotate90",
        "rotate180",
        "rotate270",
        "unsharpen 99 88",
        "unsharpen 99.0 88",
        "unsharpen -99.0 -88",
        "diff \"input.jpg\"",
        "diff 'input.jpg'",
        "diff \"C:\\Users\\Some Name\\input.jpg\"",
        "overlay \"input.jpg\" 0 0",
        "overlay '/my/path/input.jpg' 10 5",
        "horizontal-gradient rgba(10, 10, 255, 255) rgba(255, 255, 255, 255)",
        "vertical-gradient rgba(10, 10, 255, 255) rgba(255, 255, 255, 255)",
        r#"draw-text "my text" coord(0,1) rgba(10, 10, 255, 255) size(16.0) font("f.ttf")"#,
        "set sampling-filter CatmullRom",
        "set sampling-filter GAUSSIAN;\nresize 100 200",
        "set preserve-aspect-ratio true;\nresize 100 200",
        "set preserve-aspect-ratio false;\nresize 100 200",
        "del sampling-filter",
        "del preserve-aspect-ratio",
        "blur 10;flip-horizontal;flip-vertical;resize 100 200;",
    })]
    fn accepts(input: &str) {
        assert!(parse_script(input).is_ok(), "should accept {input:?}");
    }

    // Inputs the parser must reject.
    #[parameterized(input = {
        "blur 15.",
        "blur 15 28;",
        "contrast 15.",
        "blur1; brighten 2",
        "blur 4 blur 3",
        "crop -1 -1 -1 -1",
        "crop 0 0 0 4294967296",
        "unsharpen -99.0 -88.0",
        "overlay '/my/path/input.jpg' (0, 0);",
        "overlay '/my/path/input.jpg' 0, 0;",
        "overlay '/my/path/input.jpg' -1 0;",
        "horizontal-gradient rgba(10, 10, 255, 255)",
        "vertical-gradient rgba(10, 10, 255, 255)",
        r#"draw-text "my text" coord(0, 1) size(16.0) rgba(10, 10, 255, 255) font("f.ttf")"#,
        "set resize preserve-aspect-ratio true",
        "filter3x3 0 0.9 0 | 1 1.1 1|2.0 2 2",
        "filter3x3 0 0.9 0 | 1 2.2 3",
    })]
    fn rejects(input: &str) {
        assert!(parse_script(input).is_err(), "should reject {input:?}");
    }
}
