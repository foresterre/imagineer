use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read, Seek};
use std::path::Path;

use crate::errors::SicIoError;
use sic_core::{AnimatedImage, SicImage, image};

#[derive(Default)]
pub struct SicImageDecoder {
    /// For animated images, this frame will be used if we can only decode into a static image.
    selected_frame: Option<FrameIndex>,
}

impl SicImageDecoder {
    pub fn new(selected_frame: Option<FrameIndex>) -> Self {
        Self { selected_frame }
    }
}

impl SicImageDecoder {
    /// Load an image using a reader.
    /// All images are currently loaded from memory.
    pub fn decode<R: Read>(&self, reader: &mut R) -> Result<SicImage, SicIoError> {
        let reader = image::ImageReader::new(Cursor::new(read_image_to_buffer(reader)?))
            .with_guessed_format()
            .map_err(SicIoError::Io)?;

        match reader.format() {
            Some(image::ImageFormat::Png) => decode_png(reader, self.selected_frame),
            Some(image::ImageFormat::Gif) => decode_gif(reader, self.selected_frame),
            Some(_) => reader
                .decode()
                .map_err(SicIoError::ImageError)
                .map(SicImage::from),
            None => Err(SicIoError::ImageError(image::error::ImageError::Decoding(
                image::error::DecodingError::from_format_hint(
                    image::error::ImageFormatHint::Unknown,
                ),
            ))),
        }
    }
}

/// Constructs a reader which reads from the stdin.
pub fn stdin_reader() -> Result<Box<dyn Read>, SicIoError> {
    Ok(Box::new(BufReader::new(std::io::stdin())))
}

/// Constructs a reader which reads from a file path.
pub fn file_reader<P: AsRef<Path>>(path: P) -> Result<Box<dyn Read>, SicIoError> {
    Ok(Box::new(BufReader::new(
        File::open(path).map_err(SicIoError::Io)?,
    )))
}

// Let the reader store the raw bytes into a buffer.
fn read_image_to_buffer<R: Read>(reader: &mut R) -> Result<Vec<u8>, SicIoError> {
    let mut buffer = Vec::new();
    let _size = reader.read_to_end(&mut buffer).map_err(SicIoError::Io)?;
    Ok(buffer)
}

/// Decode an image into frames
fn frames<'decoder, D: image::AnimationDecoder<'decoder>>(
    decoder: D,
) -> Result<SicImage, SicIoError> {
    let mut frames = decoder
        .into_frames()
        .collect_frames()
        .map_err(SicIoError::ImageError)?;

    if frames.len() == 1 {
        // SAFETY(unwrap): We just checked .len() to be 1
        let buffer = frames.pop().unwrap();
        let buffer = buffer.into_buffer();
        Ok(SicImage::Static(image::DynamicImage::ImageRgba8(buffer)))
    } else {
        Ok(SicImage::Animated(AnimatedImage::from_frames(frames)))
    }
}

fn select_frame(image: SicImage, frame_index: Option<FrameIndex>) -> Result<SicImage, SicIoError> {
    match (image, frame_index) {
        (SicImage::Animated(animated), Some(index)) => {
            let max_frames = animated.frames().len();
            Ok(SicImage::Static(
                animated.try_into_static_image(index.as_number(max_frames))?,
            ))
        }
        (img, _) => Ok(img),
    }
}

/// Zero-indexed frame index.
#[derive(Clone, Copy, Debug)]
pub enum FrameIndex {
    First,
    Last,
    Nth(usize),
}

impl FrameIndex {
    pub fn as_number(&self, frame_count: usize) -> usize {
        match self {
            Self::First => 0,
            Self::Last => frame_count - 1,
            Self::Nth(n) => *n,
        }
    }
}

fn decode_gif<R: BufRead + Seek>(
    reader: image::ImageReader<R>,
    frame_index: Option<FrameIndex>,
) -> Result<SicImage, SicIoError> {
    let decoder =
        image::codecs::gif::GifDecoder::new(reader.into_inner()).map_err(SicIoError::ImageError)?;

    frames(decoder).and_then(|image| select_frame(image, frame_index))
}

fn decode_png<R: BufRead + Seek>(
    reader: image::ImageReader<R>,
    frame: Option<FrameIndex>,
) -> Result<SicImage, SicIoError> {
    let decoder =
        image::codecs::png::PngDecoder::new(reader.into_inner()).map_err(SicIoError::ImageError)?;

    if decoder.is_apng().map_err(SicIoError::ImageError)? {
        frames(decoder.apng().map_err(SicIoError::ImageError)?).and_then(|f| select_frame(f, frame))
    } else {
        image::DynamicImage::from_decoder(decoder)
            .map_err(SicIoError::ImageError)
            .map(SicImage::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parameterized::parameterized;
    use sic_testing::*;

    const GIF_LOOP: &str = "loop.gif";
    const GIF_NO_LOOP: &str = "noloop.gif";
    const XY: u32 = 10;

    #[test]
    fn load_gif_non_looping_frame_first() {
        let load_path = setup_test_image(GIF_NO_LOOP);

        let decoder = SicImageDecoder::new(Some(FrameIndex::First));
        let image = decoder
            .decode(&mut file_reader(load_path).unwrap())
            .unwrap();

        // color = red
        let expected: [u8; 4] = [254, 0, 0, 255];
        assert_eq!(image.get_pixel(XY, XY).0, expected);
    }

    #[test]
    fn load_gif_non_looping_frame_first_is_zero() {
        let load_path = setup_test_image(GIF_NO_LOOP);

        let decoder_first = SicImageDecoder::new(Some(FrameIndex::First));
        let decoder_zero = SicImageDecoder::new(Some(FrameIndex::Nth(0)));

        let first = decoder_first
            .decode(&mut file_reader(&load_path).unwrap())
            .unwrap();
        let zero = decoder_zero
            .decode(&mut file_reader(&load_path).unwrap())
            .unwrap();

        assert_eq!(first.get_pixel(XY, XY).0, zero.get_pixel(XY, XY).0);
    }

    #[test]
    fn load_gif_looping_frame_first_is_zero() {
        let load_path = setup_test_image(GIF_LOOP);

        let decoder_first = SicImageDecoder::new(Some(FrameIndex::First));
        let decoder_zero = SicImageDecoder::new(Some(FrameIndex::Nth(0)));

        let first = decoder_first
            .decode(&mut file_reader(&load_path).unwrap())
            .unwrap();
        let zero = decoder_zero
            .decode(&mut file_reader(&load_path).unwrap())
            .unwrap();

        assert_eq!(first.get_pixel(XY, XY).0, zero.get_pixel(XY, XY).0);
    }

    // [[expected color]; amount]
    // verified with pastel cli;
    const FRAME_COLORS: [[u8; 4]; 8] = [
        [254, 0, 0, 255],     // red
        [254, 165, 0, 255],   // orange
        [255, 255, 0, 255],   // yellow
        [0, 128, 1, 255],     // green
        [0, 0, 254, 255],     // blue
        [75, 0, 129, 255],    // indigo
        [238, 130, 239, 255], // violet
        [0, 0, 0, 255],       // black
    ];

    #[test]
    fn load_gif_non_looping_frame_nth() {
        for (i, expected) in FRAME_COLORS.iter().enumerate() {
            let load_path = setup_test_image(GIF_NO_LOOP);

            let decoder = SicImageDecoder::new(Some(FrameIndex::Nth(i)));
            let image = decoder
                .decode(&mut file_reader(load_path).unwrap())
                .unwrap();

            assert_eq!(&image.get_pixel(XY, XY).0, expected);
        }
    }

    #[test]
    fn load_gif_looping_frame_nth() {
        for (i, expected) in FRAME_COLORS.iter().enumerate() {
            let load_path = setup_test_image(GIF_LOOP);

            let decoder = SicImageDecoder::new(Some(FrameIndex::Nth(i)));
            let image = decoder
                .decode(&mut file_reader(load_path).unwrap())
                .unwrap();

            assert_eq!(&image.get_pixel(XY, XY).0, expected);
        }
    }

    #[test]
    fn load_gif_non_looping_frame_nth_beyond_length() {
        let load_path = setup_test_image(GIF_NO_LOOP);

        let decoder = SicImageDecoder::new(Some(FrameIndex::Nth(8)));
        let result = decoder.decode(&mut file_reader(load_path).unwrap());
        assert!(result.is_err());
    }

    // Even if the gif loops, it still has 8 frames.
    #[test]
    fn load_gif_looping_frame_nth_beyond_length() {
        let load_path = setup_test_image(GIF_LOOP);

        let decoder = SicImageDecoder::new(Some(FrameIndex::Nth(8)));
        let result = decoder.decode(&mut file_reader(load_path).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn load_gif_non_looping_frame_last_is_seven_index() {
        let load_path = setup_test_image(GIF_NO_LOOP);

        let decoder_last = SicImageDecoder::new(Some(FrameIndex::Last));
        let decoder_seventh = SicImageDecoder::new(Some(FrameIndex::Nth(7)));

        let last = decoder_last
            .decode(&mut file_reader(&load_path).unwrap())
            .unwrap();
        let seven = decoder_seventh
            .decode(&mut file_reader(&load_path).unwrap())
            .unwrap();

        assert_eq!(last.get_pixel(XY, XY).0, seven.get_pixel(XY, XY).0);
    }

    #[test]
    fn load_gif_looping_frame_last_is_seven_index() {
        let load_path = setup_test_image(GIF_LOOP);

        let decoder_last = SicImageDecoder::new(Some(FrameIndex::Last));
        let decoder_seventh = SicImageDecoder::new(Some(FrameIndex::Nth(7)));

        let last = decoder_last
            .decode(&mut file_reader(&load_path).unwrap())
            .unwrap();
        let seven = decoder_seventh
            .decode(&mut file_reader(&load_path).unwrap())
            .unwrap();

        assert_eq!(last.get_pixel(XY, XY).0, seven.get_pixel(XY, XY).0);
    }

    const NOT_GIFS: [&str; 3] = [
        "blackwhite_2x2.bmp",
        "bwlines.png",
        "unsplash_763569_cropped.jpg",
    ];

    #[test]
    fn load_not_gif_formatted() {
        for path in NOT_GIFS.iter() {
            let load_path = setup_test_image(path);
            let decoder = SicImageDecoder::default();
            let result = decoder.decode(&mut file_reader(load_path).unwrap());
            assert!(result.is_ok());
        }
    }

    mod apng {
        use super::*;

        const APNG_SAMPLE: &str = "apng_sample.png";

        #[parameterized(
            frame = {
                Some(FrameIndex::First),
                Some(FrameIndex::Nth(0)),
                Some(FrameIndex::Nth(1)),
                Some(FrameIndex::Nth(2)),
                Some(FrameIndex::Last),
                Some(FrameIndex::Nth(3)),
            },
            expected_color = {
                Some([255, 255, 255, 255]),
                Some([255, 255, 255, 255]),
                Some([237, 28, 36, 255]), // default red in paint.exe
                Some([0, 0, 0, 255]),
                Some([0, 0, 0, 255]),
                None,
            }
        )]
        fn apng(frame: Option<FrameIndex>, expected_color: Option<[u8; 4]>) {
            let load_path = setup_test_image(APNG_SAMPLE);

            let decoder = SicImageDecoder::new(frame);
            let result = decoder.decode(&mut file_reader(load_path).unwrap());

            match expected_color {
                Some(expected) => assert_eq!(result.unwrap().get_pixel(0, 0).0, expected),
                None => assert!(result.is_err()),
            }
        }
    }
}
