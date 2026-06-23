use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;

pub struct Filter3x3<'kernel> {
    kernel: &'kernel [f32; 9],
}

impl<'kernel> Filter3x3<'kernel> {
    pub fn new(kernel: &'kernel [f32; 9]) -> Self {
        Self { kernel }
    }
}

impl ImageOperation for Filter3x3<'_> {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        *image = image.filter3x3(self.kernel);

        Ok(())
    }
}
