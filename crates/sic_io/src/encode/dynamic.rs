use crate::encode::bmp::BmpEncoder;
use crate::encode::jpeg::JpegEncoder;
use crate::encode_settings::EncodeSettings;
use crate::errors::{EncodingError, SicIoError, UnknownImageFormatError};
use sic_core::image;
use sic_core::image::codecs::pnm::PnmSubtype;
use std::fmt;
use std::fmt::Formatter;
use std::io::{Seek, Write};
use std::path::Path;

pub trait IntoImageEncoder<W: Write + Seek> {
    /// Determine the encoding format based on the extension of a file path.
    fn from_extension(
        writer: W,
        path: &Path,
        settings: &EncodeSettings,
    ) -> Result<DynamicEncoder<W>, SicIoError>;

    /// Determine the encoding format based on a recognized given identifier.
    fn from_identifier(
        writer: W,
        identifier: impl AsRef<str>,
        settings: &EncodeSettings,
    ) -> Result<DynamicEncoder<W>, SicIoError>;
}

#[derive(Debug)]
pub struct DynamicEncoder<W: Write + Seek> {
    encoder: DynamicEncoderInner<W>,
    format: DynamicImageFormat,
}

impl<W: Write + Seek> DynamicEncoder<W> {
    pub fn image_output_format(&self) -> DynamicImageFormat {
        self.format
    }
}

impl<W: Write + Seek> DynamicEncoder<W> {
    /// Create a BMP encoder.
    pub fn bmp(writer: W) -> Result<DynamicEncoder<W>, SicIoError> {
        Ok(Self {
            encoder: DynamicEncoderInner::Bmp(BmpEncoder::new(writer)),
            format: DynamicImageFormat::Bmp,
        })
    }
}

impl<W: Write + Seek> IntoImageEncoder<W> for DynamicEncoder<W> {
    /// Determines the encoding format based on the extension of the given path.
    /// If the path has no extension, it will return an error.
    /// The extension if existing is matched against the identifiers, which currently
    /// are the extensions used.
    fn from_extension(
        writer: W,
        path: &Path,
        settings: &EncodeSettings,
    ) -> Result<DynamicEncoder<W>, SicIoError> {
        let extension = path.extension().and_then(|v| v.to_str());

        match extension {
            Some(ext) => Self::from_identifier(writer, ext, settings).map_err(|err| match err {
                SicIoError::UnknownImageFormat(UnknownImageFormatError::Identifier(_)) => {
                    SicIoError::UnknownImageFormat(UnknownImageFormatError::FileExtension(
                        path.to_path_buf(),
                    ))
                }
                e => e,
            }),
            None => Err(SicIoError::UnknownImageFormat(
                UnknownImageFormatError::FileExtension(path.to_path_buf()),
            )),
        }
    }

    /// Determines an image output format based on a given `&str` identifier.
    /// Identifiers are based on common output file extensions.
    fn from_identifier(
        writer: W,
        identifier: impl AsRef<str>,
        settings: &EncodeSettings,
    ) -> Result<DynamicEncoder<W>, SicIoError> {
        use DynamicEncoderInner::*;

        let id = identifier.as_ref();

        let (encoder, format) = match id.to_ascii_lowercase().as_str() {
            "avif" => (
                Avif(image::codecs::avif::AvifEncoder::new(writer)),
                DynamicImageFormat::Avif,
            ),
            "bmp" => (Bmp(BmpEncoder::new(writer)), DynamicImageFormat::Bmp),
            "exr" => (
                Exr(image::codecs::openexr::OpenExrEncoder::new(writer)),
                DynamicImageFormat::Exr,
            ),
            "ff" | "farbfeld" => (
                Farbfeld(image::codecs::farbfeld::FarbfeldEncoder::new(writer)),
                DynamicImageFormat::Farbfeld,
            ),
            "gif" => {
                let mut encoder = image::codecs::gif::GifEncoder::new(writer);
                encoder
                    .set_repeat(settings.repeat_animation.into())
                    .map_err(SicIoError::ImageError)?;

                (Gif(encoder), DynamicImageFormat::Gif)
            }
            "ico" => (
                Ico(image::codecs::ico::IcoEncoder::new(writer)),
                DynamicImageFormat::Ico,
            ),
            "jpeg" | "jpg" => (
                Jpeg(JpegEncoder::new(writer, settings.jpeg_quality)),
                DynamicImageFormat::Jpeg,
            ),
            "pam" => {
                let subtype = PnmSubtype::ArbitraryMap;
                let enc = image::codecs::pnm::PnmEncoder::new(writer).with_subtype(subtype);

                (Pnm(enc), DynamicImageFormat::Pnm { subtype })
            }
            "pbm" => {
                let subtype = PnmSubtype::Bitmap(settings.pnm_sample_encoding);
                let enc = image::codecs::pnm::PnmEncoder::new(writer).with_subtype(subtype);

                (Pnm(enc), DynamicImageFormat::Pnm { subtype })
            }
            "pgm" => {
                let subtype = PnmSubtype::Graymap(settings.pnm_sample_encoding);
                let enc = image::codecs::pnm::PnmEncoder::new(writer).with_subtype(subtype);

                (Pnm(enc), DynamicImageFormat::Pnm { subtype })
            }
            "png" => (
                Png(image::codecs::png::PngEncoder::new(writer)),
                DynamicImageFormat::Png,
            ),
            "ppm" => {
                let subtype = PnmSubtype::Pixmap(settings.pnm_sample_encoding);
                let enc = image::codecs::pnm::PnmEncoder::new(writer).with_subtype(subtype);

                (Pnm(enc), DynamicImageFormat::Pnm { subtype })
            }
            "qoi" => (
                Qoi(image::codecs::qoi::QoiEncoder::new(writer)),
                DynamicImageFormat::Qoi,
            ),
            "tga" => (
                Tga(image::codecs::tga::TgaEncoder::new(writer)),
                DynamicImageFormat::Tga,
            ),
            "tiff" | "tif" => (
                Tiff(image::codecs::tiff::TiffEncoder::new(writer)),
                DynamicImageFormat::Tiff,
            ),
            "webp" => (
                Webp(image::codecs::webp::WebPEncoder::new_lossless(writer)),
                DynamicImageFormat::Webp,
            ),
            _ => {
                return Err(SicIoError::UnknownImageFormat(
                    UnknownImageFormatError::Identifier(id.to_string()),
                ));
            }
        };

        Ok(Self { encoder, format })
    }
}

impl<W: Write + Seek> image::ImageEncoder for DynamicEncoder<W> {
    fn write_image(
        self,
        buf: &[u8],
        width: u32,
        height: u32,
        color_type: image::ExtendedColorType,
    ) -> image::ImageResult<()> {
        match self.encoder {
            DynamicEncoderInner::Avif(enc) => enc.write_image(buf, width, height, color_type),
            DynamicEncoderInner::Bmp(enc) => enc.write_image(buf, width, height, color_type),
            DynamicEncoderInner::Exr(enc) => enc.write_image(buf, width, height, color_type),
            DynamicEncoderInner::Farbfeld(enc) => enc.write_image(buf, width, height, color_type),
            DynamicEncoderInner::Gif(mut enc) => {
                // The `ColorTypePreprocessor` will, if enabled, convert the image to `RgbaImage`
                // if necessary.
                // This is unfortunate though, we're making a copy for sauce.
                let image_buffer = image::RgbaImage::from_raw(width, height, buf.to_vec())
                    .ok_or_else(|| {
                        image::ImageError::Encoding(image::error::EncodingError::new(
                            image::error::ImageFormatHint::Exact(image::ImageFormat::Gif),
                            "Unable to construct frame from raw buffer".to_string(),
                        ))
                    })?;

                enc.encode_frame(image::Frame::new(image_buffer))
            }
            DynamicEncoderInner::Ico(enc) => enc.write_image(buf, width, height, color_type),
            DynamicEncoderInner::Jpeg(enc) => enc.write_image(buf, width, height, color_type),
            DynamicEncoderInner::Pnm(enc) => enc.write_image(buf, width, height, color_type),
            DynamicEncoderInner::Png(enc) => enc.write_image(buf, width, height, color_type),
            DynamicEncoderInner::Qoi(enc) => enc.write_image(buf, width, height, color_type),
            DynamicEncoderInner::Tga(enc) => enc.write_image(buf, width, height, color_type),
            DynamicEncoderInner::Tiff(enc) => enc.write_image(buf, width, height, color_type),
            DynamicEncoderInner::Webp(enc) => enc.write_image(buf, width, height, color_type),
        }
    }
}

impl<W: Write + Seek> DynamicEncoder<W> {
    pub fn write_image_frames(self, frames: Vec<image::Frame>) -> Result<(), SicIoError> {
        match self.encoder {
            DynamicEncoderInner::Gif(mut enc) => {
                enc.encode_frames(frames).map_err(SicIoError::ImageError)
            }
            // Use SingleFramePreprocessor to avoid this error, by picking a single frame
            // from the animated image instead.
            enc => Err(SicIoError::Encoding(
                EncodingError::AnimatedImageUnsupported(enc.image_format()),
            )),
        }
    }
}

impl<W: Write + Seek> DynamicEncoder<W> {
    pub fn image_format(&self) -> image::ImageFormat {
        self.encoder.image_format()
    }
}

#[allow(private_interfaces)]
enum DynamicEncoderInner<W: Write + Seek> {
    Avif(image::codecs::avif::AvifEncoder<W>),
    Bmp(BmpEncoder<W>),
    Exr(image::codecs::openexr::OpenExrEncoder<W>),
    Farbfeld(image::codecs::farbfeld::FarbfeldEncoder<W>),
    Gif(image::codecs::gif::GifEncoder<W>),
    Ico(image::codecs::ico::IcoEncoder<W>),
    Jpeg(JpegEncoder<W>),
    Pnm(image::codecs::pnm::PnmEncoder<W>),
    Png(image::codecs::png::PngEncoder<W>),
    Qoi(image::codecs::qoi::QoiEncoder<W>),
    Tga(image::codecs::tga::TgaEncoder<W>),
    Tiff(image::codecs::tiff::TiffEncoder<W>),
    Webp(image::codecs::webp::WebPEncoder<W>),
}

impl<W: Write + Seek> fmt::Debug for DynamicEncoderInner<W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use DynamicEncoderInner::*;

        match self {
            Avif(_) => f.write_str("DynamicEncoder(Avif)"),
            Bmp(_) => f.write_str("DynamicEncoder(Bmp)"),
            Exr(_) => f.write_str("DynamicEncoder(Exr)"),
            Farbfeld(_) => f.write_str("DynamicEncoder(Farbfeld)"),
            Gif(_) => f.write_str("DynamicEncoder(Gif)"),
            Ico(_) => f.write_str("DynamicEncoder(Ico)"),
            Jpeg(_) => f.write_str("DynamicEncoder(Jpeg)"),
            Pnm(_) => f.write_str("DynamicEncoder(Pnm)"),
            Png(_) => f.write_str("DynamicEncoder(Png)"),
            Qoi(_) => f.write_str("DynamicEncoder(Qoi)"),
            Tga(_) => f.write_str("DynamicEncoder(Tga)"),
            Tiff(_) => f.write_str("DynamicEncoder(Tiff)"),
            Webp(_) => f.write_str("DynamicEncoder(Webp)"),
        }
    }
}

impl<W: Write + Seek> DynamicEncoderInner<W> {
    pub fn image_format(&self) -> image::ImageFormat {
        match self {
            Self::Avif(_) => image::ImageFormat::Avif,
            Self::Bmp(_) => image::ImageFormat::Bmp,
            Self::Exr(_) => image::ImageFormat::OpenExr,
            Self::Farbfeld(_) => image::ImageFormat::Farbfeld,
            Self::Gif(_) => image::ImageFormat::Gif,
            Self::Ico(_) => image::ImageFormat::Ico,
            Self::Jpeg(_) => image::ImageFormat::Jpeg,
            Self::Pnm(_) => image::ImageFormat::Pnm,
            Self::Png(_) => image::ImageFormat::Png,
            Self::Qoi(_) => image::ImageFormat::Qoi,
            Self::Tga(_) => image::ImageFormat::Tga,
            Self::Tiff(_) => image::ImageFormat::Tiff,
            Self::Webp(_) => image::ImageFormat::WebP,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DynamicImageFormat {
    Avif,
    Bmp,
    Exr,
    Farbfeld,
    Gif,
    Ico,
    Jpeg,
    Pnm { subtype: PnmSubtype },
    Png,
    Qoi,
    Tga,
    Tiff,
    Webp,
}

#[cfg(test)]
mod tests {
    use super::*;
    use parameterized::parameterized;
    use std::io::SeekFrom;

    #[derive(Debug)]
    struct DummyMem;

    impl Seek for DummyMem {
        fn seek(&mut self, _pos: SeekFrom) -> std::io::Result<u64> {
            Ok(0)
        }
    }

    impl Write for DummyMem {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Ok(0)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[parameterized(
        ext = {
            "avif",
            "bmp",
            "exr",
            "farbfeld",
            "gif",
            "ico",
            "jpg",
            "jpeg",
            "png",
            "pbm",
            "pgm",
            "ppm",
            "pam",
            "qoi",
            "tga",
            "tiff",
            "tif"
        },
        expected = {
            image::ImageFormat::Avif,
            image::ImageFormat::Bmp,
            image::ImageFormat::OpenExr,
            image::ImageFormat::Farbfeld,
            image::ImageFormat::Gif,
            image::ImageFormat::Ico,
            image::ImageFormat::Jpeg,
            image::ImageFormat::Jpeg,
            image::ImageFormat::Png,
            image::ImageFormat::Pnm,
            image::ImageFormat::Pnm,
            image::ImageFormat::Pnm,
            image::ImageFormat::Pnm,
            image::ImageFormat::Qoi,
            image::ImageFormat::Tga,
            image::ImageFormat::Tiff,
            image::ImageFormat::Tiff,
        }
    )]
    fn test_with_extensions(ext: &str, expected: image::ImageFormat) {
        let path = format!("image.{}", ext);
        let path = Path::new(&path);

        let settings = EncodeSettings::default();
        let mut mem = DummyMem;
        let dynamic_encoder = DynamicEncoder::from_extension(&mut mem, path, &settings);

        assert_eq!(dynamic_encoder.unwrap().image_format(), expected);
    }

    #[parameterized(
        identifier = {
            "avif",
            "bmp",
            "exr",
            "farbfeld",
            "gif",
            "ico",
            "jpg",
            "jpeg",
            "png",
            "pbm",
            "pgm",
            "ppm",
            "pam",
            "qoi",
            "tga",
            "tiff",
            "tif"
        },
        expected = {
            image::ImageFormat::Avif,
            image::ImageFormat::Bmp,
            image::ImageFormat::OpenExr,
            image::ImageFormat::Farbfeld,
            image::ImageFormat::Gif,
            image::ImageFormat::Ico,
            image::ImageFormat::Jpeg,
            image::ImageFormat::Jpeg,
            image::ImageFormat::Png,
            image::ImageFormat::Pnm,
            image::ImageFormat::Pnm,
            image::ImageFormat::Pnm,
            image::ImageFormat::Pnm,
            image::ImageFormat::Qoi,
            image::ImageFormat::Tga,
            image::ImageFormat::Tiff,
            image::ImageFormat::Tiff,
        }
    )]
    fn test_with_identifier(identifier: &str, expected: image::ImageFormat) {
        let settings = EncodeSettings::default();
        let mut mem = DummyMem;
        let dynamic_encoder = DynamicEncoder::from_identifier(&mut mem, identifier, &settings);

        assert_eq!(dynamic_encoder.unwrap().image_format(), expected);
    }

    #[parameterized(
        identifier = {
            "AVIF",
            "BMP",
            "EXR",
            "FARBFELD",
            "GIF",
            "ICO",
            "JPG",
            "JPEG",
            "PNG",
            "PBM",
            "PGM",
            "PPM",
            "PAM",
            "QOI",
            "TGA",
            "TIFF",
            "TIF"
        },
        expected = {
            image::ImageFormat::Avif,
            image::ImageFormat::Bmp,
            image::ImageFormat::OpenExr,
            image::ImageFormat::Farbfeld,
            image::ImageFormat::Gif,
            image::ImageFormat::Ico,
            image::ImageFormat::Jpeg,
            image::ImageFormat::Jpeg,
            image::ImageFormat::Png,
            image::ImageFormat::Pnm,
            image::ImageFormat::Pnm,
            image::ImageFormat::Pnm,
            image::ImageFormat::Pnm,
            image::ImageFormat::Qoi,
            image::ImageFormat::Tga,
            image::ImageFormat::Tiff,
            image::ImageFormat::Tiff,
        }
    )]
    fn test_with_identifier_uppercase(identifier: &str, expected: image::ImageFormat) {
        let settings = EncodeSettings::default();
        let mut mem = DummyMem;
        let dynamic_encoder = DynamicEncoder::from_identifier(&mut mem, identifier, &settings);

        assert_eq!(dynamic_encoder.unwrap().image_format(), expected);
    }

    #[test]
    fn extension_unknown_extension() {
        let path = Path::new("w_ext.h");
        let settings = EncodeSettings::default();
        let mut mem = DummyMem;
        let dynamic_encoder = DynamicEncoder::from_extension(&mut mem, path, &settings);

        assert_eq!(
            dynamic_encoder.unwrap_err(),
            SicIoError::UnknownImageFormat(UnknownImageFormatError::FileExtension(
                Path::new("w_ext.h").to_path_buf()
            ))
        );
    }

    #[test]
    fn extension_no_extension() {
        let path = Path::new("png");
        let settings = EncodeSettings::default();
        let mut mem = DummyMem;
        let dynamic_encoder = DynamicEncoder::from_extension(&mut mem, path, &settings);

        assert_eq!(
            dynamic_encoder.unwrap_err(),
            SicIoError::UnknownImageFormat(UnknownImageFormatError::FileExtension(
                Path::new("png").to_path_buf()
            ))
        );
    }

    #[test]
    fn extension_invalid_extension() {
        let path = Path::new(".png");
        let settings = EncodeSettings::default();
        let mut mem = DummyMem;
        let dynamic_encoder = DynamicEncoder::from_extension(&mut mem, path, &settings);

        assert_eq!(
            dynamic_encoder.unwrap_err(),
            SicIoError::UnknownImageFormat(UnknownImageFormatError::FileExtension(
                Path::new(".png").to_path_buf()
            ))
        );
    }

    #[test]
    fn identifier_unknown_identifier() {
        let path = Path::new("");
        let settings = EncodeSettings::default();
        let mut mem = DummyMem;
        let dynamic_encoder = DynamicEncoder::from_extension(&mut mem, path, &settings);

        assert_eq!(
            dynamic_encoder.unwrap_err(),
            SicIoError::UnknownImageFormat(UnknownImageFormatError::FileExtension(
                Path::new("").to_path_buf()
            ))
        );
    }

    // non default: pnm ascii + "pbm"
    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn identifier_custom_pnm_sample_encoding_ascii_pbm() {
        let mut settings = EncodeSettings::default();
        settings.pnm_sample_encoding = image::codecs::pnm::SampleEncoding::Ascii;

        let mut mem = DummyMem;
        let dynamic_encoder = DynamicEncoder::from_identifier(&mut mem, "pbm", &settings).unwrap();

        assert_eq!(dynamic_encoder.image_format(), image::ImageFormat::Pnm);
    }

    // non default: pnm ascii + "pgm"
    #[parameterized(
        identifier = {
            "pbm",
            "pgm",
            "ppm",
        }
    )]
    #[allow(clippy::field_reassign_with_default)]
    fn identifier_custom_pnm_sample_encoding_ascii_pgm(identifier: &str) {
        let mut settings = EncodeSettings::default();
        settings.pnm_sample_encoding = image::codecs::pnm::SampleEncoding::Ascii;

        let mut mem = DummyMem;
        let dynamic_encoder =
            DynamicEncoder::from_identifier(&mut mem, identifier, &settings).unwrap();

        assert_eq!(dynamic_encoder.image_format(), image::ImageFormat::Pnm);
    }
}
