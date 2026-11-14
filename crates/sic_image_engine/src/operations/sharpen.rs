use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::{DynamicImage, Pixel};
use sic_core::imageproc::definitions::{Clamp, Image};
use sic_core::imageproc::filter::filter;
use sic_core::imageproc::kernel::Kernel;

const KERNEL: Kernel<'static, i32> = Kernel::new(&[0, -1, 0, -1, 5, -1, 0, -1, 0], 3, 3);

const KERNEL_F32: Kernel<'static, f32> =
    Kernel::new(&[0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0], 3, 3);

pub struct Sharpen;

impl Sharpen {
    pub fn new() -> Self {
        Self
    }
}

impl ImageOperation for Sharpen {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        *image = sharpen(image);

        Ok(())
    }
}

fn sharpen(image: &DynamicImage) -> DynamicImage {
    match image {
        DynamicImage::ImageLuma8(buffer) => DynamicImage::ImageLuma8(sharpen_int(buffer)),
        DynamicImage::ImageLumaA8(buffer) => DynamicImage::ImageLumaA8(sharpen_int(buffer)),
        DynamicImage::ImageRgb8(buffer) => DynamicImage::ImageRgb8(sharpen_int(buffer)),
        DynamicImage::ImageRgba8(buffer) => DynamicImage::ImageRgba8(sharpen_int(buffer)),
        DynamicImage::ImageLuma16(buffer) => DynamicImage::ImageLuma16(sharpen_int(buffer)),
        DynamicImage::ImageLumaA16(buffer) => DynamicImage::ImageLumaA16(sharpen_int(buffer)),
        DynamicImage::ImageRgb16(buffer) => DynamicImage::ImageRgb16(sharpen_int(buffer)),
        DynamicImage::ImageRgba16(buffer) => DynamicImage::ImageRgba16(sharpen_int(buffer)),
        DynamicImage::ImageRgb32F(buffer) => DynamicImage::ImageRgb32F(sharpen_f32(buffer)),
        DynamicImage::ImageRgba32F(buffer) => DynamicImage::ImageRgba32F(sharpen_f32(buffer)),
        other => DynamicImage::ImageRgba32F(sharpen_f32(&other.to_rgba32f())),
    }
}

fn sharpen_int<P>(image: &Image<P>) -> Image<P>
where
    P: Pixel,
    P::Subpixel: Into<i32> + Clamp<i32>,
{
    filter(image, KERNEL, <P::Subpixel as Clamp<i32>>::clamp)
}

fn sharpen_f32<P>(image: &Image<P>) -> Image<P>
where
    P: Pixel<Subpixel = f32>,
{
    filter(image, KERNEL_F32, |value| value.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sic_core::image::{GrayImage, Luma, Rgb, Rgb32FImage, RgbImage};

    #[test]
    fn flat_image_is_unchanged_including_edges() {
        let image = RgbImage::from_pixel(4, 4, Rgb([10, 120, 250]));

        assert_eq!(sharpen_int(&image), image);
    }

    #[test]
    fn center_spike_is_amplified_and_clamped() {
        let mut image = GrayImage::from_pixel(3, 3, Luma([50]));
        image.put_pixel(1, 1, Luma([100]));

        let out = sharpen_int(&image);

        assert_eq!(out.get_pixel(1, 1), &Luma([255]));
        assert_eq!(out.get_pixel(0, 1), &Luma([0]));
        assert_eq!(out.get_pixel(0, 0), &Luma([50]));
    }

    #[test]
    fn float_values_stay_in_unit_range() {
        let mut image = Rgb32FImage::from_pixel(3, 3, Rgb([0.5, 0.5, 0.5]));
        image.put_pixel(1, 1, Rgb([1.0, 0.0, 0.5]));

        let out = sharpen_f32(&image);

        assert_eq!(out.get_pixel(1, 1), &Rgb([1.0, 0.0, 0.5]));
        assert!(
            out.pixels()
                .flat_map(|pixel| pixel.0)
                .all(|v| (0.0..=1.0).contains(&v))
        );
    }

    #[test]
    fn keeps_color_type() {
        let image = DynamicImage::new_rgba16(4, 4);

        assert_eq!(sharpen(&image).color(), image.color());
    }
}
