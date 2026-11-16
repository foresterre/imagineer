use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use sic_core::image::DynamicImage;

#[derive(Debug)]
pub struct Thumbnail {
    width: u32,
    height: u32,
    preserve_aspect_ratio: bool,
}

impl Thumbnail {
    pub fn new(width: u32, height: u32, preserve_aspect_ratio: bool) -> Self {
        Self {
            width,
            height,
            preserve_aspect_ratio,
        }
    }
}

impl ImageOperation for Thumbnail {
    fn apply_to_frame(&self, image: &mut DynamicImage) -> Result<(), SicImageEngineError> {
        *image = if self.preserve_aspect_ratio {
            image.thumbnail(self.width, self.height)
        } else {
            image.thumbnail_exact(self.width, self.height)
        };

        Ok(())
    }
}
