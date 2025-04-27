use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::{
    SicImage,
    image::{self, DynamicImage, RgbaImage},
    imageproc,
};

pub struct Threshold;

impl Threshold {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImageOperation for Threshold {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => *image = threshold_image(image),
            SicImage::Animated(image) => threshold_animated_image(image.frames_mut()),
        }

        Ok(())
    }
}

fn threshold_image(img: &mut DynamicImage) -> DynamicImage {
    let gray_image = img.to_luma8();
    let best_threshold = imageproc::contrast::otsu_level(&gray_image);
    let out = imageproc::contrast::threshold(
        &gray_image,
        best_threshold,
        imageproc::contrast::ThresholdType::Truncate,
    );
    DynamicImage::ImageLuma8(out)
}

fn threshold_frame(img: &RgbaImage) -> RgbaImage {
    let gray_image = DynamicImage::ImageRgba8(img.clone()).into_luma8();
    let best_threshold = imageproc::contrast::otsu_level(&gray_image);
    let out = imageproc::contrast::threshold(
        &gray_image,
        best_threshold,
        imageproc::contrast::ThresholdType::Truncate,
    );
    DynamicImage::ImageLuma8(out).into_rgba8()
}

fn threshold_animated_image(frames: &mut [image::Frame]) {
    frames.par_iter_mut().for_each(|frame| {
        *frame.buffer_mut() = threshold_frame(frame.buffer());
    });
}
