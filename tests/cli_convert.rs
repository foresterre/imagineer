#![deny(clippy::all)]
#![allow(clippy::items_after_test_module)]

#[macro_use]
extern crate parameterized;

use std::io::Read;
use std::path::{Path, PathBuf};

use sic_core::image;

use imagineer::cli::app::{build_app_config, create_app as get_app}; // build_app_config
use imagineer::cli::config::InputOutputMode;
use imagineer::cli::pipeline::run_with_devices;

// copied from sic_lib::processor::mod_test_includes
// I preferred to not make that module public (2018-11-28)
// Originally named: setup_test_image.
fn setup_input_path(test_image_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join(test_image_path)
}

// copied from sic_lib::processor::mod_test_includes
// I preferred to not make that module public (2018-11-28)
fn setup_output_path(test_output_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(test_output_path)
}

fn path_buf_str(pb: &Path) -> &str {
    pb.to_str().unwrap()
}

// copied from sic_lib::processor::mod_test_includes
// I preferred to not make that module public (2018-11-28)
fn clean_up_output_path(test_output_path: &str) {
    std::fs::remove_file(setup_output_path(test_output_path))
        .expect("Unable to remove file after test.");
}

// by image header
fn is_image_format(output_path: &str, hope: image::ImageFormat) -> bool {
    let mut file = std::fs::File::open(setup_output_path(output_path))
        .expect("Failed to find (produced) test image.");

    let mut bytes = vec![];

    file.read_to_end(&mut bytes)
        .expect("Failed to read (produced) test image.");

    hope == image::guess_format(&bytes).expect("Format could not be guessed.")
}

// convert_to_X_by_extension
// BMP, GIF, JPEG, PNG, ICO, PBM, PGM, PPM, PAM

#[cfg(test)]
mod convert_to_x {
    use super::*;

    ide!();

    #[yare::parameterized(
        bmp = { "bmp", image::ImageFormat::Bmp },
        farbfeld = { "ff", image::ImageFormat::Farbfeld },
        gif = { "gif", image::ImageFormat::Gif },
        jpg = { "jpg", image::ImageFormat::Jpeg },
        jpeg = {"jpeg", image::ImageFormat::Jpeg },
        ico = { "ico", image::ImageFormat::Ico },
        png = { "png", image::ImageFormat::Png },
        pbm = { "pbm", image::ImageFormat::Pnm },
        pgm = { "pgm", image::ImageFormat::Pnm },
        ppm = { "ppm", image::ImageFormat::Pnm },
        pam = { "pam", image::ImageFormat::Pnm },
        qoi = { "qoi", image::ImageFormat::Qoi },
        tif = { "tif", image::ImageFormat::Tiff },
        tiff = { "tiff", image::ImageFormat::Tiff },
        webp = { "webp", image::ImageFormat::WebP },
    )]
    fn convert_to_x_by_extension(ext: &str, expected_format: image::ImageFormat) {
        let input_path = setup_input_path("palette_4x4.png");
        let output_path = setup_output_path(&["cli_convert_to_x_by_extension", ext].join("."));

        let args = vec![
            "imagineer",
            "--input",
            input_path.to_str().unwrap(),
            "--output",
            path_buf_str(&output_path),
        ];
        let matches = get_app("", "", "").get_matches_from(args);

        let complete = run_with_devices(
            InputOutputMode::try_from_matches(&matches).unwrap(),
            &build_app_config(&matches).unwrap(),
        );

        complete.unwrap();
        assert!(output_path.exists());
        assert!(is_image_format(path_buf_str(&output_path), expected_format));

        clean_up_output_path(path_buf_str(&output_path));
    }
}

#[cfg(test)]
mod convert_to_x_by_ff {
    use super::*;

    ide!();

    fn args<'a>(which: &'a str, input: &'a Path, output: &'a Path) -> Vec<&'a str> {
        vec![
            "imagineer",
            "--output-format",
            which,
            "--input",
            path_buf_str(input),
            "--output",
            path_buf_str(output),
        ]
    }

    #[yare::parameterized(
        bmp = { "bmp", image::ImageFormat::Bmp },
        farbfeld = { "ff", image::ImageFormat::Farbfeld },
        gif = { "gif", image::ImageFormat::Gif },
        jpg = { "jpg", image::ImageFormat::Jpeg },
        jpeg = {"jpeg", image::ImageFormat::Jpeg },
        ico = { "ico", image::ImageFormat::Ico },
        png = { "png", image::ImageFormat::Png },
        pbm = { "pbm", image::ImageFormat::Pnm },
        pgm = { "pgm", image::ImageFormat::Pnm },
        ppm = { "ppm", image::ImageFormat::Pnm },
        pam = { "pam", image::ImageFormat::Pnm },
        qoi = { "qoi", image::ImageFormat::Qoi },
        tif = { "tif", image::ImageFormat::Tiff },
        tiff = { "tiff", image::ImageFormat::Tiff },
        webp = { "webp", image::ImageFormat::WebP },
    )]
    fn convert_to_bmp_by_ff(which: &str, expected_format: image::ImageFormat) {
        let input_path = setup_input_path("palette_4x4.png");
        let output_path = setup_output_path(&format!("cli_convert_to_x_by_extension_ff_{}", which));

        let args = args(which, &input_path, &output_path);
        let matches = get_app("", "", "").get_matches_from(args);
        let complete = run_with_devices(
            InputOutputMode::try_from_matches(&matches).unwrap(),
            &build_app_config(&matches).unwrap(),
        );

        complete.unwrap();
        assert!(output_path.exists());
        assert!(is_image_format(path_buf_str(&output_path), expected_format));

        clean_up_output_path(path_buf_str(&output_path));
    }
}

fn read_file_to_bytes<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut f = std::fs::File::open(path).unwrap();
    let mut buffer = Vec::new();

    // read the whole file
    f.read_to_end(&mut buffer).unwrap();

    buffer
}

// Try to determine that PBM, PGM, PPM in ascii mode (P1, P2, P3 resp.) are ascii encoded
// and if they are 'binary' encoded (P4, P5, P6), they are obviously not ascii encoded.
#[cfg(test)]
mod pnm_ascii_and_binary {
    use super::*;

    ide!();

    fn guess_is_ascii_encoded(input: &[u8]) -> bool {
        // The character P in ascii encoding
        let is_ascii_p = |c: u8| c == 0x50;

        // P1, P2, P3 are ascii
        // Checks for numbers 1, 2, 3, binary known as: 0x31, 0x32, 0x33 respectively
        let is_ascii_magic_number = |c| c == 0x31 || c == 0x32 || c == 0x33;

        let mut iter = input.iter();
        let first = iter.next().unwrap();
        let second = iter.next().unwrap();

        // check also to be sure that every character in the file can be ascii encoded
        is_ascii_p(*first) && is_ascii_magic_number(*second) && iter.all(|c| *c <= 0x7F)
    }

    fn pnm_encode_test_case(which: &str, is_ascii: bool) {
        let input_path = setup_input_path("palette_4x4.png");
        let ascii_str = |bool: bool| if bool { "ascii" } else { "binary" };
        let output_path = setup_output_path(&format!(
            "cli_convert_pnm_encoding_test_{}_{}",
            which,
            ascii_str(is_ascii)
        ));

        let mut args = Vec::with_capacity(8);
        args.push("imagineer");
        if is_ascii {
            args.push("--pnm-encoding-ascii");
        }
        args.push("--output-format");
        args.push(which);
        args.push("--input");
        args.push(path_buf_str(&input_path));
        args.push("--output");
        args.push(path_buf_str(&output_path));

        let matches = get_app("", "", "").get_matches_from(args);
        let complete = run_with_devices(
            InputOutputMode::try_from_matches(&matches).unwrap(),
            &build_app_config(&matches).unwrap(),
        );

        complete.unwrap();
        assert!(output_path.exists());

        // read file contents
        let contents = read_file_to_bytes(path_buf_str(&output_path));

        // is it just ascii?
        if is_ascii {
            assert!(guess_is_ascii_encoded(&contents));
        } else {
            assert!(!guess_is_ascii_encoded(&contents));
        }

        clean_up_output_path(path_buf_str(&output_path));
    }

    #[yare::parameterized(
        pbm = { "pbm" },
        pgm = { "pgm" },
        ppm = { "ppm" },
    )]
    fn pnm_encode_ascii(which: &str) {
        pnm_encode_test_case(which, true)
    }

    #[yare::parameterized(
        pbm = { "pbm" },
        pgm = { "pgm" },
        ppm = { "ppm" },
    )]
    fn pnm_encode_binary(which: &str) {
        pnm_encode_test_case(which, false)
    }
}

// JPEG different quality
// Currently just tested by default 80 ?!= not(80)
// Can we do better?
#[test]
fn convert_jpeg_quality_different() {
    let which = "jpeg";

    let our_input = setup_input_path("palette_4x4.png");
    let out1 = setup_output_path("out_02_jpeg_1.jpeg");
    let out2 = setup_output_path("out_02_jpeg_2.jpeg");

    let args1 = vec![
        "imagineer",
        "--output-format",
        which,
        "--input",
        path_buf_str(&our_input),
        "--output",
        path_buf_str(&out1),
    ];

    let args2 = vec![
        "imagineer",
        "--jpeg-encoding-quality",
        "81",
        "--output-format",
        which,
        "--input",
        path_buf_str(&our_input),
        "--output",
        path_buf_str(&out2),
    ];

    let matches1 = get_app("", "", "").get_matches_from(args1);
    run_with_devices(
        InputOutputMode::try_from_matches(&matches1).unwrap(),
        &build_app_config(&matches1).unwrap(),
    )
    .unwrap();

    let matches2 = get_app("", "", "").get_matches_from(args2);
    run_with_devices(
        InputOutputMode::try_from_matches(&matches2).unwrap(),
        &build_app_config(&matches2).unwrap(),
    )
    .unwrap();

    assert!(out1.exists() && out2.exists());

    // read file contents
    let contents1 = read_file_to_bytes(path_buf_str(&out1));
    let contents2 = read_file_to_bytes(path_buf_str(&out2));

    assert_ne!(contents1, contents2);

    clean_up_output_path(path_buf_str(&out1));
    clean_up_output_path(path_buf_str(&out2));
}
