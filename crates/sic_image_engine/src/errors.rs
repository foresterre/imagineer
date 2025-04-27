use sic_core::SicCoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SicImageEngineError {
    #[error("{0}")]
    SicCoreError(#[from] SicCoreError),

    #[error("Could not get frame {0} from animated image")]
    AnimatedFrameUnobtainable(usize),

    #[error(
        "Unable to crop; required top-left anchor < bottom-right anchor; note that (x=0,y=0) is the smallest top-left coordinate; [top-left anchor: (x={0}, y={1}), bottom-right anchor: (x={2}, y={3})]"
    )]
    CropInvalidSelection(u32, u32, u32, u32),

    #[error(
        "Unable to crop; anchor coordinates should be within image bounds [image size: (x={0}, y={1}), top-left anchor: (x={2}, y={3}), bottom-right anchor: (x={4}, y={5})]"
    )]
    CropCoordinateOutOfBounds(u32, u32, u32, u32, u32, u32),

    #[error("Unable to load image argument from given path")]
    LoadImageFromPath,

    #[error("Filter type '{0}' not found")]
    UnknownFilterType(String),

    #[error("Unable to load font: '{0}'")]
    FontError(sic_core::ab_glyph::InvalidFont),

    #[error("Unable to open font file from path: '{0}'")]
    FontFileLoadError(std::io::Error),
}
