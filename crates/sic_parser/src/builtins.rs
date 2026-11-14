use crate::ast::{Call, Expr, Literal};
use crate::errors::{ArgumentError, ResolveError};
use crate::types::{Argument, Param, Type};
use sic_core::image::Rgba;
use sic_image_engine::ImgOp;
use sic_image_engine::engine::{EnvItem, Instr, ItemName};
use sic_image_engine::wrapper::draw_text_inner::DrawTextInner;
use sic_image_engine::wrapper::font_options::{FontOptions, FontScale};
use sic_image_engine::wrapper::gradient_input::GradientInput;
use sic_image_engine::wrapper::image_path::ImageFromPath;
use sic_image_engine::wrapper::overlay::OverlayInputs;
use std::path::PathBuf;

pub fn operation<A: Argument>(call: &Call<'_, A>) -> Result<ImgOp, ResolveError> {
    match call.name.to_ascii_lowercase().as_str() {
        "blur" => call.bind(["sigma"]).map(|(sigma,)| ImgOp::Blur(sigma)),
        "brighten" => call
            .bind(["amount"])
            .map(|(amount,)| ImgOp::Brighten(amount)),
        "contrast" => call
            .bind(["amount"])
            .map(|(amount,)| ImgOp::Contrast(amount)),
        "crop" => call.bind(["lx", "ly", "rx", "ry"]).map(ImgOp::Crop),
        "diff" => call
            .bind(["path"])
            .map(|(path,)| ImgOp::Diff(ImageFromPath::new(path))),
        "dither" => call.bind([]).map(|()| ImgOp::Dither),
        "dither-quant" => call
            .bind(["colors", "sample-factor"])
            .map(ImgOp::DitherQuant),
        "draw-text" => call.bind(["text", "coord", "color", "size", "font"]).map(
            |(text, Coord(x, y), Color(color), FontSize(size), Font(font))| {
                let font_options = FontOptions::new(font, color, FontScale::Uniform(size));
                ImgOp::DrawText(DrawTextInner::new(text, (x, y), font_options))
            },
        ),
        "filter3x3" => call
            .bind([
                "k11", "k12", "k13", "k21", "k22", "k23", "k31", "k32", "k33",
            ])
            .map(|(k11, k12, k13, k21, k22, k23, k31, k32, k33)| {
                ImgOp::Filter3x3([k11, k12, k13, k21, k22, k23, k31, k32, k33])
            }),
        "flip-horizontal" => call.bind([]).map(|()| ImgOp::FlipHorizontal),
        "flip-vertical" => call.bind([]).map(|()| ImgOp::FlipVertical),
        "grayscale" => call.bind([]).map(|()| ImgOp::Grayscale),
        "horizontal-gradient" => call.bind(["from", "to"]).map(|(Color(from), Color(to))| {
            ImgOp::HorizontalGradient(GradientInput::new((from, to)))
        }),
        "hue-rotate" => call
            .bind(["degrees"])
            .map(|(degrees,)| ImgOp::HueRotate(degrees)),
        "index-colors" => call.bind([]).map(|()| ImgOp::IndexColors),
        "index-colors-quant" => call
            .bind(["colors", "sample-factor"])
            .map(ImgOp::IndexColorsQuant),
        "invert" => call.bind([]).map(|()| ImgOp::Invert),
        "overlay" => call
            .bind(["path", "x", "y"])
            .map(|(path, x, y): (PathBuf, u32, u32)| {
                let position = (i64::from(x), i64::from(y));
                ImgOp::Overlay(OverlayInputs::new(ImageFromPath::new(path), position))
            }),
        "resize" => call.bind(["width", "height"]).map(ImgOp::Resize),
        "rotate90" => call.bind([]).map(|()| ImgOp::Rotate90),
        "rotate180" => call.bind([]).map(|()| ImgOp::Rotate180),
        "rotate270" => call.bind([]).map(|()| ImgOp::Rotate270),
        "threshold" => call.bind([]).map(|()| ImgOp::Threshold),
        "unsharpen" => call.bind(["sigma", "threshold"]).map(ImgOp::Unsharpen),
        "vertical-gradient" => call.bind(["from", "to"]).map(|(Color(from), Color(to))| {
            ImgOp::VerticalGradient(GradientInput::new((from, to)))
        }),
        _ => Err(ResolveError::UnknownOperation {
            name: call.name.to_string(),
        }),
    }
}

pub fn set_option<A: Argument>(option: &str, value: &A) -> Result<Instr, ResolveError> {
    match option.to_ascii_lowercase().as_str() {
        "sampling-filter" => bind_option(option, value)
            .map(|filter| Instr::EnvAdd(EnvItem::CustomSamplingFilter(filter))),
        "preserve-aspect-ratio" => bind_option(option, value)
            .map(|enabled| Instr::EnvAdd(EnvItem::PreserveAspectRatio(enabled))),
        _ => Err(ResolveError::UnknownOption {
            name: option.to_string(),
        }),
    }
}

pub fn del_option(option: &str) -> Result<Instr, ResolveError> {
    match option.to_ascii_lowercase().as_str() {
        "sampling-filter" => Ok(Instr::EnvRemove(ItemName::CustomSamplingFilter)),
        "preserve-aspect-ratio" => Ok(Instr::EnvRemove(ItemName::PreserveAspectRatio)),
        _ => Err(ResolveError::UnknownOption {
            name: option.to_string(),
        }),
    }
}

fn bind_option<P: Param, A: Argument>(option: &str, value: &A) -> Result<P, ResolveError> {
    value.bind().map_err(|source| ResolveError::OptionValue {
        option: option.to_string(),
        source: Box::new(source),
    })
}

pub struct Color(pub Rgba<u8>);

impl Param for Color {
    const TYPE: Type = Type::Color;

    fn from_expr(expr: &Expr<'_>) -> Result<Self, ArgumentError> {
        let (r, g, b, a) = constructor(expr, "rgba", Self::TYPE)?.bind(["r", "g", "b", "a"])?;
        Ok(Color(Rgba([r, g, b, a])))
    }
}

pub struct Coord(pub i32, pub i32);

impl Param for Coord {
    const TYPE: Type = Type::Coord;

    fn from_expr(expr: &Expr<'_>) -> Result<Self, ArgumentError> {
        let (x, y) = constructor(expr, "coord", Self::TYPE)?.bind(["x", "y"])?;
        Ok(Coord(x, y))
    }
}

pub struct FontSize(pub f32);

impl Param for FontSize {
    const TYPE: Type = Type::FontSize;

    fn from_expr(expr: &Expr<'_>) -> Result<Self, ArgumentError> {
        let (px,) = constructor(expr, "size", Self::TYPE)?.bind(["px"])?;
        Ok(FontSize(px))
    }
}

pub struct Font(pub PathBuf);

impl Param for Font {
    const TYPE: Type = Type::Font;

    fn from_expr(expr: &Expr<'_>) -> Result<Self, ArgumentError> {
        let (path,) = constructor(expr, "font", Self::TYPE)?.bind(["path"])?;
        Ok(Font(path))
    }
}

fn constructor<'e, 's>(
    expr: &'e Expr<'s>,
    name: &str,
    expected: Type,
) -> Result<&'e Call<'s, Literal<'s>>, ArgumentError> {
    match expr {
        Expr::Call(call) if call.name.eq_ignore_ascii_case(name) => Ok(call),
        other => Err(ArgumentError::mismatch(expected, other)),
    }
}
