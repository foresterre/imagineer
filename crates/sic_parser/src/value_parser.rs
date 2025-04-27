use crate::errors::SicParserError;
use sic_image_engine::wrapper::image_path::ImageFromPath;
use sic_image_engine::wrapper::{filter_type::FilterTypeWrap, gradient_input::GradientInput};
use std::path::PathBuf;

use sic_image_engine::wrapper::draw_text_inner::DrawTextInner;
use sic_image_engine::wrapper::overlay::OverlayInputs;

/// The value parser module has a goal to parse image operation inputs.

#[derive(Clone, Debug)]
pub struct Describable<'a>(&'a str);

impl<'a> From<&'a &'a str> for Describable<'a> {
    fn from(v: &'a &'a str) -> Self {
        Describable(v)
    }
}

impl<'a> From<&'a str> for Describable<'a> {
    fn from(v: &'a str) -> Self {
        Describable(v)
    }
}

impl<'a> From<&'a String> for Describable<'a> {
    fn from(v: &'a String) -> Self {
        Describable(v.as_str())
    }
}

/// This iteration of the parse trait should help with unifying the
/// Pest parsing and cli ops arguments directly from &str.
///
/// From Pest we receive iterators, but we can use as_str, to request &str values.
/// We'll try to use this to map the received values as_str, and then we'll have
/// a similar structure as the image operation arguments from the cli (we receive these
/// eventually as Vec<&str>, thus iterable.
pub trait ParseInputsFromIter {
    type Error;

    fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
        Self: std::marker::Sized;
}

macro_rules! parse_next {
    ($iter:expr, $ty:ty, $err_msg:expr) => {
        $iter
            .next()
            .ok_or_else(|| SicParserError::ValueParsingError($err_msg.to_string()))
            .and_then(|v| {
                let v: Describable = v.into();
                v.0.parse::<$ty>().map_err(|err| {
                    SicParserError::ValueParsingErrorWithInnerError(
                        $err_msg.to_string(),
                        Box::new(err),
                    )
                })
            })?
    };
}

macro_rules! return_if_complete {
    ($iter:expr, $ok_value:expr) => {
        if $iter.next().is_some() {
            Err(SicParserError::ValueParsingError(
                too_many_arguments_err_msg().to_string(),
            ))
        } else {
            Ok($ok_value)
        }
    };
}

macro_rules! define_parse_single_input {
    ($ty:ty, $err_msg:expr) => {
        impl ParseInputsFromIter for $ty {
            type Error = SicParserError;

            fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
            where
                T: IntoIterator,
                T::Item: Into<Describable<'a>> + std::fmt::Debug,
                Self: std::marker::Sized,
            {
                let mut iter = iterable.into_iter();
                let value = parse_next!(iter, $ty, $err_msg);

                return_if_complete!(iter, value)
            }
        }
    };
}

define_parse_single_input!(f32, "Unable to map a value to f32. v2");
define_parse_single_input!(i32, "Unable to map a value to i32. v2");
define_parse_single_input!(u32, "Unable to map a value to u32. v2");
define_parse_single_input!(bool, "Unable to map a value to bool. v2");

const fn too_many_arguments_err_msg() -> &'static str {
    "Too many arguments found for image operation"
}

// FIXME(foresterre): define macros for generic tuples and array (i.e. define_parse_multi!((u32, u32));
//                    we can combine the parse_single_input and parse_multi_input as well.

// for: crop
impl ParseInputsFromIter for (u32, u32, u32, u32) {
    type Error = SicParserError;

    fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let mut iter = iterable.into_iter();
        const ERR_MSG: &str = "Unable to map a value to (u32, u32, u32, u32). v2";

        let res: (u32, u32, u32, u32) = (
            parse_next!(iter, u32, ERR_MSG),
            parse_next!(iter, u32, ERR_MSG),
            parse_next!(iter, u32, ERR_MSG),
            parse_next!(iter, u32, ERR_MSG),
        );

        return_if_complete!(iter, res)
    }
}

// for: filter3x3
impl ParseInputsFromIter for [f32; 9] {
    type Error = SicParserError;

    fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let mut iter = iterable.into_iter();
        const ERR_MSG: &str = "Unable to map a value to [f32; 9]. v2";

        let res: [f32; 9] = [
            //
            parse_next!(iter, f32, ERR_MSG),
            parse_next!(iter, f32, ERR_MSG),
            parse_next!(iter, f32, ERR_MSG),
            //
            parse_next!(iter, f32, ERR_MSG),
            parse_next!(iter, f32, ERR_MSG),
            parse_next!(iter, f32, ERR_MSG),
            //
            parse_next!(iter, f32, ERR_MSG),
            parse_next!(iter, f32, ERR_MSG),
            parse_next!(iter, f32, ERR_MSG),
        ];

        return_if_complete!(iter, res)
    }
}

// for: resize
impl ParseInputsFromIter for (u32, u32) {
    type Error = SicParserError;

    fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let mut iter = iterable.into_iter();
        const ERR_MSG: &str = "Unable to map a value to (u32, u32). v2";

        let res: (u32, u32) = (
            parse_next!(iter, u32, ERR_MSG),
            parse_next!(iter, u32, ERR_MSG),
        );

        return_if_complete!(iter, res)
    }
}

// for: unsharpen
impl ParseInputsFromIter for (f32, i32) {
    type Error = SicParserError;

    fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let mut iter = iterable.into_iter();
        const ERR_MSG: &str = "Unable to map a value to (f32, i32). v2";

        let res: (f32, i32) = (
            parse_next!(iter, f32, ERR_MSG),
            parse_next!(iter, i32, ERR_MSG),
        );

        return_if_complete!(iter, res)
    }
}

impl ParseInputsFromIter for String {
    type Error = SicParserError;

    fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let mut iter = iterable.into_iter();
        const ERR_MSG: &str = "Unable to map a value to string";

        let res: Describable<'a> = iter
            .next()
            .ok_or_else(|| SicParserError::ValueParsingError(ERR_MSG.to_string()))?
            .into();

        return_if_complete!(iter, String::from(res.0))
    }
}

impl ParseInputsFromIter for (i64, i64) {
    type Error = SicParserError;

    fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let mut iter = iterable.into_iter();
        const ERR_MSG: &str = "Unable to map a value to (u32, u32). v2";

        let res: (i64, i64) = (
            parse_next!(iter, i64, ERR_MSG),
            parse_next!(iter, i64, ERR_MSG),
        );

        return_if_complete!(iter, res)
    }
}

impl ParseInputsFromIter for ImageFromPath {
    type Error = SicParserError;

    fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let mut iter = iterable.into_iter();

        let path = parse_to_path_buf(iter.next().map(Into::<Describable>::into))?;

        return_if_complete!(iter, ImageFromPath::new(path))
    }
}

impl ParseInputsFromIter for OverlayInputs {
    type Error = SicParserError;

    fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let mut iter = iterable.into_iter();
        let image_path = parse_to_path_buf(iter.next().map(Into::<Describable>::into))?;

        let position: (i64, i64) = (
            parse_next!(
                iter,
                i64,
                "x-axis position value for overlay should be a natural number"
            ),
            parse_next!(
                iter,
                i64,
                "y-axis position value for overlay should be a natural number"
            ),
        );

        let overlay_inputs = OverlayInputs::new(ImageFromPath::new(image_path), position);

        return_if_complete!(iter, overlay_inputs)
    }
}

impl ParseInputsFromIter for FilterTypeWrap {
    type Error = SicParserError;

    fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        let mut iter = iterable.into_iter();

        let err_msg_no_such_element =
            || "A filter type was expected but none was found.".to_string();

        let filter_type = iter
            .next()
            .map(Into::<Describable>::into)
            .ok_or_else(|| SicParserError::ValueParsingError(err_msg_no_such_element()))
            .and_then(|v: Describable| {
                FilterTypeWrap::try_from_str(v.0).map_err(SicParserError::FilterTypeError)
            })?;

        return_if_complete!(iter, filter_type)
    }
}

fn parse_to_path_buf(value: Option<Describable>) -> Result<PathBuf, SicParserError> {
    let err_msg_no_such_element = || "A path was expected but none was found.".to_string();

    value
        .ok_or_else(|| SicParserError::ValueParsingError(err_msg_no_such_element()))
        .map(|v: Describable| PathBuf::from(v.0))
}

impl ParseInputsFromIter for DrawTextInner {
    type Error = SicParserError;

    fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        use crate::named_value::NamedValue;
        use sic_core::image::Rgba;
        use sic_image_engine::wrapper::font_options::{FontOptions, FontScale};

        let mut iter = iterable.into_iter();

        let text = iter
            .next()
            .map(Into::<Describable>::into)
            .ok_or_else(|| SicParserError::ExpectedValue(String::from("String")))?
            .0;
        let coord = parse_next!(iter, NamedValue, "Coord");
        let color = parse_next!(iter, NamedValue, "Rgba");
        let size = parse_next!(iter, NamedValue, "Float");
        let font_file = parse_next!(iter, NamedValue, "String");

        let res = DrawTextInner::new(
            text.to_string(),
            (coord.extract_coord()).map_err(SicParserError::NamedValueParsingError)?,
            FontOptions::new(
                font_file
                    .extract_font()
                    .map_err(SicParserError::NamedValueParsingError)?,
                Rgba(
                    color
                        .extract_rgba()
                        .map_err(SicParserError::NamedValueParsingError)?,
                ),
                FontScale::Uniform(
                    size.extract_size()
                        .map_err(SicParserError::NamedValueParsingError)?,
                ),
            ),
        );

        return_if_complete!(iter, res)
    }
}

// Horizontal gradient
impl ParseInputsFromIter for GradientInput {
    type Error = SicParserError;

    fn parse<'a, T>(iterable: T) -> Result<Self, Self::Error>
    where
        T: IntoIterator,
        T::Item: Into<Describable<'a>> + std::fmt::Debug,
        Self: std::marker::Sized,
    {
        use crate::named_value::NamedValue;
        use sic_core::image::Rgba;

        let mut iter = iterable.into_iter();
        let color1 = parse_next!(iter, NamedValue, "Rgba");
        let color2 = parse_next!(iter, NamedValue, "Rgba");

        let res = GradientInput::new((
            Rgba(
                color1
                    .extract_rgba()
                    .map_err(SicParserError::NamedValueParsingError)?,
            ),
            Rgba(
                color2
                    .extract_rgba()
                    .map_err(SicParserError::NamedValueParsingError)?,
            ),
        ));

        return_if_complete!(iter, res)
    }
}

#[cfg(test)]
mod tests_parse_from_iter {
    use super::*;
    use sic_testing::*;

    macro_rules! assert_iter_impl {
        ($lhs_iter:expr, $rhs_iter:expr, $f:expr) => {
            assert!($lhs_iter.into_iter().zip($rhs_iter.into_iter()).all($f))
        };
    }

    macro_rules! assert_iter_f32 {
        ($lhs_iter:expr, $rhs_iter:expr) => {
            assert_iter_impl!($lhs_iter, $rhs_iter, |(l, r)| (l - r).abs() < f32::EPSILON)
        };
    }

    #[test]
    fn a_f32() {
        let some: f32 = ParseInputsFromIter::parse(&["-1.03"]).unwrap();
        sic_testing::approx_eq_f32!(some, -1.03f32)
    }

    mod tuple_u32_u32_u32_u32 {
        use super::*;

        #[test]
        fn should_succeed_with() {
            let some: (u32, u32, u32, u32) =
                ParseInputsFromIter::parse(&["03579", "0", "1", "1"]).unwrap();
            assert_eq!(some, (3579u32, 0u32, 1u32, 1u32));
        }

        #[pm(input = {
            &["-4", "3", "2", "1"],         // &[a, _b, _c, _d]: a not u32 (neg)
            &["4", "-3", "2", "1"],         // &[_a, b, _c, _d]: b not u32 (neg)
            &["4", "3", "-2", "1"],         // &[_a, _b, c, _d]: c not u32 (neg)
            &["4", "3", "2", "-1"],         // &[_a, _b, _c, d]: d not u32 (neg)
            &["4", "3", "2", "o"],          // &[..., x]: x not f32 (not a number)
            &["4", "3", "2"],               // len() == 4 expected
            &["4", "3", "2", "1", "0"],     // len() == 4 expected
            &[],                            // empty
        })]
        fn expected_failures(input: &[&str]) {
            let result: Result<(u32, u32, u32, u32), SicParserError> =
                ParseInputsFromIter::parse(input);
            assert!(result.is_err());
        }
    }

    mod array_f32x9 {
        use super::*;

        #[test]
        fn array_of_f32() {
            let some: [f32; 9] = ParseInputsFromIter::parse(&[
                "1", "2", "3", "4", "5.5", "-6.0", "7", "8", "-9.9999",
            ])
            .unwrap();
            const EXPECTED: [f32; 9] = [
                1f32, 2f32, 3f32, 4f32, 5.5f32, -6.0f32, 7f32, 8f32, -9.9999f32,
            ];

            assert_iter_f32!(&some, &EXPECTED)
        }

        #[pm(input = {
            &["1", "2", "3", "4", "5.5", "-6.0", "7", "8", "lalala"],               // &[..., x]: x not f32 (not a number)
            &["1", "2", "3", "4", "5.5", "-6.0", "7", "8"],                         // len() == 9 expected
            &["1", "2", "3", "4", "5.5", "-6.0", "7", "8", "-9.9999", "1"],         // len() == 9 expected
            &[],                                                                    // empty
        })]
        fn expected_failures(input: &[&str]) {
            let result: Result<[f32; 9], SicParserError> = ParseInputsFromIter::parse(input);
            assert!(result.is_err())
        }
    }

    mod tuple_u32_u32 {
        use super::*;

        #[test]
        fn a_tuple_of_u32_u32() {
            let some: (u32, u32) = ParseInputsFromIter::parse(&["03579", "0"]).unwrap();
            assert_eq!(some, (3579u32, 0u32))
        }

        #[pm(input = {
            &["4", "-0"],   // [_x, y]: y not u32 (neg 0)
            &["4", "-4"],   // [_x, y]: y not u32 (neg number)
            &["4", "f"],    // [_x, y]: y not u32 (not a number)
            &["03579"],     // len() == 2 expected
            &[],            // empty
        })]
        fn expected_failures(input: &[&str]) {
            let result: Result<(u32, u32), SicParserError> = ParseInputsFromIter::parse(input);
            assert!(result.is_err());
        }
    }

    mod tuple_f32_i32 {
        use super::*;

        #[test]
        fn a_tuple_of_f32_i32() {
            let some: (f32, i32) = ParseInputsFromIter::parse(&["-03579.1", "-0"]).unwrap();
            assert_eq!(some, (-3579.1f32, 0i32))
        }

        #[pm(input = {
            &["-f", "-1"],      // [x, _y]: x not f32
            &["-1.0", "f"],     // [_x, y]: y not i32
            &["4"],             // len() == 2 expected
            &["4", "4", "4"],   // len() == 2 expected
            &[],                // empty
        })]
        fn expected_failures(input: &[&str]) {
            let result: Result<(f32, i32), SicParserError> = ParseInputsFromIter::parse(input);
            assert!(result.is_err());
        }
    }
}
