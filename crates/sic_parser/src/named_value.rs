//! Parsing for named values, which have the following syntax N(T)
//! where N is the identifier of the named value, and T is a comma separated tuple of values,
//! like so: `a,b,c`. Dangling commas are not supported. A full example look like this: `rgb(4, 255, 0)`.

use super::Rule;
use pest::iterators::Pair;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NamedValueError {
    #[error(
        "Named value argument expected a string, but given value `{0}` is not valid (note: \
        the value of the string should be wrapped in either single or double quotation marks, \
        e.g. 'hello' or \"hello\")"
    )]
    FaultyString(String),

    #[error("Named value expected an identifier but none was found")]
    IdentifierNotFound,

    #[error("Found an unknown identifier for named value '{0}'")]
    IdentifierInvalid(String),

    #[error("Named value argument has an incorrect type")]
    InvalidArgumentType,

    #[error("Unable to create named value: no matching arguments for identifier '{0}' found")]
    UnableToCreateNamedValueWithArgs(Ident),

    #[error(
        "Unable to extract arguments: expected named value with identifier '{0}' but found '{1}'"
    )]
    UnableToExtractNamedValueArgs(String, String),

    #[error("Unable to extract value: expected value of type '{0}' but found '{1}'")]
    UnableToExtractValue(String, String),

    #[error("Unable to parse value '{0}', with type '{1}'")]
    UnableToParse(String, String),
}

type NVResult<T> = Result<T, NamedValueError>;

pub fn parse_named_value(pair: Pair<'_, Rule>) -> NVResult<NamedValue> {
    let mut pairs = pair.into_inner();

    // identifier
    let ident = pairs.next().ok_or(NamedValueError::IdentifierNotFound)?;

    let ident = match ident.as_rule() {
        Rule::ident => parse_ident(ident.as_str())?,
        _ => return Err(NamedValueError::IdentifierNotFound),
    };

    // arguments
    let arguments = pairs
        .map(|pair| Value::try_from_pair(pair, ident))
        .collect::<NVResult<Vec<_>>>()?;

    NamedValue::try_from_annotated(AnnotatedArgs { ident, arguments })
}

impl FromStr for NamedValue {
    type Err = NamedValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut inputs = s.splitn(2, '(');

        let ident = inputs.next().ok_or(NamedValueError::IdentifierNotFound)?;

        let ident = parse_ident(ident)?;

        let arguments = inputs
            .next()
            .and_then(|right_side| right_side.rsplitn(2, ')').last())
            .ok_or(NamedValueError::UnableToCreateNamedValueWithArgs(ident))?;

        let arguments = arguments
            .split(',')
            .map(|arg| Value::try_from_str(arg.trim(), ident))
            .collect::<NVResult<Vec<_>>>()?;

        NamedValue::try_from_annotated(AnnotatedArgs { ident, arguments })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Ident {
    // rgba(<u8>,<u8>,<u8>,<u8>)
    Rgba,

    // size(<u32>)
    Size,

    // font("<path>")
    Font,

    // coord(<u32>, <u32>)
    Coord,
}

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rgba => f.write_str("Rgba"),
            Self::Size => f.write_str("Size"),
            Self::Font => f.write_str("Font"),
            Self::Coord => f.write_str("Coord"),
        }
    }
}

fn parse_ident(ident: &str) -> NVResult<Ident> {
    let ident = match ident {
        "rgba" => Ident::Rgba,
        "size" => Ident::Size,
        "font" => Ident::Font,
        "coord" => Ident::Coord,
        _ => return Err(NamedValueError::IdentifierInvalid(ident.to_string())),
    };

    Ok(ident)
}

#[derive(Debug, Clone)]
enum Value<'a> {
    Byte(u8),
    Float(f32),
    Integer(i32),
    NatNum(u32),
    String(&'a str),
}

impl<'a> Value<'a> {
    pub fn try_from_pair(pair: Pair<'a, Rule>, ident: Ident) -> NVResult<Self> {
        match (pair.as_rule(), ident) {
            (Rule::fp, Ident::Rgba) => Ok(Value::parse_byte(pair.as_str())?),
            (Rule::fp, Ident::Size) => Ok(Value::parse_float(pair.as_str())?),
            (Rule::fp, Ident::Coord) => Ok(Value::parse_integer(pair.as_str())?),
            (Rule::string_unicode, _) => Ok(Value::parse_string(pair.into_inner().as_str())?),
            _ => Err(NamedValueError::InvalidArgumentType),
        }
    }

    pub fn try_from_str(s: &'a str, ident: Ident) -> NVResult<Self> {
        match ident {
            Ident::Rgba => Ok(Value::parse_byte(s)?),
            Ident::Size => Ok(Value::parse_float(s)?),
            Ident::Coord => Ok(Value::parse_integer(s)?),
            Ident::Font => Ok(Value::parse_string(slice_str_tokens(s)?)?),
        }
    }

    pub fn extract_byte(&self) -> NVResult<u8> {
        if let Self::Byte(inner) = self {
            Ok(*inner)
        } else {
            Err(NamedValueError::UnableToExtractValue(
                String::from("Byte"),
                self.error_type(),
            ))
        }
    }

    pub fn extract_float(&self) -> NVResult<f32> {
        if let Self::Float(inner) = self {
            Ok(*inner)
        } else {
            Err(NamedValueError::UnableToExtractValue(
                String::from("Float"),
                self.error_type(),
            ))
        }
    }

    pub fn extract_integer(&self) -> NVResult<i32> {
        if let Self::Integer(inner) = self {
            Ok(*inner)
        } else {
            Err(NamedValueError::UnableToExtractValue(
                String::from("Integer"),
                self.error_type(),
            ))
        }
    }

    #[allow(unused)]
    pub fn extract_nat_num(&self) -> NVResult<u32> {
        if let Self::NatNum(inner) = self {
            Ok(*inner)
        } else {
            Err(NamedValueError::UnableToExtractValue(
                String::from("NatNum"),
                self.error_type(),
            ))
        }
    }

    pub fn extract_string(&self) -> NVResult<&str> {
        if let Self::String(inner) = self {
            Ok(inner)
        } else {
            Err(NamedValueError::UnableToExtractValue(
                String::from("String"),
                self.error_type(),
            ))
        }
    }

    fn parse_byte(value: &str) -> NVResult<Self> {
        value
            .parse::<u8>()
            .map(Value::Byte)
            .map_err(|_err| NamedValueError::UnableToParse(value.to_string(), String::from("Byte")))
    }

    fn parse_float(value: &str) -> NVResult<Self> {
        value.parse::<f32>().map(Value::Float).map_err(|_err| {
            NamedValueError::UnableToParse(value.to_string(), String::from("Float"))
        })
    }

    fn parse_integer(value: &str) -> NVResult<Self> {
        value.parse::<i32>().map(Value::Integer).map_err(|_err| {
            NamedValueError::UnableToParse(value.to_string(), String::from("Integer"))
        })
    }

    #[allow(unused)]
    fn parse_nat_num(value: &str) -> NVResult<Self> {
        value.parse::<u32>().map(Value::NatNum).map_err(|_err| {
            NamedValueError::UnableToParse(value.to_string(), String::from("NatNum"))
        })
    }

    fn parse_string(value: &'a str) -> NVResult<Self> {
        Ok(Value::String(value))
    }

    fn error_type(&self) -> String {
        let typ = match self {
            Self::Byte(_) => "Byte",
            Self::Float(_) => "Float",
            Self::Integer(_) => "Integer",
            Self::NatNum(_) => "NatNum",
            Self::String(_) => "String",
        };

        typ.to_string()
    }
}

#[derive(Debug, Clone)]
struct AnnotatedArgs<'a> {
    ident: Ident,
    arguments: Vec<Value<'a>>,
}

impl<'a> AnnotatedArgs<'a> {
    fn ident(&self) -> Ident {
        self.ident
    }

    // moves arguments!
    fn arguments(&self) -> &[Value<'a>] {
        self.arguments.as_slice()
    }
}

#[derive(Debug, Clone)]
pub enum NamedValue {
    Rgba(u8, u8, u8, u8),
    Size(f32),
    Font(PathBuf),
    Coord((i32, i32)),
}

impl NamedValue {
    fn try_from_annotated(args: AnnotatedArgs) -> NVResult<Self> {
        match args.ident() {
            Ident::Rgba => NamedValue::create_rgba(args.arguments()),
            Ident::Size => NamedValue::create_size(args.arguments()),
            Ident::Font => NamedValue::create_font(args.arguments()),
            Ident::Coord => NamedValue::create_coord(args.arguments()),
        }
    }

    pub fn extract_rgba(&self) -> NVResult<[u8; 4]> {
        if let Self::Rgba(r, g, b, a) = self {
            Ok([*r, *g, *b, *a])
        } else {
            Err(NamedValueError::UnableToExtractValue(
                String::from("Rgba"),
                self.error_type(),
            ))
        }
    }

    pub fn extract_size(&self) -> NVResult<f32> {
        if let Self::Size(size) = self {
            Ok(*size)
        } else {
            Err(NamedValueError::UnableToExtractValue(
                String::from("Size"),
                self.error_type(),
            ))
        }
    }

    pub fn extract_font(&self) -> NVResult<PathBuf> {
        if let Self::Font(font) = self {
            Ok(font.to_path_buf())
        } else {
            Err(NamedValueError::UnableToExtractValue(
                String::from("Font"),
                self.error_type(),
            ))
        }
    }

    pub fn extract_coord(&self) -> NVResult<(i32, i32)> {
        if let Self::Coord(coords) = self {
            Ok(*coords)
        } else {
            Err(NamedValueError::UnableToExtractValue(
                String::from("Coord"),
                self.error_type(),
            ))
        }
    }

    fn create_rgba(args: &[Value]) -> NVResult<Self> {
        match args {
            [r, g, b, a] => Ok(Self::Rgba(
                r.extract_byte()?,
                g.extract_byte()?,
                b.extract_byte()?,
                a.extract_byte()?,
            )),
            _ => Err(NamedValueError::UnableToCreateNamedValueWithArgs(
                Ident::Rgba,
            )),
        }
    }

    fn create_size(args: &[Value]) -> NVResult<Self> {
        match args {
            [size] => Ok(Self::Size(size.extract_float()?)),
            _ => Err(NamedValueError::UnableToCreateNamedValueWithArgs(
                Ident::Size,
            )),
        }
    }

    fn create_font(args: &[Value]) -> NVResult<Self> {
        match args {
            [font_file] => Ok(Self::Font(font_file.extract_string()?.into())),
            _ => Err(NamedValueError::UnableToCreateNamedValueWithArgs(
                Ident::Font,
            )),
        }
    }

    fn create_coord(args: &[Value]) -> NVResult<Self> {
        match args {
            [x, y] => Ok(Self::Coord((x.extract_integer()?, y.extract_integer()?))),
            _ => Err(NamedValueError::UnableToCreateNamedValueWithArgs(
                Ident::Coord,
            )),
        }
    }

    fn error_type(&self) -> String {
        let typ = match self {
            Self::Rgba(_, _, _, _) => "Rgba",
            Self::Size(_) => "Size",
            Self::Font(_) => "Font",
            Self::Coord(_) => "Coord",
        };

        typ.to_string()
    }
}

fn slice_str_tokens(s: &str) -> NVResult<&str> {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        Ok(&s[1..s.len() - 1])
    } else {
        Err(NamedValueError::FaultyString(s.to_string()))
    }
}
