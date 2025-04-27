use std::collections::HashMap;
use std::hash::Hash;

use sic_core::image::imageops::FilterType;

use crate::errors::SicImageEngineError;
use crate::operations::ImageOperation;
use crate::wrapper::filter_type::FilterTypeWrap;
use crate::{ImgOp, operations};
use sic_core::SicImage;

trait EnvironmentKey {
    fn key(&self) -> ItemName;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(name(ItemName), derive(Display, Hash))]
pub enum EnvItem {
    CustomSamplingFilter(FilterTypeWrap),
    PreserveAspectRatio(bool),
}

impl EnvItem {
    pub fn resize_sampling_filter(self) -> Option<FilterTypeWrap> {
        match self {
            EnvItem::CustomSamplingFilter(k) => Some(k),
            _ => None,
        }
    }

    pub fn preserve_aspect_ratio(self) -> Option<bool> {
        match self {
            EnvItem::PreserveAspectRatio(k) => Some(k),
            _ => None,
        }
    }
}

impl EnvironmentKey for EnvItem {
    fn key(&self) -> ItemName {
        match self {
            EnvItem::CustomSamplingFilter(_) => ItemName::CustomSamplingFilter,
            EnvItem::PreserveAspectRatio(_) => ItemName::PreserveAspectRatio,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Env {
    store: HashMap<ItemName, EnvItem>,
}

impl Env {
    pub fn insert_or_update(&mut self, item: EnvItem) {
        let key = item.key();

        *self.store.entry(key).or_insert(item) = item;
    }

    pub fn remove(&mut self, key: ItemName) -> Option<()> {
        self.store.remove(&key).map(|_| ())
    }

    pub fn get(&self, key: ItemName) -> Option<&EnvItem> {
        self.store.get(&key)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instr {
    Operation(ImgOp),
    EnvAdd(EnvItem),
    EnvRemove(ItemName),
}

#[derive(Clone)]
pub struct ImageEngine {
    environment: Box<Env>,
    image: Box<SicImage>,
}

impl ImageEngine {
    pub fn new(image: SicImage) -> Self {
        Self {
            environment: Box::from(Env::default()),
            image: Box::from(image),
        }
    }

    pub fn ignite(mut self, instructions: &[Instr]) -> Result<SicImage, SicImageEngineError> {
        for instruction in instructions {
            match self.process_instruction(instruction) {
                Ok(_) => continue,
                Err(err) => return Err(err),
            }
        }

        Ok(*self.image)
    }

    fn process_instruction(&mut self, instruction: &Instr) -> Result<(), SicImageEngineError> {
        match instruction {
            Instr::Operation(op) => self.process_operation(op),
            Instr::EnvAdd(item) => self.insert_env(*item),
            Instr::EnvRemove(key) => self.remove_env(*key),
        }
    }

    fn process_operation(&mut self, operation: &ImgOp) -> Result<(), SicImageEngineError> {
        match operation {
            ImgOp::Blur(sigma) => {
                operations::blur::Blur::new(*sigma).apply_operation(&mut self.image)
            }
            ImgOp::Brighten(amount) => {
                operations::brighten::Brighten::new(*amount).apply_operation(&mut self.image)
            }
            ImgOp::Contrast(f) => {
                operations::contrast::Contrast::new(*f).apply_operation(&mut self.image)
            }
            ImgOp::Crop((lx, ly, rx, ry)) => {
                operations::crop::Crop::new((*lx, *ly), (*rx, *ry)).apply_operation(&mut self.image)
            }
            ImgOp::Diff(path) => operations::diff::Diff::new(path).apply_operation(&mut self.image),
            ImgOp::DrawText(inner) => {
                operations::draw_text::DrawText::new(inner).apply_operation(&mut self.image)
            }
            ImgOp::Filter3x3(kernel) => {
                operations::filter3x3::Filter3x3::new(kernel).apply_operation(&mut self.image)
            }
            ImgOp::FlipHorizontal => {
                operations::flip_horizontal::FlipHorizontal::new().apply_operation(&mut self.image)
            }
            ImgOp::FlipVertical => {
                operations::flip_vertical::FlipVertical::new().apply_operation(&mut self.image)
            }
            ImgOp::Grayscale => {
                operations::grayscale::Grayscale::new().apply_operation(&mut self.image)
            }
            ImgOp::HueRotate(degree) => {
                operations::hue_rotate::HueRotate::new(*degree).apply_operation(&mut self.image)
            }
            ImgOp::HorizontalGradient(colors) => {
                operations::horizontal_gradient::HorizontalGradient::new(*colors)
                    .apply_operation(&mut self.image)
            }
            ImgOp::Invert => operations::invert::Invert::new().apply_operation(&mut self.image),
            ImgOp::Overlay(inputs) => {
                operations::overlay::Overlay::new(inputs).apply_operation(&mut self.image)
            }
            ImgOp::Resize((x, y)) => {
                let aspect_ratio = should_preserve_aspect_ratio(&self.environment);
                let sampling_filter = resize_filter_or_default(&self.environment);
                let op = operations::resize::Resize::new(*x, *y, aspect_ratio, sampling_filter);
                op.apply_operation(&mut self.image)
            }
            ImgOp::Rotate90 => {
                operations::rotate90::Rotate90::new().apply_operation(&mut self.image)
            }
            ImgOp::Rotate180 => {
                operations::rotate180::Rotate180::new().apply_operation(&mut self.image)
            }
            ImgOp::Rotate270 => {
                operations::rotate270::Rotate270::new().apply_operation(&mut self.image)
            }
            ImgOp::Threshold => {
                operations::threshold::Threshold::new().apply_operation(&mut self.image)
            }
            ImgOp::Unsharpen((sigma, threshold)) => {
                operations::unsharpen::Unsharpen::new(*sigma, *threshold)
                    .apply_operation(&mut self.image)
            }
            ImgOp::VerticalGradient(colors) => {
                operations::vertical_gradient::VerticalGradient::new(*colors)
                    .apply_operation(&mut self.image)
            }
        }
    }

    fn insert_env(&mut self, item: EnvItem) -> Result<(), SicImageEngineError> {
        self.environment.insert_or_update(item);

        Ok(())
    }

    fn remove_env(&mut self, key: ItemName) -> Result<(), SicImageEngineError> {
        let success = self.environment.remove(key);

        if success.is_none() {
            eprintln!(
                "Warning: tried to de-register: {:?}, but wasn't registered.",
                key
            );
        }

        Ok(())
    }
}

fn resize_filter_or_default(env: &Env) -> FilterType {
    env.get(ItemName::CustomSamplingFilter)
        .and_then(|item| item.resize_sampling_filter())
        .map(FilterType::from)
        .unwrap_or_else(|| FilterTypeWrap::default().into())
}

fn should_preserve_aspect_ratio(env: &Env) -> bool {
    env.get(ItemName::PreserveAspectRatio)
        .and_then(|item| item.preserve_aspect_ratio())
        .unwrap_or_default()
}

#[cfg(test)]
mod compatibility {
    use sic_core::SicImage;

    // The raw_pixels() method was removed from the image crate in version 0.23
    // We replace it for our test cases with this straightforward trait, and trait impl for
    // DynamicImage.
    pub(crate) trait RawPixels {
        fn raw_pixels(&self) -> Vec<u8>;
    }

    impl RawPixels for SicImage {
        fn raw_pixels(&self) -> Vec<u8> {
            match self.as_ref() {
                sic_core::image::DynamicImage::ImageLuma8(buffer) => buffer.to_vec(),
                sic_core::image::DynamicImage::ImageLumaA8(buffer) => buffer.to_vec(),
                sic_core::image::DynamicImage::ImageRgb8(buffer) => buffer.to_vec(),
                sic_core::image::DynamicImage::ImageRgba8(buffer) => buffer.to_vec(),
                _ => unimplemented!(),
            }
        }
    }
}

#[cfg(test)]
mod environment_tests {
    use super::*;

    #[test]
    fn environment_insert() {
        let mut env = Env::default();
        assert!(!env.store.contains_key(&ItemName::CustomSamplingFilter));

        env.insert_or_update(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
            FilterType::Triangle,
        )));

        assert!(env.store.contains_key(&ItemName::CustomSamplingFilter));
    }

    #[test]
    fn environment_update() {
        let mut env = Env::default();

        env.insert_or_update(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
            FilterType::Triangle,
        )));

        assert!(env.store.contains_key(&ItemName::CustomSamplingFilter));
        assert_eq!(
            EnvItem::CustomSamplingFilter(FilterTypeWrap::new(FilterType::Triangle)),
            *env.get(ItemName::CustomSamplingFilter).unwrap()
        );

        env.insert_or_update(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
            FilterType::Gaussian,
        )));

        assert!(env.store.contains_key(&ItemName::CustomSamplingFilter));
        assert_eq!(
            EnvItem::CustomSamplingFilter(FilterTypeWrap::new(FilterType::Gaussian)),
            *env.get(ItemName::CustomSamplingFilter).unwrap()
        );
    }

    #[test]
    fn environment_remove() {
        let mut env = Env::default();

        env.insert_or_update(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
            FilterType::Triangle,
        )));

        assert!(env.store.contains_key(&ItemName::CustomSamplingFilter));
        assert_eq!(
            EnvItem::CustomSamplingFilter(FilterTypeWrap::new(FilterType::Triangle)),
            *env.get(ItemName::CustomSamplingFilter).unwrap()
        );

        let removed = env.remove(ItemName::CustomSamplingFilter);

        assert!(removed.is_some());
        assert!(!env.store.contains_key(&ItemName::CustomSamplingFilter));
    }

    #[test]
    fn environment_remove_not_existing() {
        let mut env = Env::default();

        assert!(!env.store.contains_key(&ItemName::CustomSamplingFilter));

        let removed = env.remove(ItemName::CustomSamplingFilter);

        assert!(removed.is_none());
        assert!(!env.store.contains_key(&ItemName::CustomSamplingFilter));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::compatibility::*;
    use crate::operations::diff::{DIFF_PX_DIFF, DIFF_PX_NO_OVERLAP, DIFF_PX_SAME};
    use crate::wrapper::gradient_input::GradientInput;
    use crate::wrapper::image_path::ImageFromPath;
    use sic_core::image::Rgba;
    use sic_core::image::imageops::FilterType;
    use sic_testing::*;
    use std::path::PathBuf;

    // output images during tests to verify the results visually
    fn output_test_image_for_manual_inspection(img: &SicImage, path: &str) {
        if cfg!(feature = "output-test-images") {
            let _ = img.as_ref().save(path);
        }
    }

    fn setup_default_test_image() -> SicImage {
        const DEFAULT_TEST_IMAGE_PATH: &str = "unsplash_763569_cropped.jpg";
        sic_testing::open_test_image(sic_testing::in_!(DEFAULT_TEST_IMAGE_PATH))
    }

    #[test]
    fn diff_check_out_pixels() {
        const LEFT: &str = "2x3_wrabaa.png";
        let left = sic_testing::open_test_image(sic_testing::in_!(LEFT));
        const RIGHT: &str = "3x2_wbaaba.png";

        let engine = ImageEngine::new(left);
        let out = engine.ignite(&[Instr::Operation(ImgOp::Diff(ImageFromPath::new(
            PathBuf::from(in_!(RIGHT)),
        )))]);

        let out = out.unwrap();

        assert_eq!(out.width(), 3);
        assert_eq!(out.height(), 3);

        assert_eq!(out.get_pixel(0, 0), DIFF_PX_SAME);
        assert_eq!(out.get_pixel(1, 0), DIFF_PX_DIFF);
        assert_eq!(out.get_pixel(2, 0), DIFF_PX_DIFF);
        assert_eq!(out.get_pixel(0, 1), DIFF_PX_SAME);
        assert_eq!(out.get_pixel(1, 1), DIFF_PX_SAME);
        assert_eq!(out.get_pixel(2, 1), DIFF_PX_DIFF);
        assert_eq!(out.get_pixel(0, 2), DIFF_PX_DIFF);
        assert_eq!(out.get_pixel(1, 2), DIFF_PX_DIFF);
        assert_eq!(out.get_pixel(2, 2), DIFF_PX_NO_OVERLAP);

        output_test_image_for_manual_inspection(&out, out_!("test_diff_3x3.png"));
    }

    mod sizes {
        use super::*;

        ide!();

        #[pm(
            left = {
                "1x1_a.png",
                "2x2_baab.png",
                "2x3_yrgyyb.bmp",
                "2x2_baab.png",
            },
            right = {
                "1x1_b.png",
                "1x1_b.png",
                "3x2_ygyryb.bmp",
                "2x3_rrgrbb.bmp",
            },
            expected_width = {
                1,
                2,
                3,
                2,
            },
            expected_height = {
                1,
                2,
                3,
                3,
            },
        )]
        fn diff_has_expected_width_and_height(
            left: &str,
            right: &str,
            expected_width: u32,
            expected_height: u32,
        ) {
            let left_img = sic_testing::open_test_image(sic_testing::in_!(left));

            let engine = ImageEngine::new(left_img);
            let out = engine.ignite(&[Instr::Operation(ImgOp::Diff(ImageFromPath::new(
                PathBuf::from(in_!(right)),
            )))]);

            let out = out.unwrap();

            assert_eq!(out.width(), expected_width);
            assert_eq!(out.height(), expected_height);

            let name = format!("test_diff_{},{}.png", left, right);
            output_test_image_for_manual_inspection(&out, out_!(&name));
        }
    }

    #[test]
    fn resize_with_preserve_aspect_ratio() {
        // W 217 H 447
        let img = setup_default_test_image();

        let engine = ImageEngine::new(img);
        let engine2 = engine.clone();
        let cmp_left = engine.ignite(&[
            Instr::EnvAdd(EnvItem::PreserveAspectRatio(true)),
            Instr::Operation(ImgOp::Resize((100, 100))),
        ]);

        assert!(cmp_left.is_ok());

        let cmp_right = engine2.ignite(&[Instr::Operation(ImgOp::Resize((100, 100)))]);

        assert!(cmp_left.is_ok());

        let left = cmp_left.unwrap();
        let right = cmp_right.unwrap();

        assert_ne!(left.raw_pixels(), right.raw_pixels());

        // 447 > 217 ->  (i64) |_ 217px/447px*100px _| = 49px
        assert_eq!((49, 100), left.dimensions());

        output_test_image_for_manual_inspection(
            &left,
            out_!("test_resize_preserve_aspect_ratio_left_preserve.png"),
        );

        output_test_image_for_manual_inspection(
            &right,
            out_!("test_resize_preserve_aspect_ratio_right_default.png"),
        );
    }

    #[test]
    fn resize_with_preserve_aspect_ratio_set_to_false() {
        // W 217 H 447
        let img = setup_default_test_image();

        let engine = ImageEngine::new(img);
        let engine2 = engine.clone();
        let cmp_left = engine.ignite(&[
            Instr::EnvAdd(EnvItem::PreserveAspectRatio(false)),
            Instr::Operation(ImgOp::Resize((100, 100))),
        ]);

        assert!(cmp_left.is_ok());

        let cmp_right = engine2.ignite(&[Instr::Operation(ImgOp::Resize((100, 100)))]);

        assert!(cmp_left.is_ok());

        let left = cmp_left.unwrap();
        let right = cmp_right.unwrap();

        assert_eq!(left.raw_pixels(), right.raw_pixels());

        assert_eq!((100, 100), left.dimensions());

        output_test_image_for_manual_inspection(
            &left,
            out_!("test_resize_preserve_aspect_ratio_left_preserve_f.png"),
        );

        output_test_image_for_manual_inspection(
            &right,
            out_!("test_resize_preserve_aspect_ratio_right_default_f.png"),
        );
    }

    #[test]
    fn resize_with_sampling_filter_nearest() {
        let img = setup_default_test_image();

        let engine = ImageEngine::new(img);
        let engine2 = engine.clone();
        let cmp_left = engine.ignite(&[
            Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
                FilterType::Nearest,
            ))),
            Instr::Operation(ImgOp::Resize((100, 100))),
        ]);

        assert!(cmp_left.is_ok());

        let cmp_right = engine2.ignite(&[Instr::Operation(ImgOp::Resize((100, 100)))]);

        assert!(cmp_left.is_ok());

        let left = cmp_left.unwrap();
        let right = cmp_right.unwrap();

        assert_ne!(left.raw_pixels(), right.raw_pixels());

        output_test_image_for_manual_inspection(
            &left,
            out_!("test_resize_sampling_filter_left_nearest.png"),
        );

        output_test_image_for_manual_inspection(
            &right,
            out_!("test_resize_sampling_filter_right_default_gaussian.png"),
        );
    }

    #[test]
    fn register_unregister_sampling_filter() {
        let img = setup_default_test_image();

        let engine = ImageEngine::new(img);
        let engine2 = engine.clone();

        let cmp_left = engine.ignite(&[
            Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::new(
                FilterType::Nearest,
            ))),
            Instr::EnvRemove(ItemName::CustomSamplingFilter),
            Instr::Operation(ImgOp::Resize((100, 100))),
        ]);

        assert!(cmp_left.is_ok());

        let cmp_right = engine2.ignite(&[Instr::Operation(ImgOp::Resize((100, 100)))]);

        assert!(cmp_left.is_ok());

        let left = cmp_left.unwrap();
        let right = cmp_right.unwrap();

        assert_eq!(left.raw_pixels(), right.raw_pixels());

        output_test_image_for_manual_inspection(
            &left,
            out_!("test_register_unregister_sampling_filter_left.png"),
        );

        output_test_image_for_manual_inspection(
            &right,
            out_!("test_register_unregister_sampling_filter_right.png"),
        );
    }

    #[test]
    fn horizontal_gradient_test() {
        let img =
            sic_core::image::DynamicImage::ImageRgb8(sic_core::image::RgbImage::new(200, 200))
                .into();

        let operation = ImgOp::HorizontalGradient(GradientInput::new((
            Rgba([255, 150, 150, 255]),
            Rgba([255, 50, 50, 50]),
        )));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);
        assert!(done.is_ok());

        output_test_image_for_manual_inspection(
            &done.unwrap(),
            out_!("horizontal-gradient-test.png"),
        );
    }

    #[test]
    fn vertical_gradient_test() {
        let img =
            sic_core::image::DynamicImage::ImageRgb8(sic_core::image::RgbImage::new(200, 200))
                .into();

        let operation = ImgOp::VerticalGradient(GradientInput::new((
            Rgba([255, 150, 150, 255]),
            Rgba([255, 50, 50, 50]),
        )));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);
        assert!(done.is_ok());

        output_test_image_for_manual_inspection(
            &done.unwrap(),
            out_!("vertical-gradient-test.png"),
        );
    }

    #[test]
    fn test_blur() {
        let img = setup_default_test_image();
        let operation = ImgOp::Blur(10.0);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        output_test_image_for_manual_inspection(&done.unwrap(), out_!("test_blur.png"));
    }

    #[test]
    fn test_brighten_pos() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::Brighten(25);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_brighten_pos_25.png"));
    }

    #[test]
    fn test_brighten_zero() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();
        let operation = ImgOp::Brighten(0);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_eq!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_brighten_zero.png"));
    }

    #[test]
    fn test_brighten_neg() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::Brighten(-25);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_brighten_neg_25.png"));
    }

    #[test]
    fn test_contrast_pos() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::Contrast(150.9);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_contrast_pos_15_9.png"));
    }

    #[test]
    fn test_contrast_neg() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::Contrast(-150.9);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_contrast_pos_15_9.png"));
    }

    #[test]
    fn test_crop_ok_no_change() {
        let img = open_test_image(in_!("blackwhite_2x2.bmp"));
        let cmp = open_test_image(in_!("blackwhite_2x2.bmp"));

        let operation = ImgOp::Crop((0, 0, 2, 2));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_eq!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_crop_no_change.bmp"));
    }

    #[test]
    fn test_crop_ok_to_one_pixel() {
        let img = open_test_image(in_!("blackwhite_2x2.bmp"));
        let cmp = open_test_image(in_!("blackwhite_2x2.bmp"));

        let operation = ImgOp::Crop((0, 0, 1, 1));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        let result_dim = result_img.dimensions();
        assert_eq!(1, result_dim.0);
        assert_eq!(1, result_dim.1);

        assert_eq!(Rgba([0, 0, 0, 255]), result_img.get_pixel(0, 0));

        output_test_image_for_manual_inspection(
            &result_img,
            out_!("test_crop_ok_to_one_pixel.bmp"),
        );
    }

    #[test]
    fn test_crop_ok_to_half_horizontal() {
        let img = open_test_image(in_!("blackwhite_2x2.bmp"));
        let cmp = open_test_image(in_!("blackwhite_2x2.bmp"));

        let operation = ImgOp::Crop((0, 0, 2, 1));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        let result_dim = result_img.dimensions();
        assert_eq!(2, result_dim.0);
        assert_eq!(1, result_dim.1);

        assert_eq!(Rgba([0, 0, 0, 255]), result_img.get_pixel(0, 0));
        assert_eq!(Rgba([255, 255, 255, 255]), result_img.get_pixel(1, 0));

        output_test_image_for_manual_inspection(
            &result_img,
            out_!("test_crop_ok_to_half_horizontal.bmp"),
        );
    }

    #[test]
    fn test_crop_err_lx_larger_than_rx() {
        let img = open_test_image(in_!("blackwhite_2x2.bmp"));

        // not rx >= lx
        let operation = ImgOp::Crop((1, 0, 0, 0));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_ly_larger_than_ry() {
        let img = open_test_image(in_!("blackwhite_2x2.bmp"));

        // not rx >= lx
        let operation = ImgOp::Crop((0, 1, 0, 0));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_out_of_image_bounds_top_lx() {
        let img = open_test_image(in_!("blackwhite_2x2.bmp"));

        let operation = ImgOp::Crop((3, 0, 1, 1));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_out_of_image_bounds_top_ly() {
        let img = open_test_image(in_!("blackwhite_2x2.bmp"));

        let operation = ImgOp::Crop((0, 3, 1, 1));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_out_of_image_bounds_top_rx() {
        let img = open_test_image(in_!("blackwhite_2x2.bmp"));

        let operation = ImgOp::Crop((0, 0, 3, 1));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_out_of_image_bounds_top_ry() {
        let img = open_test_image(in_!("blackwhite_2x2.bmp"));

        let operation = ImgOp::Crop((0, 0, 1, 3));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_err());
    }

    #[test]
    fn test_filter3x3() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::Filter3x3([1.0, 0.5, 0.0, 1.0, 0.5, 0.0, 1.0, 0.5, 0.0]);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_filter3x3.png"))
    }

    #[test]
    fn test_flip_h() {
        let img = setup_default_test_image();
        let operation = ImgOp::FlipHorizontal;

        let (xa, ya) = img.dimensions();
        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xa, xb);
        assert_eq!(ya, yb);

        output_test_image_for_manual_inspection(&img_result, out_!("test_fliph.png"));
    }

    #[test]
    fn test_flip_v() {
        let img = setup_default_test_image();
        let operation = ImgOp::FlipVertical;

        let (xa, ya) = img.dimensions();
        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xa, xb);
        assert_eq!(ya, yb);

        output_test_image_for_manual_inspection(&img_result, out_!("test_flipv.png"));
    }

    #[test]
    fn test_gray_scale() {
        use sic_core::image::Pixel;

        let img = open_test_image(in_!("rainbow_8x6.bmp"));
        let operation = ImgOp::Grayscale;

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();

        // The color type isn't actually changed to luma, so instead of checking color type,
        // here pixels are checked to have equal (r, g, b) components.
        for i in 0..8 {
            for j in 0..6 {
                let pixel = img_result.get_pixel(i, j);
                let channels_result = pixel.channels();
                let r_component = channels_result[0];
                let g_component = channels_result[1];
                let b_component = channels_result[2];

                assert_eq!(r_component, g_component);
                assert_eq!(g_component, b_component);
            }
        }

        output_test_image_for_manual_inspection(&img_result, out_!("test_gray_scale.png"));
    }

    #[test]
    fn test_hue_rotate_neg() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::HueRotate(-100);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_hue_rot_neg_100.png"));
    }

    #[test]
    fn test_hue_rotate_pos() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::HueRotate(100);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_hue_rot_pos_100.png"));
    }

    #[test]
    fn test_hue_rotate_zero() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::HueRotate(0);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_eq!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_hue_rot_0.png"));
    }

    #[test]
    fn test_hue_rotate_360() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::HueRotate(360);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        // https://docs.rs/image/0.19.0/image/enum.DynamicImage.html#method.huerotate
        // huerotate(0) should be huerotate(360), but this doesn't seem the case
        let expected = SicImage::from(cmp.as_ref().huerotate(360));
        assert_eq!(expected.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_hue_rot_pos_360.png"));
    }

    #[test]
    fn test_hue_rotate_over_rotate_pos() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::HueRotate(460);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        let expected = SicImage::from(cmp.as_ref().huerotate(100));
        assert_ne!(expected.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_hue_rot_pos_460.png"));
    }

    #[test]
    fn test_invert() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::Invert;

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_invert.png"));
    }

    mod overlay {
        use super::*;
        use crate::wrapper::overlay::OverlayInputs;

        #[test]
        fn overlay_with_self_at_origin() {
            let img = setup_default_test_image();
            let overlay = sic_testing::in_!("unsplash_763569_cropped.jpg");

            let engine = ImageEngine::new(img.clone());
            let res = engine.ignite(&[Instr::Operation(ImgOp::Overlay(OverlayInputs::new(
                ImageFromPath::new(overlay.into()),
                (0, 0),
            )))]);

            let res_image = res.unwrap();
            assert_eq!(img.raw_pixels(), res_image.raw_pixels());

            output_test_image_for_manual_inspection(
                &res_image,
                out_!("test_overlay_self_origin.png"),
            );
        }

        #[test]
        fn overlay_with_self_outside_bounds() {
            let img = setup_default_test_image();
            let bounds = img.dimensions();

            let overlay = sic_testing::in_!("unsplash_763569_cropped.jpg");

            let engine = ImageEngine::new(img.clone());
            let res = engine.ignite(&[Instr::Operation(ImgOp::Overlay(OverlayInputs::new(
                ImageFromPath::new(overlay.into()),
                (bounds.0 as i64, bounds.1 as i64),
            )))]);

            let res_image = res.unwrap();
            assert_eq!(img.raw_pixels(), res_image.raw_pixels());

            output_test_image_for_manual_inspection(
                &res_image,
                out_!("test_overlay_self_bounds.png"),
            );
        }

        #[test]
        fn overlay_with_self_se_quarter() {
            let img = setup_default_test_image();
            let bounds = img.dimensions();

            let overlay = sic_testing::in_!("unsplash_763569_cropped.jpg");

            let engine = ImageEngine::new(img.clone());
            let res = engine.ignite(&[
                Instr::Operation(ImgOp::Invert),
                Instr::Operation(ImgOp::Overlay(OverlayInputs::new(
                    ImageFromPath::new(overlay.into()),
                    ((bounds.0 / 2) as i64, (bounds.1 / 2) as i64),
                ))),
            ]);

            let res_image = res.unwrap();
            assert_ne!(img.raw_pixels(), res_image.raw_pixels());

            output_test_image_for_manual_inspection(
                &res_image,
                out_!("test_overlay_self_se_quarter.png"),
            );
        }
    }

    #[test]
    fn test_resize_down_gaussian() {
        // 217x447px => 100x200
        let img = setup_default_test_image();
        let operation = ImgOp::Resize((100, 200));

        let (xa, ya) = img.dimensions();

        assert_eq!(xa, 217);
        assert_eq!(ya, 447);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xb, 100);
        assert_eq!(yb, 200);

        output_test_image_for_manual_inspection(&img_result, out_!("test_scale_100x200.png"));
    }

    #[test]
    fn test_resize_up_gaussian() {
        // 217x447px => 300x500
        let img = setup_default_test_image();
        let operation = ImgOp::Resize((250, 500));

        let (xa, ya) = img.dimensions();

        assert_eq!(xa, 217);
        assert_eq!(ya, 447);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xb, 250);
        assert_eq!(yb, 500);

        output_test_image_for_manual_inspection(&img_result, out_!("test_scale_250x500.png"));
    }

    #[test]
    fn test_rotate90() {
        let img = setup_default_test_image();
        let operation = ImgOp::Rotate90;

        let (xa, ya) = img.dimensions();
        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xa, yb);
        assert_eq!(xb, ya);

        output_test_image_for_manual_inspection(&img_result, out_!("test_rotate90.png"));
    }

    #[test]
    fn test_rotate180() {
        let img = setup_default_test_image();
        let operation = ImgOp::Rotate180;

        let (xa, ya) = img.dimensions();
        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xa, xb);
        assert_eq!(ya, yb);

        output_test_image_for_manual_inspection(&img_result, out_!("test_rotate180.png"));
    }

    #[test]
    fn test_rotate270() {
        let img = setup_default_test_image();
        let operation = ImgOp::Rotate270;

        let (xa, ya) = img.dimensions();
        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xa, yb);
        assert_eq!(xb, ya);

        output_test_image_for_manual_inspection(&img_result, out_!("test_rotate270.png"));
    }

    #[test]
    fn test_unsharpen_pos() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::Unsharpen((20.1, 20));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);
        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_unsharpen_20_1_20.png"));
    }

    #[test]
    fn test_unsharpen_neg() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::Unsharpen((-20.1, -20));

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);
        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(
            &result_img,
            out_!("test_unsharpen_neg20_1_neg20.png"),
        );
    }

    #[test]
    fn test_threshold() {
        let img = setup_default_test_image();
        let cmp = setup_default_test_image();

        let operation = ImgOp::Threshold;

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&[Instr::Operation(operation)]);
        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, out_!("test_threshold.png"));
    }

    #[test]
    fn test_multi() {
        // 217x447px original
        let img = setup_default_test_image();
        let operations = vec![
            Instr::Operation(ImgOp::Resize((80, 100))),
            Instr::Operation(ImgOp::Blur(5.0)),
            Instr::Operation(ImgOp::FlipHorizontal),
            Instr::Operation(ImgOp::FlipVertical),
            Instr::Operation(ImgOp::Rotate90),
        ];
        let (xa, ya) = img.dimensions();

        assert_eq!(ya, 447);
        assert_eq!(xa, 217);

        let operator = ImageEngine::new(img);
        let done = operator.ignite(&operations);

        assert!(done.is_ok());

        let done_image = done.unwrap();
        let (xb, yb) = done_image.dimensions();

        // dim original => 80x100 => 100x80
        assert_eq!(xb, 100);
        assert_eq!(yb, 80);

        output_test_image_for_manual_inspection(&done_image, out_!("test_multi.png"));
    }

    mod imageproc_ops_tests {
        use super::*;
        use crate::wrapper::draw_text_inner::DrawTextInner;
        use crate::wrapper::font_options::{FontOptions, FontScale};

        #[test]
        fn draw_text() {
            let img =
                sic_core::image::DynamicImage::ImageRgb8(sic_core::image::RgbImage::new(200, 200))
                    .into();

            let font_file = Into::<PathBuf>::into(env!("CARGO_MANIFEST_DIR"))
                .join("../../resources/font/Lato-Regular.ttf");

            let operation = ImgOp::DrawText(DrawTextInner::new(
                "HELLO WORLD".to_string(),
                (0, 0),
                FontOptions::new(
                    font_file,
                    Rgba([255, 255, 0, 255]),
                    FontScale::Uniform(16.0),
                ),
            ));

            let operator = ImageEngine::new(img);
            let done = operator.ignite(&[Instr::Operation(operation)]);
            assert!(done.is_ok());

            let result_img = done.unwrap();

            output_test_image_for_manual_inspection(
                &result_img,
                out_!("test_imageproc_ops_draw_text.png"),
            );
        }
    }
}
