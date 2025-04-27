use crate::cli::config::{
    Config, ConfigBuilder, InputOutputModeType, SelectedLicenses, validate_jpeg_quality,
};
use anyhow::anyhow;
use arg_names::*;
use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches};
use sic_cli_ops::create_image_ops;
use sic_cli_ops::operations::OperationId;
use sic_io::decode::FrameIndex;
use sic_io::encode_settings::gif::RepeatAnimation;
use std::path::Path;
use std::str::FromStr;

macro_rules! define_arg_consts {
    ($mod:ident, { $($argdef:ident),+ $(,)? } ) => {
            pub mod $mod {
                $(
                    pub const $argdef: &str = stringify!($argdef);
                )+
            }
    };
}

define_arg_consts!(arg_names, {
    // organisational:
    ARG_LICENSE,
    ARG_DEP_LICENSES,

    // input and output images
    ARG_INPUT,
    ARG_INPUT_GLOB,
    ARG_OUTPUT,
    ARG_OUTPUT_GLOB,

    // config for glob/batch mode
    ARG_GLOB_NO_SKIP_UNSUPPORTED_EXTENSIONS,

    // set specific configurations for decoding
    ARG_SELECT_FRAME,

    // set specific configurations for encoding
    ARG_DISABLE_AUTOMATIC_COLOR_TYPE_ADJUSTMENT,
    ARG_FORCED_OUTPUT_FORMAT,
    ARG_JPEG_ENCODING_QUALITY,
    ARG_PNM_ENCODING_ASCII,
    ARG_GIF_REPEAT,

    // provide image operations using image script
    ARG_APPLY_OPERATIONS,
    ARG_OPERATIONS_SCRIPT,

    // group: image operations
    GROUP_IMAGE_OPERATIONS,
});

fn wrap_with(app: App<'static, 'static>) -> App<'static, 'static> {
    app.arg(
        Arg::with_name(OperationId::DrawText.as_str())
            .help("Operation: draw-text.")
            .long(OperationId::DrawText.as_str())
            .takes_value(true)
            .value_name(
                "<text> <coord(x, y)> <rgba(r,g,b,a)> <size(s)> <font(\"path/to/font.ttf\">)",
            )
            .number_of_values(5)
            .multiple(true),
    )
    .arg(
        Arg::with_name(OperationId::Threshold.as_str())
            .help("Operation: Threshold the input image using Otsu's method")
            .long(OperationId::Threshold.as_str())
            .multiple(true),
    )
}

pub fn create_app(
    version: &'static str,
    about: &'static str,
    help_ops: &'static str,
) -> App<'static, 'static> {
    wrap_with(App::new("imagineer")
        .version(version)
        .about(about)
        .after_help("For more information, visit: https://github.com/foresterre/imagineer")
        .author("Martijn Gribnau <garm@ilumeo.com>")

        // settings
        .global_setting(AppSettings::NextLineHelp)
        .global_setting(AppSettings::ColoredHelp)
        .global_setting(AppSettings::ColorAuto)
        .global_setting(AppSettings::DontCollapseArgsInUsage)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .max_term_width(120)

        // cli arguments

        // organisational:
        .arg(Arg::with_name(ARG_LICENSE)
            .long("license")
            .help("Displays the license of this piece of software (`imagineer`).")
            .takes_value(false)
            .conflicts_with_all(&[ARG_DEP_LICENSES, ARG_INPUT, ARG_OUTPUT, ARG_INPUT_GLOB, ARG_OUTPUT_GLOB]))
        .arg(Arg::with_name(ARG_DEP_LICENSES)
            .long("dep-licenses")
            .help("Displays the licenses of the dependencies on which this software relies.")
            .takes_value(false)
            .conflicts_with_all(&[ARG_LICENSE, ARG_INPUT, ARG_OUTPUT, ARG_INPUT_GLOB, ARG_OUTPUT_GLOB]))

        // io(input):
        .arg(Arg::with_name(ARG_INPUT)
            .long("input")
            .short("i")
            .value_name("INPUT_PATH")
            .takes_value(true)
            .help("Input image path. When using this option, input piped from stdin will be ignored. \
                      If using unexpanded globs as argument, use --glob-input instead.")
            .conflicts_with_all(&[ARG_LICENSE, ARG_DEP_LICENSES, ARG_INPUT_GLOB, ARG_OUTPUT_GLOB]))

        .arg(Arg::with_name(ARG_INPUT_GLOB)
            .long("glob-input")
            .takes_value(true)
            .value_name("GLOB_INPUT_PATTERN")
            .help("Input glob path which attempts to match all files matching the given glob pattern. Use with --glob-output. \
                Depending on your shell you may need to add explicit quotation marks around the argument.")
            .conflicts_with_all(&[ARG_LICENSE, ARG_DEP_LICENSES, ARG_INPUT, ARG_OUTPUT])
        )

        // io(output):
        .arg(Arg::with_name(ARG_OUTPUT)
            .long("output")
            .short("o")
            .value_name("OUTPUT_PATH")
            .takes_value(true)
            .help("Output image path. When using this option, output won't be piped to stdout.")
            .conflicts_with_all(&[ARG_LICENSE, ARG_DEP_LICENSES, ARG_INPUT_GLOB, ARG_OUTPUT_GLOB])
        )

        .arg(Arg::with_name(ARG_OUTPUT_GLOB)
            .long("glob-output")
            .value_name("GLOB_OUTPUT_ROOT_FOLDER")
            .takes_value(true)
            .help("This output should point to a folder in which the greatest root common directory of the glob input will be mirrored")
            .conflicts_with_all(&[ARG_LICENSE, ARG_DEP_LICENSES, ARG_INPUT, ARG_OUTPUT])
        )

        // config for glob/batch mode
        .arg(Arg::with_name(ARG_GLOB_NO_SKIP_UNSUPPORTED_EXTENSIONS)
            .long("no-skip-unsupported-extensions")
            .help("Files which don't have a known extension will not be skipped in glob mode")
            .long_help("Only has an effect when combined with --glob-input")
            .takes_value(false)
        )

        // config(in):
        .arg(Arg::with_name(ARG_SELECT_FRAME)
            .long("select-frame")
            .value_name("#FRAME")
            .help("Frame to be loaded as still image if the input image is an animated image.\
                      To pick the first and last frame respectively, you can provide 'first' and 'last' as arguments. \
                      Otherwise provide a single zero-indexed positive number which corresponds with the frame index. \
                      For example, to select the first frame, the argument would be '0', for the second '1', etc.")
            .takes_value(true))

        // config(out):
        .arg(Arg::with_name(ARG_DISABLE_AUTOMATIC_COLOR_TYPE_ADJUSTMENT)
            .long("disable-automatic-color-type-adjustment")
            .help("Some image output formats do not support the color type of the image buffer prior to encoding. \
                      By default the program tries to adjust the color type. If this flag is provided, \
                      the program will not try to adjust the color type."))

        .arg(Arg::with_name(ARG_FORCED_OUTPUT_FORMAT)
            .short("f")
            .long("output-format")
            .value_name("FORMAT")
            .help("Force the output image format to use FORMAT, regardless of the (if any) extension of the given output file path. \
                      Output formats (FORMAT values) supported: AVIF, BMP, Farbfeld, GIF, ICO, JPEG, PNG, PAM, PBM, PGM, PPM and TGA.")
            .takes_value(true))

        .arg(Arg::with_name(ARG_JPEG_ENCODING_QUALITY)
            .long("jpeg-encoding-quality")
            .help("Set the jpeg quality to QUALITY. Valid values are positive numbers from 1 up to and including 100. Will only be used when the output format is determined to be jpeg.")
            .value_name("QUALITY")
            .takes_value(true))

        .arg(Arg::with_name(ARG_PNM_ENCODING_ASCII)
            .long("pnm-encoding-ascii")
            .help("Use ascii based encoding when using a PNM image output format (pbm, pgm or ppm). Doesn't apply to 'pam' (PNM Arbitrary Map)."))

        .arg(Arg::with_name(ARG_GIF_REPEAT)
            .long("gif-repeat")
            .help("Repeat the frames of a (to be) gif encoded image `n` times, `infinite` times or `never`.")
            .value_name("REPETITIONS")
            .takes_value(true)
        )

        // image-operations(script):
        .arg(Arg::with_name(ARG_APPLY_OPERATIONS)
            .long("apply-operations")
            .short("x")
            .alias("A")
            .help(help_ops)
            .value_name("OPERATIONS")
            .takes_value(true)
            .conflicts_with(ARG_OPERATIONS_SCRIPT))

        .arg(Arg::with_name(ARG_OPERATIONS_SCRIPT)
            .long("operations-script")
            .help("Like '--apply-operations' but takes a file path where the file contains the script instead of taking it as value directly")
            .value_name("SCRIPT_FILE")
            .takes_value(true)
            .conflicts_with(ARG_APPLY_OPERATIONS))

        // image-operations(cli-arguments):
        .group(ArgGroup::with_name(GROUP_IMAGE_OPERATIONS)
            .args(OperationId::variants())
            .conflicts_with(ARG_APPLY_OPERATIONS)
            .multiple(true))
        .arg(Arg::with_name(OperationId::Blur.as_str())
            .help("Operation: perform a gaussian blur on the input image")
            .long(OperationId::Blur.as_str())
            .takes_value(true)
            .value_name("fp")
            .number_of_values(1)
            .multiple(true)
            .allow_hyphen_values(true))
        .arg(Arg::with_name(OperationId::Brighten.as_str())
            .help("Operation: increase or decrease the brightness of the input image")
            .long(OperationId::Brighten.as_str())
            .takes_value(true)
            .value_name("int")
            .number_of_values(1)
            .multiple(true)
            .allow_hyphen_values(true))
        .arg(Arg::with_name(OperationId::Contrast.as_str())
            .help("Operation: increase or decrease the contrast of the input image")
            .long(OperationId::Contrast.as_str())
            .takes_value(true)
            .value_name("fp")
            .number_of_values(1)
            .multiple(true)
            .allow_hyphen_values(true))
        .arg(Arg::with_name(OperationId::Crop.as_str())
            .help("Operation: crop the input image to a bounding rectangle ranging from top-left (lx, ly) to bottom-right (rx, ry) coordinates")
            .long(OperationId::Crop.as_str())
            .takes_value(true)
            .value_names(&["lx", "ly", "rx", "ry"])
            .number_of_values(4)
            .multiple(true))
        .arg(Arg::with_name(OperationId::Diff.as_str())
            .help("Operation: show ")
            .long(OperationId::Diff.as_str())
            .takes_value(true)
            .value_name("path to image")
            .number_of_values(1)
            .multiple(true))

        .arg(Arg::with_name(OperationId::Filter3x3.as_str())
            .help("Operation: apply a 3x3 convolution filter to the input image (matrix arguments should be given left-to-right, top-to-bottom)")
            .long(OperationId::Filter3x3.as_str())
            .takes_value(true)
            .value_names(&["fp", "fp", "fp", "fp", "fp", "fp", "fp", "fp", "fp"])
            .number_of_values(9)
            .multiple(true)
            .allow_hyphen_values(true))
        .arg(Arg::with_name(OperationId::FlipHorizontal.as_str())
            .help("Operation: flip the input image horizontally")
            .long(OperationId::FlipHorizontal.as_str())
            .multiple(true))
        .arg(Arg::with_name(OperationId::FlipVertical.as_str())
            .help("Operation: flip the input image vertically")
            .long(OperationId::FlipVertical.as_str())
            .multiple(true))
        .arg(Arg::with_name(OperationId::Grayscale.as_str())
            .help("Operation: discard the chrominance signal from the input image, so it becomes achromatic")
            .long_help("Note that (depending on the provided settings flags), the processed image may still be stored in a format which encodes its chrominance")
            .long(OperationId::Grayscale.as_str())
            .multiple(true))
        .arg(Arg::with_name(OperationId::HueRotate.as_str())
            .help("Operation: rotate the hue for each pixel of the input image by a provided degree")
            .long_help("Range is 0-360 degrees, any other value will be mapped to that range by rotation")
            .long(OperationId::HueRotate.as_str())
            .takes_value(true)
            .value_name("int")
            .number_of_values(1)
            .multiple(true)
             .allow_hyphen_values(true))
        .arg(
            Arg::with_name(OperationId::HorizontalGradient.as_str())
            .help("Operation: blend the image with a horizontal gradient from color1 to color2.")
            .long(OperationId::HorizontalGradient.as_str())
            .takes_value(true)
            .value_name(
                "<rgba(r,g,b,a)> <rgba(r,g,b,a)>",
            )
            .number_of_values(2)
            .multiple(true))
        .arg(Arg::with_name(OperationId::Invert.as_str())
            .help("Operation: invert the each pixel of the input image ")
            .long(OperationId::Invert.as_str())
            .multiple(true))
        .arg(Arg::with_name(OperationId::Overlay.as_str())
            .help("Operation: overlay an image loaded from the provided path argument, over the input image (at a certain position)")
            .long(OperationId::Overlay.as_str())
            .value_names(&["overlay image path", "x", "y"])
            .takes_value(true)
            .number_of_values(3)
            .multiple(true))
        .arg(Arg::with_name(OperationId::Resize.as_str())
            .help("Operation: resize the input image to x by y pixels")
            .long(OperationId::Resize.as_str())
            .takes_value(true)
            .value_names(&["x", "y"])
            .number_of_values(2)
            .multiple(true))
        .arg(Arg::with_name(OperationId::Rotate90.as_str())
            .help("Operation: rotate the input image by 90 degrees")
            .long(OperationId::Rotate90.as_str())
            .multiple(true))
        .arg(Arg::with_name(OperationId::Rotate180.as_str())
            .help("Operation: rotate the input image by 180 degrees")
            .long(OperationId::Rotate180.as_str())
            .multiple(true))
        .arg(Arg::with_name(OperationId::Rotate270.as_str())
            .help("Operation: rotate the input image by 270 degrees")
            .long(OperationId::Rotate270.as_str())
            .multiple(true))
        .arg(Arg::with_name(OperationId::Unsharpen.as_str())
            .help("Operation: sharpen an image by combining an unsharp (blurred) mask of the input image with the (original) input image, sharpening for pixels where the difference is bigger than the provided threshold")
            .long(OperationId::Unsharpen.as_str())
            .takes_value(true)
            .value_names(&["blur amount","threshold"])
            .number_of_values(2)
            .multiple(true)
            .allow_hyphen_values(true))
        .arg(Arg::with_name(OperationId::VerticalGradient.as_str())
            .help("Operation: blend the image with a vertical gradient from color1 to color2.")
            .long(OperationId::VerticalGradient.as_str())
            .takes_value(true)
            .value_name(
                "<rgba(r,g,b,a)> <rgba(r,g,b,a)>",
            )
            .number_of_values(2)
            .multiple(true))

        // image-operations(cli-arguments/modifiers):
        .arg(Arg::with_name(OperationId::PreserveAspectRatio.as_str())
            .help("Operation modifier for 'resize': preserve the aspect ratio of the original input image")
            .long(OperationId::PreserveAspectRatio.as_str())
            .takes_value(true)
            .value_name("bool")
            .number_of_values(1)
            .multiple(true)
            .possible_values(&["true", "false"])
        )

        .arg(Arg::with_name(OperationId::SamplingFilter.as_str())
            .help("Operation modifier for 'resize': resize the image using a specific sampling-filter")
            .long(OperationId::SamplingFilter.as_str())
            .takes_value(true)
            .value_name("sampling filter")
            .number_of_values(1)
            .multiple(true)
            .possible_values(&["catmullrom", "gaussian", "lanczos3", "nearest", "triangle"])
        ))
}

// Here any argument should not panic when invalid.
// Previously, it was allowed to panic within Config, but this is no longer the case.
pub fn build_app_config<'a>(matches: &'a ArgMatches) -> anyhow::Result<Config<'a>> {
    let mut builder = ConfigBuilder::new();

    // organisational/licenses:

    let show_license = if matches.is_present(ARG_LICENSE) {
        builder = builder.show_license_text_of(SelectedLicenses::ThisSoftware);

        Some(())
    } else if matches.is_present(ARG_DEP_LICENSES) {
        builder = builder.show_license_text_of(SelectedLicenses::Dependencies);

        Some(())
    } else {
        None
    };

    if show_license.is_some() {
        return Ok(builder.build());
    }

    builder = builder.mode(InputOutputModeType::from_arg_matches(matches));

    // config(in)/select-frame:
    if let Some(value) = matches.value_of(ARG_SELECT_FRAME) {
        let index = parse_frame_index(value)?;
        builder = builder.select_frame(Some(index));
    }

    // config(out)/disable-automatic-color-type-adjustment:
    if matches.is_present(ARG_DISABLE_AUTOMATIC_COLOR_TYPE_ADJUSTMENT) {
        builder = builder.disable_automatic_color_type_adjustment(true);
    }

    // config(out)/output-format:
    if let Some(format) = matches.value_of(ARG_FORCED_OUTPUT_FORMAT) {
        builder = builder.forced_output_format(format);
    }

    // config(out)/jpeg-encoding-quality:
    if let Some(value) = matches.value_of(ARG_JPEG_ENCODING_QUALITY) {
        let requested_jpeg_quality = u8::from_str(value)
            .map_err(|_| {
                anyhow!("JPEG Encoding quality should be a value between 1 and 100 (inclusive).")
            })
            .and_then(validate_jpeg_quality)?;
        builder = builder.jpeg_quality(requested_jpeg_quality);
    }

    // config(out)/pnm-encoding-type:
    if matches.is_present(ARG_PNM_ENCODING_ASCII) {
        builder = builder.pnm_format_type(true);
    }

    if let Some(value) = matches.value_of(ARG_GIF_REPEAT) {
        let repeat = RepeatAnimation::try_from_str(value)?;
        builder = builder.gif_repeat(repeat);
    }

    // image-operations:
    //
    // Image operations are a bit more involved.
    // Thanks to clap, we know either ARG_APPLY_OPERATIONS xor GROUP_IMAGE_OPERATIONS
    // will be the method of providing an image operations program.
    //
    // However with Arg::multiple(true) and Arg::number_of_values(n) we can set to allow multiple
    // operations, like: --blur --blur (multiple is ok), and --crop 0 0 1 1 (number of values = 4),
    // but Clap also allows: --crop 0 0 --crop 1 1 (multiple is ok and number of values = 4).
    //
    // We want to set multiple to true, because image operations can be repeated and can have different
    // effects than the first time.
    // But since the effects can be different, we need to know the order in which arguments are
    // provided. Luckily Clap does tell us the indices of values if we ask for them.
    // If we use --crop 0 0 --blur 1 --crop 1 1, the order of the operations would be undefined, not to
    // say perhaps feel not logical for a user. Therefor, we enforce left to right ordering of
    // operations and require all values to be provided at once, after the operation argument.
    //
    // There is an edge case which we can't (as far as I am aware) handle without looking within the
    // argv ourselves: --crop 0 0 1 1 --crop, is valid according to Clap. However, since we do not
    // receive the amount of times --crop was defined, but rather all the separate provided values for
    // the name of the argument, we just know that for `crop` we have values 0,0,1,1.
    let program = if let Some(script) = matches.value_of(ARG_APPLY_OPERATIONS) {
        sic_parser::parse_script(script)?
    } else if let Some(path) = matches.value_of(ARG_OPERATIONS_SCRIPT) {
        let contents = std::fs::read_to_string(Path::new(path))
            .map_err(|err| anyhow::anyhow!("unable to read script file: {}", err))?;
        sic_parser::parse_script(&contents)?
    } else {
        create_image_ops(std::env::args())?
    };

    builder = builder.image_operations_program(program);

    Ok(builder.build())
}

fn parse_frame_index(input: &str) -> anyhow::Result<FrameIndex> {
    match input {
        "first" => Ok(FrameIndex::First),
        "last" => Ok(FrameIndex::Last),
        n => {
            let pick = n.parse::<usize>().map_err(|_| {
                anyhow!(
                    "Provided argument for --select-frame is not a valid option. \
                         Valid options are 'first', 'last' or a (one-indexed) positive number."
                )
            })?;

            Ok(FrameIndex::Nth(pick))
        }
    }
}
