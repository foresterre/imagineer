use crate::encode::dynamic::DynamicImageFormat;
use crate::errors::SicIoError;
use crate::preprocessor::color_type::ColorTypePreprocessor;
use crate::preprocessor::pick_frame::PickFramePreprocessor;
use sic_core::{SicImage, image};

pub mod color_type;
pub mod pick_frame;

pub trait Preprocess {
    type Err;

    fn preprocess(&self, image: SicImage) -> Result<SicImage, Self::Err>;
}

#[derive(Default)]
pub struct Preprocessors {
    preprocessors: Vec<Box<dyn Preprocess<Err = SicIoError>>>,
}

impl Preprocessors {
    pub fn color_type_preprocessor(&mut self, format: DynamicImageFormat) -> &mut Self {
        self.preprocessors
            .push(Box::new(ColorTypePreprocessor::new(format)));

        self
    }

    pub fn pick_frame_preprocessor(&mut self, image_format: image::ImageFormat) -> &mut Self {
        self.preprocessors
            .push(Box::new(PickFramePreprocessor::new(image_format)));

        self
    }

    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn Preprocess<Err = SicIoError>>> {
        self.preprocessors.iter()
    }
}
