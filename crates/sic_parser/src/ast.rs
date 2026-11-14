use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub struct Program<'s> {
    pub statements: Vec<Statement<'s>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'s> {
    Call(Call<'s, Expr<'s>>),
    Set { option: &'s str, value: Expr<'s> },
    Del { option: &'s str },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call<'s, A> {
    pub name: &'s str,
    pub args: Vec<A>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'s> {
    Literal(Literal<'s>),
    Call(Call<'s, Literal<'s>>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Literal<'s> {
    Integer(&'s str),
    Decimal(&'s str),
    Bool(bool),
    String(&'s str),
    Ident(&'s str),
}

impl Display for Literal<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Integer(lexeme) | Literal::Decimal(lexeme) | Literal::Ident(lexeme) => {
                f.write_str(lexeme)
            }
            Literal::Bool(value) => write!(f, "{value}"),
            Literal::String(text) if text.contains('"') => write!(f, "'{text}'"),
            Literal::String(text) => write!(f, "\"{text}\""),
        }
    }
}

impl<A: Display> Display for Call<'_, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{arg}")?;
        }
        f.write_str(")")
    }
}

impl Display for Expr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(literal) => literal.fmt(f),
            Expr::Call(call) => call.fmt(f),
        }
    }
}
