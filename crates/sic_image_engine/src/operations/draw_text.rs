use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use crate::wrapper::draw_text_inner::DrawTextInner;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::image::DynamicImage;
use sic_core::{SicImage, ab_glyph, image, imageproc};

pub struct DrawText<'dt> {
    text: &'dt DrawTextInner,
}

impl<'dt> DrawText<'dt> {
    pub fn new(text: &'dt DrawTextInner) -> Self {
        Self { text }
    }
}

impl ImageOperation for DrawText<'_> {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => draw_text_static_image(image, self.text),
            SicImage::Animated(image) => draw_text_animated_image(image.frames_mut(), self.text),
        }
    }
}

fn draw_text_animated_image(
    frames: &mut [image::Frame],
    inner: &DrawTextInner,
) -> Result<(), SicImageEngineError> {
    let text = inner.text();
    let coords = inner.coords();
    let font_options = inner.font_options();
    let font_file =
        std::fs::read(&font_options.font_path).map_err(SicImageEngineError::FontFileLoadError)?;
    let font =
        ab_glyph::FontVec::try_from_vec(font_file).map_err(SicImageEngineError::FontError)?;

    frames.par_iter_mut().for_each(|frame| {
        imageproc::drawing::draw_text_mut(
            frame.buffer_mut(),
            font_options.color,
            coords.0,
            coords.1,
            font_options.scale,
            &font,
            text,
        );
    });

    Ok(())
}

fn draw_text_static_image(
    image: &mut DynamicImage,
    inner: &DrawTextInner,
) -> Result<(), SicImageEngineError> {
    let text = inner.text();
    let coords = inner.coords();
    let font_options = inner.font_options();
    let font_file =
        std::fs::read(&font_options.font_path).map_err(SicImageEngineError::FontFileLoadError)?;
    let font =
        ab_glyph::FontVec::try_from_vec(font_file).map_err(SicImageEngineError::FontError)?;

    *image = DynamicImage::ImageRgba8(imageproc::drawing::draw_text(
        image,
        font_options.color,
        coords.0,
        coords.1,
        font_options.scale,
        &font,
        text,
    ));

    Ok(())
}
