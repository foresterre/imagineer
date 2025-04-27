use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use sic_core::image::{DynamicImage, GenericImageView};
use sic_core::{SicImage, image};

pub struct Crop {
    // top left anchor
    anchor_left: (u32, u32),
    // bottom right anchor
    anchor_right: (u32, u32),
}

impl Crop {
    pub fn new(anchor_left: (u32, u32), anchor_right: (u32, u32)) -> Self {
        Self {
            anchor_left,
            anchor_right,
        }
    }
}

impl ImageOperation for Crop {
    fn apply_operation(&self, image: &mut SicImage) -> Result<(), SicImageEngineError> {
        match image {
            SicImage::Static(image) => crop_impl(image, self),
            SicImage::Animated(image) => crop_animated_image(image.frames_mut(), self),
        }
    }
}

fn crop_animated_image(frames: &mut [image::Frame], cfg: &Crop) -> Result<(), SicImageEngineError> {
    // Do the in-bounds check on the first frame only: setup
    let index = 0;
    let frame = frames
        .get(index)
        .ok_or(SicImageEngineError::AnimatedFrameUnobtainable(index))?;

    let selection = CropSelection::new(
        cfg.anchor_left.0,
        cfg.anchor_left.1,
        cfg.anchor_right.0,
        cfg.anchor_right.1,
    );
    let (x, y, width, height) = selection.box_dimensions();

    // The in-bounds check
    let _ = selection
        .dimensions_are_ok()
        .and_then(|selection| selection.fits_within(frame.buffer()))?;

    // Crop the frames
    frames.par_iter_mut().for_each(|frame| {
        *frame.buffer_mut() =
            image::imageops::crop(frame.buffer_mut(), x, y, width, height).to_image();
    });

    Ok(())
}

fn crop_impl(image: &mut DynamicImage, cfg: &Crop) -> Result<(), SicImageEngineError> {
    let lx = cfg.anchor_left.0;
    let ly = cfg.anchor_left.1;
    let rx = cfg.anchor_right.0;
    let ry = cfg.anchor_right.1;

    let selection = CropSelection::new(lx, ly, rx, ry);
    // 1. verify that the top left anchor is smaller than the bottom right anchor
    // 2. verify that the selection is within the bounds of the image
    selection
        .dimensions_are_ok()
        .and_then(|selection| selection.fits_within(image))
        .map(|_| *image = image.crop(lx, ly, rx - lx, ry - ly))
}

struct CropSelection {
    lx: u32,
    ly: u32,
    rx: u32,
    ry: u32,
}

impl CropSelection {
    pub(crate) fn new(lx: u32, ly: u32, rx: u32, ry: u32) -> Self {
        Self { lx, ly, rx, ry }
    }

    pub(crate) fn dimensions_are_ok(&self) -> Result<&Self, SicImageEngineError> {
        if self.are_dimensions_incorrect() {
            Err(SicImageEngineError::CropInvalidSelection(
                self.lx, self.ly, self.rx, self.ry,
            ))
        } else {
            Ok(self)
        }
    }

    // returns a tuple containing the x, y, width and height
    fn box_dimensions(&self) -> (u32, u32, u32, u32) {
        (self.lx, self.ly, self.rx - self.lx, self.ry - self.ly)
    }

    fn fits_within<I: GenericImageView>(
        &self,
        bounded_image: &I,
    ) -> Result<&Self, SicImageEngineError> {
        let (dim_x, dim_y) = bounded_image.dimensions();

        match (
            self.lx <= dim_x,
            self.ly <= dim_y,
            self.rx <= dim_x,
            self.ry <= dim_y,
        ) {
            (true, true, true, true) => Ok(self),
            _ => Err(SicImageEngineError::CropCoordinateOutOfBounds(
                dim_x, dim_y, self.lx, self.ly, self.rx, self.ry,
            )),
        }
    }

    fn are_dimensions_incorrect(&self) -> bool {
        (self.rx <= self.lx) || (self.ry <= self.ly)
    }
}
