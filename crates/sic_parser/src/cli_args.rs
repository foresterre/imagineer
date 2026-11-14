use crate::ast::{Call, Literal};
use crate::builtins;
use crate::errors::{ArgumentError, ResolveError, SicParserError};
use crate::parser;
use crate::types::{Argument, Param, Type};
use sic_image_engine::engine::Instr;

struct RawArg<'s>(&'s str);

impl Argument for RawArg<'_> {
    fn bind<P: Param>(&self) -> Result<P, ArgumentError> {
        match P::TYPE {
            Type::String | Type::Path => P::from_literal(&Literal::String(self.0)),
            _ => P::from_expr(&parser::parse_expr(self.0)?),
        }
    }
}

pub fn operation(name: &str, args: &[&str]) -> Result<Instr, SicParserError> {
    let call = Call {
        name,
        args: args.iter().map(|arg| RawArg(arg)).collect(),
    };

    Ok(builtins::operation(&call).map(Instr::Operation)?)
}

pub fn set_option(option: &str, args: &[&str]) -> Result<Instr, SicParserError> {
    let &[arg] = args else {
        return Err(ResolveError::ArgumentCount {
            function: option.to_string(),
            expected: 1,
            found: args.len(),
        }
        .into());
    };

    Ok(builtins::set_option(option, &RawArg(arg))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sic_core::image::Rgba;
    use sic_core::image::imageops::FilterType;
    use sic_image_engine::ImgOp;
    use sic_image_engine::engine::EnvItem;
    use sic_image_engine::wrapper::draw_text_inner::DrawTextInner;
    use sic_image_engine::wrapper::filter_type::FilterTypeWrap;
    use sic_image_engine::wrapper::font_options::{FontOptions, FontScale};
    use sic_image_engine::wrapper::image_path::ImageFromPath;
    use std::path::PathBuf;

    #[test]
    fn numbers() {
        assert_eq!(
            operation("unsharpen", &["-1.0", "-1"]).unwrap(),
            Instr::Operation(ImgOp::Unsharpen((-1.0, -1)))
        );
        assert!(operation("brighten", &["-1.0"]).is_err());
        assert!(operation("blur", &["A"]).is_err());
        assert!(operation("contrast", &[""]).is_err());
    }

    #[test]
    fn text_is_taken_as_is() {
        assert_eq!(
            operation("diff", &["▲ b.png"]).unwrap(),
            Instr::Operation(ImgOp::Diff(ImageFromPath::new("▲ b.png".into())))
        );
    }

    #[test]
    fn value_calls() {
        assert_eq!(
            operation(
                "draw-text",
                &[
                    "my text",
                    "coord(0, 1)",
                    "rgba(10, 10, 255, 255)",
                    "size(16.0)",
                    r#"font("f.ttf")"#,
                ],
            )
            .unwrap(),
            Instr::Operation(ImgOp::DrawText(DrawTextInner::new(
                "my text".to_string(),
                (0, 1),
                FontOptions::new(
                    PathBuf::from("f.ttf"),
                    Rgba([10, 10, 255, 255]),
                    FontScale::Uniform(16.0),
                ),
            )))
        );
    }

    #[test]
    fn argument_count() {
        assert!(operation("resize", &["1"]).is_err());
        assert!(operation("resize", &["1", "1", "1"]).is_err());
        assert!(operation("grayscale", &["1"]).is_err());
    }

    #[test]
    fn options() {
        assert_eq!(
            set_option("sampling-filter", &["nearest"]).unwrap(),
            Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
                FilterType::Nearest
            )))
        );
        assert_eq!(
            set_option("preserve-aspect-ratio", &["true"]).unwrap(),
            Instr::EnvAdd(EnvItem::PreserveAspectRatio(true))
        );
        assert!(set_option("preserve-aspect-ratio", &["yes"]).is_err());
        assert!(set_option("sampling-filter", &[]).is_err());
        assert!(set_option("sampling-filter", &["nearest", "nearest"]).is_err());
    }
}
