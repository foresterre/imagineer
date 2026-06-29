//! The unified parser: a generic application syntax (an operation name followed by typed value
//! arguments), resolved against a prelude registry which holds the operations as data.

use crate::errors::SicParserError;
use sic_core::image::Rgba;
use sic_image_engine::ImgOp;
use sic_image_engine::engine::{EnvItem, Instr, ItemName};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use winnow::ascii::{digit1, space0, space1};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated};
use winnow::error::{ContextError, ErrMode};
use winnow::token::take_while;
use winnow::{ModalResult, Parser};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Ident(String),
    Named { name: String, args: Vec<Value> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub name: String,
    pub args: Vec<Value>,
}

impl Value {
    pub fn as_f32(&self) -> Result<f32, SicParserError> {
        match self {
            Value::Int(n) => Ok(*n as f32),
            Value::Float(f) => Ok(*f as f32),
            other => Err(type_err("a number", other)),
        }
    }

    pub fn as_i32(&self) -> Result<i32, SicParserError> {
        match self {
            Value::Int(n) => i32::try_from(*n).map_err(|_| type_err("an i32", self)),
            other => Err(type_err("an i32", other)),
        }
    }

    pub fn as_u32(&self) -> Result<u32, SicParserError> {
        match self {
            Value::Int(n) => u32::try_from(*n).map_err(|_| type_err("a u32", self)),
            other => Err(type_err("a u32", other)),
        }
    }

    pub fn as_bool(&self) -> Result<bool, SicParserError> {
        match self {
            Value::Bool(b) => Ok(*b),
            other => Err(type_err("a bool", other)),
        }
    }

    pub fn as_str(&self) -> Result<&str, SicParserError> {
        match self {
            Value::Str(s) => Ok(s),
            other => Err(type_err("a string", other)),
        }
    }

    pub fn as_path(&self) -> Result<PathBuf, SicParserError> {
        Ok(PathBuf::from(self.as_str()?))
    }

    pub fn as_ident(&self) -> Result<&str, SicParserError> {
        match self {
            Value::Ident(s) => Ok(s),
            other => Err(type_err("an identifier", other)),
        }
    }

    pub fn as_rgba(&self) -> Result<Rgba<u8>, SicParserError> {
        match self {
            Value::Named { name, args } if name == "rgba" => match args.as_slice() {
                [r, g, b, a] => Ok(Rgba([as_byte(r)?, as_byte(g)?, as_byte(b)?, as_byte(a)?])),
                _ => Err(type_err("rgba(r, g, b, a)", self)),
            },
            other => Err(type_err("rgba(r, g, b, a)", other)),
        }
    }

    pub fn as_coord(&self) -> Result<(i32, i32), SicParserError> {
        match self {
            Value::Named { name, args } if name == "coord" => match args.as_slice() {
                [x, y] => Ok((x.as_i32()?, y.as_i32()?)),
                _ => Err(type_err("coord(x, y)", self)),
            },
            other => Err(type_err("coord(x, y)", other)),
        }
    }

    pub fn as_size(&self) -> Result<f32, SicParserError> {
        match self {
            Value::Named { name, args } if name == "size" => match args.as_slice() {
                [s] => s.as_f32(),
                _ => Err(type_err("size(s)", self)),
            },
            other => Err(type_err("size(s)", other)),
        }
    }

    pub fn as_font(&self) -> Result<PathBuf, SicParserError> {
        match self {
            Value::Named { name, args } if name == "font" => match args.as_slice() {
                [p] => p.as_path(),
                _ => Err(type_err("font(path)", self)),
            },
            other => Err(type_err("font(path)", other)),
        }
    }
}

fn as_byte(v: &Value) -> Result<u8, SicParserError> {
    match v {
        Value::Int(n) => u8::try_from(*n).map_err(|_| type_err("a byte", v)),
        other => Err(type_err("a byte", other)),
    }
}

fn type_err(expected: &str, got: &Value) -> SicParserError {
    SicParserError::ValueParsingError(format!("expected {expected}, found {got:?}"))
}

fn backtrack() -> ErrMode<ContextError> {
    ErrMode::Backtrack(ContextError::new())
}

fn number(input: &mut &str) -> ModalResult<Value> {
    let checkpoint = *input;
    let token = (opt('-'), digit1, opt(('.', digit1)))
        .take()
        .parse_next(input)?;

    let parsed = if token.contains('.') {
        token.parse::<f64>().map(Value::Float).ok()
    } else {
        token.parse::<i64>().map(Value::Int).ok()
    };

    parsed.ok_or_else(|| {
        *input = checkpoint;
        backtrack()
    })
}

fn boolean(input: &mut &str) -> ModalResult<Value> {
    alt((
        "true".value(Value::Bool(true)),
        "false".value(Value::Bool(false)),
    ))
    .parse_next(input)
}

fn string_inner<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    alt((
        delimited('"', take_while(0.., |c| c != '"'), '"'),
        delimited('\'', take_while(0.., |c| c != '\''), '\''),
    ))
    .parse_next(input)
}

fn kebab_ident<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '-').parse_next(input)
}

fn named_value(input: &mut &str) -> ModalResult<Value> {
    (
        kebab_ident,
        delimited(
            (space0, '(', space0),
            separated(0.., value, (space0, ',', space0)),
            (space0, ')'),
        ),
    )
        .map(|(name, args): (&str, Vec<Value>)| Value::Named {
            name: name.to_string(),
            args,
        })
        .parse_next(input)
}

fn value(input: &mut &str) -> ModalResult<Value> {
    alt((
        boolean,
        number,
        string_inner.map(|s| Value::Str(s.to_string())),
        named_value,
        kebab_ident.map(|s| Value::Ident(s.to_string())),
    ))
    .parse_next(input)
}

// A value, or the filter3x3 `|` triplet separator, which we swallow and discard.
fn arg(input: &mut &str) -> ModalResult<Option<Value>> {
    alt(('|'.value(None), value.map(Some))).parse_next(input)
}

fn statement(input: &mut &str) -> ModalResult<Call> {
    let name = kebab_ident.parse_next(input)?;
    let args: Vec<Option<Value>> = repeat(0.., preceded(space1, arg)).parse_next(input)?;
    Ok(Call {
        name: name.to_string(),
        args: args.into_iter().flatten().collect(),
    })
}

fn ws(input: &mut &str) -> ModalResult<()> {
    take_while(0.., |c: char| matches!(c, ' ' | '\n' | '\r' | '\t'))
        .void()
        .parse_next(input)
}

pub fn parse_calls(input: &str) -> Result<Vec<Call>, SicParserError> {
    let mut rest = input;
    let parsed: ModalResult<Vec<Call>> = delimited(
        ws,
        separated(0.., statement, (ws, ';', ws)),
        (opt((ws, ';')), ws),
    )
    .parse_next(&mut rest);

    match parsed {
        Ok(calls) if rest.is_empty() => Ok(calls),
        Ok(_) => Err(SicParserError::ValueParsingError(format!(
            "unexpected trailing input: {rest}"
        ))),
        Err(err) => Err(SicParserError::ParseError(err.to_string())),
    }
}

pub fn parse_script(input: &str) -> Result<Vec<Instr>, SicParserError> {
    resolve(parse_calls(input)?)
}

#[derive(Clone, Copy)]
enum Arity {
    Exact(usize),
}

impl Arity {
    fn check(self, got: usize, name: &str) -> Result<(), SicParserError> {
        match self {
            Arity::Exact(want) if want == got => Ok(()),
            Arity::Exact(want) => Err(SicParserError::ValueParsingError(format!(
                "operation `{name}` expects {want} argument(s), found {got}"
            ))),
        }
    }
}

struct OpDef {
    arity: Arity,
    build: fn(&[Value]) -> Result<Instr, SicParserError>,
}

fn op(arity: Arity, build: fn(&[Value]) -> Result<Instr, SicParserError>) -> OpDef {
    OpDef { arity, build }
}

fn prelude() -> &'static HashMap<&'static str, OpDef> {
    static PRELUDE: OnceLock<HashMap<&'static str, OpDef>> = OnceLock::new();
    PRELUDE.get_or_init(|| {
        use sic_image_engine::wrapper::draw_text_inner::DrawTextInner;
        use sic_image_engine::wrapper::font_options::{FontOptions, FontScale};
        use sic_image_engine::wrapper::gradient_input::GradientInput;
        use sic_image_engine::wrapper::image_path::ImageFromPath;
        use sic_image_engine::wrapper::overlay::OverlayInputs;

        let mut m: HashMap<&'static str, OpDef> = HashMap::new();

        m.insert(
            "blur",
            op(Arity::Exact(1), |a| {
                Ok(Instr::Operation(ImgOp::Blur(a[0].as_f32()?)))
            }),
        );
        m.insert(
            "brighten",
            op(Arity::Exact(1), |a| {
                Ok(Instr::Operation(ImgOp::Brighten(a[0].as_i32()?)))
            }),
        );
        m.insert(
            "contrast",
            op(Arity::Exact(1), |a| {
                Ok(Instr::Operation(ImgOp::Contrast(a[0].as_f32()?)))
            }),
        );
        m.insert(
            "crop",
            op(Arity::Exact(4), |a| {
                Ok(Instr::Operation(ImgOp::Crop((
                    a[0].as_u32()?,
                    a[1].as_u32()?,
                    a[2].as_u32()?,
                    a[3].as_u32()?,
                ))))
            }),
        );
        m.insert(
            "diff",
            op(Arity::Exact(1), |a| {
                Ok(Instr::Operation(ImgOp::Diff(ImageFromPath::new(
                    a[0].as_path()?,
                ))))
            }),
        );
        m.insert(
            "dither",
            op(Arity::Exact(0), |_| Ok(Instr::Operation(ImgOp::Dither))),
        );
        m.insert(
            "dither-quant",
            op(Arity::Exact(2), |a| {
                Ok(Instr::Operation(ImgOp::DitherQuant((
                    a[0].as_u32()?,
                    a[1].as_u32()?,
                ))))
            }),
        );
        m.insert(
            "draw-text",
            op(Arity::Exact(5), |a| {
                Ok(Instr::Operation(ImgOp::DrawText(DrawTextInner::new(
                    a[0].as_str()?.to_string(),
                    a[1].as_coord()?,
                    FontOptions::new(
                        a[4].as_font()?,
                        a[2].as_rgba()?,
                        FontScale::Uniform(a[3].as_size()?),
                    ),
                ))))
            }),
        );
        m.insert(
            "filter3x3",
            op(Arity::Exact(9), |a| {
                let mut kernel = [0f32; 9];
                for (slot, v) in kernel.iter_mut().zip(a) {
                    *slot = v.as_f32()?;
                }
                Ok(Instr::Operation(ImgOp::Filter3x3(kernel)))
            }),
        );
        m.insert(
            "flip-horizontal",
            op(Arity::Exact(0), |_| {
                Ok(Instr::Operation(ImgOp::FlipHorizontal))
            }),
        );
        m.insert(
            "flip-vertical",
            op(Arity::Exact(0), |_| {
                Ok(Instr::Operation(ImgOp::FlipVertical))
            }),
        );
        m.insert(
            "grayscale",
            op(Arity::Exact(0), |_| Ok(Instr::Operation(ImgOp::Grayscale))),
        );
        m.insert(
            "hue-rotate",
            op(Arity::Exact(1), |a| {
                Ok(Instr::Operation(ImgOp::HueRotate(a[0].as_i32()?)))
            }),
        );
        m.insert(
            "horizontal-gradient",
            op(Arity::Exact(2), |a| {
                Ok(Instr::Operation(ImgOp::HorizontalGradient(
                    GradientInput::new((a[0].as_rgba()?, a[1].as_rgba()?)),
                )))
            }),
        );
        m.insert(
            "index-colors",
            op(Arity::Exact(0), |_| {
                Ok(Instr::Operation(ImgOp::IndexColors))
            }),
        );
        m.insert(
            "index-colors-quant",
            op(Arity::Exact(2), |a| {
                Ok(Instr::Operation(ImgOp::IndexColorsQuant((
                    a[0].as_u32()?,
                    a[1].as_u32()?,
                ))))
            }),
        );
        m.insert(
            "invert",
            op(Arity::Exact(0), |_| Ok(Instr::Operation(ImgOp::Invert))),
        );
        m.insert(
            "overlay",
            op(Arity::Exact(3), |a| {
                Ok(Instr::Operation(ImgOp::Overlay(OverlayInputs::new(
                    ImageFromPath::new(a[0].as_path()?),
                    (a[1].as_u32()? as i64, a[2].as_u32()? as i64),
                ))))
            }),
        );
        m.insert(
            "resize",
            op(Arity::Exact(2), |a| {
                Ok(Instr::Operation(ImgOp::Resize((
                    a[0].as_u32()?,
                    a[1].as_u32()?,
                ))))
            }),
        );
        m.insert(
            "rotate90",
            op(Arity::Exact(0), |_| Ok(Instr::Operation(ImgOp::Rotate90))),
        );
        m.insert(
            "rotate180",
            op(Arity::Exact(0), |_| Ok(Instr::Operation(ImgOp::Rotate180))),
        );
        m.insert(
            "rotate270",
            op(Arity::Exact(0), |_| Ok(Instr::Operation(ImgOp::Rotate270))),
        );
        m.insert(
            "threshold",
            op(Arity::Exact(0), |_| Ok(Instr::Operation(ImgOp::Threshold))),
        );
        m.insert(
            "unsharpen",
            op(Arity::Exact(2), |a| {
                Ok(Instr::Operation(ImgOp::Unsharpen((
                    a[0].as_f32()?,
                    a[1].as_i32()?,
                ))))
            }),
        );
        m.insert(
            "vertical-gradient",
            op(Arity::Exact(2), |a| {
                Ok(Instr::Operation(ImgOp::VerticalGradient(
                    GradientInput::new((a[0].as_rgba()?, a[1].as_rgba()?)),
                )))
            }),
        );
        m.insert("set", op(Arity::Exact(2), build_set));
        m.insert("del", op(Arity::Exact(1), build_del));

        m
    })
}

fn build_set(args: &[Value]) -> Result<Instr, SicParserError> {
    use sic_image_engine::wrapper::filter_type::FilterTypeWrap;
    match args[0].as_ident()?.to_lowercase().as_str() {
        "sampling-filter" => {
            let filter = FilterTypeWrap::try_from_str(args[1].as_ident()?)
                .map_err(SicParserError::FilterTypeError)?;
            Ok(Instr::EnvAdd(EnvItem::CustomSamplingFilter(filter)))
        }
        "preserve-aspect-ratio" => Ok(Instr::EnvAdd(EnvItem::PreserveAspectRatio(
            args[1].as_bool()?,
        ))),
        other => Err(SicParserError::ValueParsingError(format!(
            "unknown environment option `{other}`"
        ))),
    }
}

fn build_del(args: &[Value]) -> Result<Instr, SicParserError> {
    match args[0].as_ident()?.to_lowercase().as_str() {
        "sampling-filter" => Ok(Instr::EnvRemove(ItemName::CustomSamplingFilter)),
        "preserve-aspect-ratio" => Ok(Instr::EnvRemove(ItemName::PreserveAspectRatio)),
        other => Err(SicParserError::ValueParsingError(format!(
            "unknown environment option `{other}`"
        ))),
    }
}

fn resolve(calls: Vec<Call>) -> Result<Vec<Instr>, SicParserError> {
    let registry = prelude();
    calls
        .into_iter()
        .map(|call| {
            let name = call.name.to_lowercase();
            let def = registry
                .get(name.as_str())
                .ok_or(SicParserError::UnknownOperationError)?;
            def.arity.check(call.args.len(), &name)?;

            (def.build)(&call.args)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sic_core::image::imageops::FilterType;
    use sic_image_engine::wrapper::filter_type::FilterTypeWrap;
    use sic_image_engine::wrapper::gradient_input::GradientInput;
    use sic_image_engine::wrapper::image_path::ImageFromPath;
    use sic_image_engine::wrapper::overlay::OverlayInputs;

    ide!();

    fn op(o: ImgOp) -> Vec<Instr> {
        vec![Instr::Operation(o)]
    }

    #[parameterized(input = {
        "blur 15",
        "blur -15.0",
        "brighten -3579",
        "contrast 15.8",
        "crop 1 2 3 4",
        "resize 99 88",
        "unsharpen 99.0 88",
        "hue-rotate -3579",
        "dither-quant 64 10",
        "index-colors-quant 64 10",
        "filter3x3 0 0 0 1 1 1 2 2 3.0",
        "filter3x3 1.9 2 3 | 4 5.9 6 | 7 8 9.9",
        "overlay 'input.jpg' 10 5",
        "diff 'input.jpg'",
    }, expected = {
        op(ImgOp::Blur(15.0)),
        op(ImgOp::Blur(-15.0)),
        op(ImgOp::Brighten(-3579)),
        op(ImgOp::Contrast(15.8)),
        op(ImgOp::Crop((1, 2, 3, 4))),
        op(ImgOp::Resize((99, 88))),
        op(ImgOp::Unsharpen((99.0, 88))),
        op(ImgOp::HueRotate(-3579)),
        op(ImgOp::DitherQuant((64, 10))),
        op(ImgOp::IndexColorsQuant((64, 10))),
        op(ImgOp::Filter3x3([0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0])),
        op(ImgOp::Filter3x3([1.9, 2.0, 3.0, 4.0, 5.9, 6.0, 7.0, 8.0, 9.9])),
        op(ImgOp::Overlay(OverlayInputs::new(ImageFromPath::new("input.jpg".into()), (10, 5)))),
        op(ImgOp::Diff(ImageFromPath::new("input.jpg".into()))),
    })]
    fn builds_expected(input: &str, expected: Vec<Instr>) {
        assert_eq!(parse_script(input).unwrap(), expected);
    }

    #[parameterized(input = {
        "horizontal-gradient rgba(10, 10, 255, 255) rgba(255, 255, 255, 255)",
        "vertical-gradient rgba(10, 10, 255, 255) rgba(255, 255, 255, 255)",
    }, expected = {
        op(ImgOp::HorizontalGradient(GradientInput::new((Rgba([10, 10, 255, 255]), Rgba([255, 255, 255, 255]))))),
        op(ImgOp::VerticalGradient(GradientInput::new((Rgba([10, 10, 255, 255]), Rgba([255, 255, 255, 255]))))),
    })]
    fn builds_gradients(input: &str, expected: Vec<Instr>) {
        assert_eq!(parse_script(input).unwrap(), expected);
    }

    #[parameterized(input = {
        "set sampling-filter gaussian",
        "set sampling-filter CatmullRom",
        "set preserve-aspect-ratio true",
        "del sampling-filter",
        "del preserve-aspect-ratio",
    }, expected = {
        vec![Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(FilterType::Gaussian)))],
        vec![Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(FilterType::CatmullRom)))],
        vec![Instr::EnvAdd(EnvItem::PreserveAspectRatio(true))],
        vec![Instr::EnvRemove(ItemName::CustomSamplingFilter)],
        vec![Instr::EnvRemove(ItemName::PreserveAspectRatio)],
    })]
    fn builds_env(input: &str, expected: Vec<Instr>) {
        assert_eq!(parse_script(input).unwrap(), expected);
    }

    #[test]
    fn builds_multi_statement_program() {
        let parsed = parse_script("blur 10;flip-horizontal;flip-vertical;resize 100 200").unwrap();
        assert_eq!(
            parsed,
            vec![
                Instr::Operation(ImgOp::Blur(10.0)),
                Instr::Operation(ImgOp::FlipHorizontal),
                Instr::Operation(ImgOp::FlipVertical),
                Instr::Operation(ImgOp::Resize((100, 200))),
            ]
        );
    }
}
