use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use crate::wrapper::gradient_fn;
use crate::wrapper::gradient_input::GradientInput;
use sic_core::SicImage;
use sic_core::image::imageops;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HorizontalGradient {
    pub colors: GradientInput,
}

impl HorizontalGradient {
    pub fn new(colors: GradientInput) -> Self {
        Self { colors }
    }
}

impl ImageOperation for HorizontalGradient {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => gradient_fn::gradient_static_image(
                image,
                self.colors,
                imageops::horizontal_gradient,
            ),
            SicImage::Animated(image) => gradient_fn::gradient_animated_image(
                image.frames_mut(),
                self.colors,
                imageops::horizontal_gradient,
            ),
        }

        Ok(())
    }
}
