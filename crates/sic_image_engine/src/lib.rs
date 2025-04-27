#![deny(clippy::all)]
#![allow(clippy::unnecessary_wraps)]

#[macro_use]
extern crate strum_macros;

use crate::wrapper::draw_text_inner::DrawTextInner;
use crate::wrapper::gradient_input::GradientInput;
use crate::wrapper::image_path::ImageFromPath;
use crate::wrapper::overlay::OverlayInputs;

pub mod engine;
pub mod errors;
pub(crate) mod helper;
pub mod operations;
pub mod wrapper;

#[derive(Debug, PartialEq, Clone)]
pub enum ImgOp {
    Blur(f32),
    Brighten(i32),
    Contrast(f32),
    Crop((u32, u32, u32, u32)),
    Diff(ImageFromPath),

    DrawText(DrawTextInner),
    Filter3x3([f32; 9]),
    FlipHorizontal,
    FlipVertical,
    Grayscale,
    HueRotate(i32),
    HorizontalGradient(GradientInput),
    Invert,
    Overlay(OverlayInputs),
    Resize((u32, u32)),
    Rotate90,
    Rotate180,
    Rotate270,

    Threshold,
    Unsharpen((f32, i32)),
    VerticalGradient(GradientInput),
}
