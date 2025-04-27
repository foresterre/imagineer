use std::borrow::BorrowMut;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Stdout, Write};

use crate::cli::config::{Config, InputOutputMode, InputOutputModeType, PathVariant};
use crate::cli::license::LicenseTexts;
use crate::cli::license::PrintTextFor;
use anyhow::{Context, anyhow, bail};
use sic_core::image;
use sic_image_engine::engine::ImageEngine;
use sic_io::decode;
use sic_io::decode::SicImageDecoder;
use sic_io::encode::SicImageEncoder;
use sic_io::encode::dynamic::{DynamicEncoder, IntoImageEncoder};
use sic_io::encode_settings::EncodeSettings;
use sic_io::encode_settings::jpeg::JpegQuality;
use sic_io::preprocessor::Preprocessors;

pub fn run_with_devices<'c>(
    in_and_output: InputOutputMode,
    config: &'c Config<'c>,
) -> anyhow::Result<()> {
    match in_and_output {
        InputOutputMode::Single { input, output } => {
            if output.is_std_stream() {
                warn_default_std_output_format();
            }

            run(
                || create_reader(&input),
                |ext: Option<&str>| create_writer(&output, ext),
                config,
                &output,
            )
            .with_context(|| format!("With: {}", input.describe_input()))
        }
        InputOutputMode::Batch {
            inputs,
            output_root_folder,
        } => {
            for (input, branch) in inputs.path_combinations() {
                let input = &PathVariant::Path(input.to_path_buf());

                let output = output_root_folder.join(branch);
                let output = &PathVariant::Path(output);

                run(
                    || create_reader(input),
                    |ext: Option<&str>| create_writer(output, ext),
                    config,
                    output,
                )
                .with_context(|| format!("With input: {}", input.describe_input()))?
            }

            Ok(())
        }
    }
}

fn warn_default_std_output_format() {
    eprintln!(
        "warn: The default output format when using stdout output (the current output mode) is \
             BMP. Other formats can be use by providing --output-format <FORMAT>."
    );
}

// TODO: simplify inputs of this function
fn run<R, W, WS>(
    supply_reader: R,
    supply_writer: W,
    config: &Config,
    output_path_variant: &PathVariant,
) -> anyhow::Result<()>
where
    R: Fn() -> anyhow::Result<Box<dyn Read>>,
    W: Fn(Option<&str>) -> anyhow::Result<WS>,
    WS: Write + Seek,
{
    let mut reader = supply_reader()?;

    // Decode
    let decoder = SicImageDecoder::new(config.selected_frame);
    let img = decoder.decode(&mut reader)?;

    // Apply image operations
    let image_engine = ImageEngine::new(img);
    let buffer = image_engine
        .ignite(&config.image_operations_program)
        .with_context(|| "Unable to apply image operations.")?;

    // FIXME: decide whether in simple mode, extension should also change by default,
    //        unless an option is set e.g. --keep-extension-unmodified
    let format = if config.mode == InputOutputModeType::Batch {
        config.forced_output_format
    } else {
        None
    };

    // Create a writer and the encoder
    let writer = supply_writer(format)?;
    let encode_settings = create_encode_settings(config)?;
    let dynamic_encoder =
        create_dynamic_encoder(writer, config, &encode_settings, output_path_variant)?;

    // Add preprocessors
    //
    // NB: order in which preprocessors are added matters!
    let mut preprocessors = Preprocessors::default();

    preprocessors.pick_frame_preprocessor(dynamic_encoder.image_format());

    if !config.disable_automatic_color_type_adjustment {
        preprocessors.color_type_preprocessor(dynamic_encoder.image_output_format());
    }

    // Encode
    let encoder = SicImageEncoder::new(preprocessors);

    encoder
        .encode(buffer, dynamic_encoder)
        .with_context(|| "Unable to write image")
}

/// Create a reader which will be used to load the image.
/// The reader can be a file or the stdin.
/// If no file path is provided, the stdin will be assumed.
fn create_reader(path_variant: &PathVariant) -> anyhow::Result<Box<dyn Read>> {
    match path_variant {
        PathVariant::StdStream if atty::is(atty::Stream::Stdin) => bail!(
            "An input image should be given by providing a path using the input argument or \
                 by piping an image to the stdin."
        ),
        PathVariant::StdStream => Ok(decode::stdin_reader()?),
        PathVariant::Path(path) => Ok(decode::file_reader(path)?),
    }
}

fn create_encode_settings(config: &Config) -> anyhow::Result<EncodeSettings> {
    Ok(EncodeSettings {
        pnm_sample_encoding: if config.encoding_settings.pnm_use_ascii_format {
            image::codecs::pnm::SampleEncoding::Ascii
        } else {
            image::codecs::pnm::SampleEncoding::Binary
        },
        jpeg_quality: { JpegQuality::try_from(config.encoding_settings.jpeg_quality)? },
        repeat_animation: config.encoding_settings.gif_repeat,
    })
}

fn create_dynamic_encoder<W: Write + Seek>(
    writer: W,
    config: &Config,
    encode_settings: &EncodeSettings,
    path_variant: &PathVariant,
) -> anyhow::Result<DynamicEncoder<W>> {
    Ok(match &config.forced_output_format {
        Some(format) => DynamicEncoder::from_identifier(writer, format, encode_settings)?,
        None => match path_variant {
            PathVariant::Path(out) => DynamicEncoder::from_extension(writer, out, encode_settings)?,
            PathVariant::StdStream => DynamicEncoder::bmp(writer)?,
        },
    })
}

#[derive(Debug)]
enum OutputType {
    File(File),
    Stdout(Stdout),
}

#[derive(Debug)]
struct Output {
    output_type: OutputType,
    written_bytes: usize,
}

impl Output {
    pub fn new_file(file: File) -> Self {
        Self {
            output_type: OutputType::File(file),
            written_bytes: 0,
        }
    }

    pub fn new_stdout(stdout: Stdout) -> Self {
        Self {
            output_type: OutputType::Stdout(stdout),
            written_bytes: 0,
        }
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.output_type.borrow_mut() {
            OutputType::File(f) => f.write(buf),
            OutputType::Stdout(stdout) => {
                let bytes = stdout.write(buf)?;
                self.written_bytes += bytes;

                Ok(bytes)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.output_type.borrow_mut() {
            OutputType::File(f) => f.flush(),
            OutputType::Stdout(stdout) => stdout.flush(),
        }
    }
}

impl Seek for Output {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match self.output_type.borrow_mut() {
            OutputType::File(f) => f.seek(pos),
            OutputType::Stdout(_) => Ok(self.written_bytes as u64),
        }
    }
}

fn create_writer(path_variant: &PathVariant, adjust_ext: Option<&str>) -> anyhow::Result<Output> {
    match path_variant {
        PathVariant::Path(out) => {
            let base = out.as_path().parent().ok_or_else(|| {
                anyhow::anyhow!("Unable to create output directory for output path")
            })?;
            std::fs::create_dir_all(base)?;

            let out = match (adjust_ext, out.file_stem()) {
                (Some(new_ext), Some(stem)) => base.join(stem).with_extension(new_ext),
                _ => out.to_path_buf(),
            };

            let file = File::create(out)?;

            Ok(Output::new_file(file))
        }
        PathVariant::StdStream => Ok(Output::new_stdout(io::stdout())),
    }
}

pub fn run_display_licenses(config: &Config, texts: &LicenseTexts) -> anyhow::Result<()> {
    config
        .show_license_text_of
        .ok_or_else(|| anyhow!("Unable to determine which license texts should be displayed."))
        .and_then(|license_text| license_text.print(texts))
}
