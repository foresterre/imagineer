use crate::ast::{Call, Expr, Literal};
use crate::errors::{ArgumentError, ResolveError};
use sic_image_engine::wrapper::filter_type::FilterTypeWrap;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Byte,
    UInt,
    Int,
    Float,
    Bool,
    String,
    Path,
    FilterType,
    Color,
    Coord,
    FontSize,
    Font,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Type::Byte => "a byte (0 to 255)",
            Type::UInt => "a whole number of 0 or more",
            Type::Int => "a whole number",
            Type::Float => "a number",
            Type::Bool => "true or false",
            Type::String => "a quoted string",
            Type::Path => "a quoted path",
            Type::FilterType => "a sampling filter name",
            Type::Color => "a color, rgba(r, g, b, a)",
            Type::Coord => "a coordinate, coord(x, y)",
            Type::FontSize => "a font size, size(px)",
            Type::Font => "a font, font(path)",
        })
    }
}

pub trait Param: Sized {
    const TYPE: Type;

    fn from_literal(literal: &Literal<'_>) -> Result<Self, ArgumentError> {
        Err(ArgumentError::mismatch(Self::TYPE, literal))
    }

    fn from_expr(expr: &Expr<'_>) -> Result<Self, ArgumentError> {
        match expr {
            Expr::Literal(literal) => Self::from_literal(literal),
            Expr::Call(call) => Err(ArgumentError::mismatch(Self::TYPE, call)),
        }
    }
}

pub trait Argument {
    fn bind<P: Param>(&self) -> Result<P, ArgumentError>;
}

impl Argument for Literal<'_> {
    fn bind<P: Param>(&self) -> Result<P, ArgumentError> {
        P::from_literal(self)
    }
}

impl Argument for Expr<'_> {
    fn bind<P: Param>(&self) -> Result<P, ArgumentError> {
        P::from_expr(self)
    }
}

pub trait Params: Sized {
    type Names;

    fn bind<A: Argument>(
        function: &str,
        names: Self::Names,
        args: &[A],
    ) -> Result<Self, ResolveError>;
}

impl<A: Argument> Call<'_, A> {
    pub fn bind<P: Params>(&self, names: P::Names) -> Result<P, ResolveError> {
        P::bind(self.name, names, &self.args)
    }
}

pub fn bind_argument<P: Param, A: Argument>(
    function: &str,
    param: &'static str,
    arg: &A,
) -> Result<P, ResolveError> {
    arg.bind().map_err(|source| ResolveError::Argument {
        function: function.to_string(),
        param,
        source: Box::new(source),
    })
}

macro_rules! params {
    ($count:literal $(, $P:ident $name:ident $arg:ident)*) => {
        impl<$($P: Param),*> Params for ($($P,)*) {
            type Names = [&'static str; $count];

            fn bind<A: Argument>(
                function: &str,
                names: Self::Names,
                args: &[A],
            ) -> Result<Self, ResolveError> {
                let [$($arg),*] = <&[A; $count]>::try_from(args).map_err(|_| {
                    ResolveError::ArgumentCount {
                        function: function.to_string(),
                        expected: $count,
                        found: args.len(),
                    }
                })?;
                let [$($name),*] = names;

                Ok(($(bind_argument(function, $name, $arg)?,)*))
            }
        }
    };
}

params!(0);
params!(1, P1 n1 a1);
params!(2, P1 n1 a1, P2 n2 a2);
params!(3, P1 n1 a1, P2 n2 a2, P3 n3 a3);
params!(4, P1 n1 a1, P2 n2 a2, P3 n3 a3, P4 n4 a4);
params!(5, P1 n1 a1, P2 n2 a2, P3 n3 a3, P4 n4 a4, P5 n5 a5);
params!(9, P1 n1 a1, P2 n2 a2, P3 n3 a3, P4 n4 a4, P5 n5 a5, P6 n6 a6, P7 n7 a7, P8 n8 a8, P9 n9 a9);

macro_rules! integer_param {
    ($ty:ty, $type:expr) => {
        impl Param for $ty {
            const TYPE: Type = $type;

            fn from_literal(literal: &Literal<'_>) -> Result<Self, ArgumentError> {
                match literal {
                    Literal::Integer(lexeme) => out_of_range(Self::TYPE, lexeme, lexeme.parse()),
                    other => Err(ArgumentError::mismatch(Self::TYPE, other)),
                }
            }
        }
    };
}

integer_param!(u8, Type::Byte);
integer_param!(u32, Type::UInt);
integer_param!(i32, Type::Int);

impl Param for f32 {
    const TYPE: Type = Type::Float;

    fn from_literal(literal: &Literal<'_>) -> Result<Self, ArgumentError> {
        match literal {
            Literal::Integer(lexeme) | Literal::Decimal(lexeme) => {
                out_of_range(Self::TYPE, lexeme, lexeme.parse())
            }
            other => Err(ArgumentError::mismatch(Self::TYPE, other)),
        }
    }
}

impl Param for bool {
    const TYPE: Type = Type::Bool;

    fn from_literal(literal: &Literal<'_>) -> Result<Self, ArgumentError> {
        match literal {
            Literal::Bool(value) => Ok(*value),
            other => Err(ArgumentError::mismatch(Self::TYPE, other)),
        }
    }
}

impl Param for String {
    const TYPE: Type = Type::String;

    fn from_literal(literal: &Literal<'_>) -> Result<Self, ArgumentError> {
        match literal {
            Literal::String(text) => Ok(text.to_string()),
            other => Err(ArgumentError::mismatch(Self::TYPE, other)),
        }
    }
}

impl Param for PathBuf {
    const TYPE: Type = Type::Path;

    fn from_literal(literal: &Literal<'_>) -> Result<Self, ArgumentError> {
        match literal {
            Literal::String(path) => Ok(PathBuf::from(path)),
            other => Err(ArgumentError::mismatch(Self::TYPE, other)),
        }
    }
}

impl Param for FilterTypeWrap {
    const TYPE: Type = Type::FilterType;

    fn from_literal(literal: &Literal<'_>) -> Result<Self, ArgumentError> {
        match literal {
            Literal::Ident(name) => {
                FilterTypeWrap::try_from_str(name).map_err(ArgumentError::FilterType)
            }
            other => Err(ArgumentError::mismatch(Self::TYPE, other)),
        }
    }
}

fn out_of_range<T, E>(
    expected: Type,
    lexeme: &str,
    parsed: Result<T, E>,
) -> Result<T, ArgumentError> {
    parsed.map_err(|_| ArgumentError::OutOfRange {
        expected,
        lexeme: lexeme.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sic_core::image::imageops::FilterType;

    fn integer<P: Param>(lexeme: &str) -> Result<P, ArgumentError> {
        Literal::Integer(lexeme).bind()
    }

    fn decimal<P: Param>(lexeme: &str) -> Result<P, ArgumentError> {
        Literal::Decimal(lexeme).bind()
    }

    #[test]
    fn byte() {
        assert_eq!(integer::<u8>("0").unwrap(), 0);
        assert_eq!(integer::<u8>("255").unwrap(), 255);
        assert!(integer::<u8>("256").is_err());
        assert!(integer::<u8>("-1").is_err());
        assert!(decimal::<u8>("1.0").is_err());
    }

    #[test]
    fn uint() {
        assert_eq!(integer::<u32>("03579").unwrap(), 3579);
        assert_eq!(integer::<u32>("4294967295").unwrap(), u32::MAX);
        assert!(integer::<u32>("4294967296").is_err());
        assert!(integer::<u32>("-1").is_err());
        assert!(integer::<u32>("-0").is_err());
        assert!(decimal::<u32>("1.5").is_err());
    }

    #[test]
    fn int() {
        assert_eq!(integer::<i32>("-0").unwrap(), 0);
        assert_eq!(integer::<i32>("-3579").unwrap(), -3579);
        assert!(integer::<i32>("2147483648").is_err());
        assert!(decimal::<i32>("-88.0").is_err());
    }

    #[test]
    fn float_accepts_integers() {
        assert_eq!(integer::<f32>("15").unwrap(), 15.0);
        assert_eq!(decimal::<f32>("-03579.1").unwrap(), -3579.1);
        assert_eq!(decimal::<f32>("15.8").unwrap(), 15.8);
    }

    #[test]
    fn non_numbers() {
        assert!(Literal::Bool(true).bind::<f32>().is_err());
        assert!(Literal::String("1").bind::<u32>().is_err());
        assert!(Literal::Ident("o").bind::<i32>().is_err());
    }

    #[test]
    fn bool_string_and_path() {
        assert!(Literal::Bool(false).bind::<bool>().is_ok());
        assert!(Literal::Ident("yes").bind::<bool>().is_err());
        assert_eq!(Literal::String("a b").bind::<String>().unwrap(), "a b");
        assert_eq!(
            Literal::String("in.png").bind::<PathBuf>().unwrap(),
            PathBuf::from("in.png")
        );
        assert!(Literal::Ident("in").bind::<PathBuf>().is_err());
    }

    #[test]
    fn filter_type() {
        assert_eq!(
            Literal::Ident("GAUSSIAN").bind::<FilterTypeWrap>().unwrap(),
            FilterTypeWrap::new(FilterType::Gaussian)
        );
        assert!(Literal::Ident("tri").bind::<FilterTypeWrap>().is_err());
        assert!(Literal::String("nearest").bind::<FilterTypeWrap>().is_err());
    }

    #[test]
    fn argument_count() {
        let call = Call {
            name: "op",
            args: vec![Literal::Integer("1")],
        };

        assert_eq!(call.bind::<(u32,)>(["a"]).unwrap(), (1,));
        assert!(matches!(
            call.bind::<(u32, u32)>(["a", "b"]),
            Err(ResolveError::ArgumentCount {
                expected: 2,
                found: 1,
                ..
            })
        ));
        assert!(call.bind::<()>([]).is_err());
    }
}
