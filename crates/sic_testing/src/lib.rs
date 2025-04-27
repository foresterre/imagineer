#![deny(clippy::all)]

use std::path::{Path, PathBuf};

// re-export parameterized macro's
pub use parameterized::ide;
pub use parameterized::parameterized as pm;
use sic_core::SicImage;
use sic_core::image::GenericImageView;

// just enough, absolute tolerance, floating point comparison.
#[macro_export]
macro_rules! approx_eq_f32 {
    ($input:expr, $expected:expr) => {
        assert!(($input - $expected).abs() <= std::f32::EPSILON);
    };
}

#[macro_export]
macro_rules! out_ {
    ($path:expr) => {
        &[env!("CARGO_MANIFEST_DIR"), "/../../target/", $path].concat()
    };
}

#[macro_export]
macro_rules! in_ {
    ($path:expr) => {
        &[env!("CARGO_MANIFEST_DIR"), "/../../resources/", $path].concat()
    };
}

/// TODO{issue#128}: rework to provide flexibility and consistency, so all modules can use this;
pub fn setup_test_image(test_image_path: &str) -> PathBuf {
    Path::new("").join(in_!(test_image_path))
}

pub fn setup_output_path(test_output_path: &str) -> PathBuf {
    Path::new("").join(out_!(test_output_path))
}

pub fn clean_up_output_path(test_output_path: &str) {
    std::fs::remove_file(setup_output_path(test_output_path))
        .expect("Unable to remove file after test.");
}

pub fn open_test_image<P: AsRef<Path>>(path: P) -> sic_core::SicImage {
    sic_core::image::open(path.as_ref()).unwrap().into()
}

pub fn image_eq<T: Into<sic_core::SicImage>>(left: T, right: T) -> bool {
    let left = left.into();
    let right = right.into();

    left.dimensions() == right.dimensions()
        && left
            .pixels()
            .zip(right.pixels())
            .all(|(l, r)| l.0 == r.0 && l.1 == r.1 && l.2 == r.2)
}

// Adds direct access for static images.
pub trait SicImageDirectAccess {
    fn get_pixel<I: GenericImageView>(&self, x: u32, y: u32) -> I::Pixel
    where
        Self: AsRef<I>,
    {
        self.as_ref().get_pixel(x, y)
    }

    fn width<I: GenericImageView>(&self) -> u32
    where
        Self: AsRef<I>,
    {
        self.as_ref().width()
    }

    fn height<I: GenericImageView>(&self) -> u32
    where
        Self: AsRef<I>,
    {
        self.as_ref().height()
    }

    fn dimensions<I: GenericImageView>(&self) -> (u32, u32)
    where
        Self: AsRef<I>,
    {
        self.as_ref().dimensions()
    }

    fn pixels<I: GenericImageView>(&self) -> sic_core::image::Pixels<I>
    where
        Self: AsRef<I>,
    {
        self.as_ref().pixels()
    }
}

impl SicImageDirectAccess for SicImage {}
