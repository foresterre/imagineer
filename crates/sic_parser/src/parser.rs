use crate::ast::{Call, Expr, Literal, Program, Statement};
use crate::errors::SyntaxError;
use winnow::ascii::{digit1, multispace0};
use winnow::combinator::{
    alt, cut_err, delimited, eof, fail, opt, preceded, repeat, separated, terminated,
};
use winnow::error::{ContextError, ErrMode, StrContext, StrContextValue};
use winnow::token::{one_of, take_till, take_while};
use winnow::{ModalResult, Parser};

pub fn parse_program(input: &str) -> Result<Program<'_>, SyntaxError> {
    program
        .parse(input)
        .map_err(|error| SyntaxError::new(input, error.offset(), error.inner()))
}

pub fn parse_expr(input: &str) -> Result<Expr<'_>, SyntaxError> {
    expr.parse(input)
        .map_err(|error| SyntaxError::new(input, error.offset(), error.inner()))
}

fn program<'s>(input: &mut &'s str) -> ModalResult<Program<'s>> {
    delimited(
        blank,
        repeat(0.., terminated(statement, terminator)),
        end_of_program,
    )
    .map(|statements| Program { statements })
    .parse_next(input)
}

fn terminator(input: &mut &str) -> ModalResult<()> {
    cut_err(delimited(blank, alt((';'.void(), eof.void())), blank))
        .context(expected("';'"))
        .context(expected("end of input"))
        .parse_next(input)
}

fn end_of_program(input: &mut &str) -> ModalResult<()> {
    eof.void()
        .context(expected("an operation"))
        .context(expected("end of input"))
        .parse_next(input)
}

fn statement<'s>(input: &mut &'s str) -> ModalResult<Statement<'s>> {
    alt((set, del, call.map(Statement::Call))).parse_next(input)
}

fn set<'s>(input: &mut &'s str) -> ModalResult<Statement<'s>> {
    preceded(
        keyword("set"),
        cut_err((option, preceded(sp, expr).context(expected("a value")))),
    )
    .map(|(option, value)| Statement::Set { option, value })
    .context(StrContext::Label("set statement"))
    .parse_next(input)
}

fn del<'s>(input: &mut &'s str) -> ModalResult<Statement<'s>> {
    preceded(keyword("del"), cut_err(option))
        .map(|option| Statement::Del { option })
        .context(StrContext::Label("del statement"))
        .parse_next(input)
}

fn option<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    preceded(sp, ident)
        .context(expected("an option name"))
        .parse_next(input)
}

fn call<'s>(input: &mut &'s str) -> ModalResult<Call<'s, Expr<'s>>> {
    let args = preceded(sp, separated(1.., expr, argument_separator));

    (ident, opt(args))
        .map(|(name, args)| Call {
            name,
            args: args.unwrap_or_default(),
        })
        .parse_next(input)
}

fn argument_separator(input: &mut &str) -> ModalResult<()> {
    (sp, opt(('|', sp))).void().parse_next(input)
}

fn expr<'s>(input: &mut &'s str) -> ModalResult<Expr<'s>> {
    alt((value_call.map(Expr::Call), literal.map(Expr::Literal))).parse_next(input)
}

fn value_call<'s>(input: &mut &'s str) -> ModalResult<Call<'s, Literal<'s>>> {
    let args = separated(0.., preceded(pad, literal), (pad, ','));
    let close = (pad, ')').context(expected("')'"));

    (ident, preceded('(', cut_err(terminated(args, close))))
        .map(|(name, args)| Call { name, args })
        .parse_next(input)
}

fn literal<'s>(input: &mut &'s str) -> ModalResult<Literal<'s>> {
    alt((
        number,
        string.map(Literal::String),
        ident.map(bool_or_ident),
        fail.context(expected("a number, string or name")),
    ))
    .parse_next(input)
}

fn bool_or_ident(name: &str) -> Literal<'_> {
    match name {
        "true" => Literal::Bool(true),
        "false" => Literal::Bool(false),
        _ => Literal::Ident(name),
    }
}

fn number<'s>(input: &mut &'s str) -> ModalResult<Literal<'s>> {
    let fraction = preceded('.', cut_err(digit1).context(expected("a digit after '.'")));

    (opt('-'), digit1, opt(fraction))
        .with_taken()
        .map(|((_, _, fraction), lexeme)| match fraction {
            Some(_) => Literal::Decimal(lexeme),
            None => Literal::Integer(lexeme),
        })
        .parse_next(input)
}

fn string<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    alt((quoted('"'), quoted('\''))).parse_next(input)
}

fn quoted<'s>(quote: char) -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
    let close = quote.context(expected("a closing quote"));

    preceded(quote, cut_err(terminated(take_till(0.., quote), close)))
}

fn ident<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    let first = one_of(|c: char| c.is_ascii_alphabetic());
    let rest = take_while(0.., |c: char| c.is_ascii_alphanumeric() || c == '-');

    (first, rest).take().parse_next(input)
}

fn keyword<'s>(word: &'static str) -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
    ident.verify(move |name: &str| name.eq_ignore_ascii_case(word))
}

fn sp(input: &mut &str) -> ModalResult<()> {
    take_while(1.., ' ').void().parse_next(input)
}

fn pad(input: &mut &str) -> ModalResult<()> {
    take_while(0.., ' ').void().parse_next(input)
}

fn blank(input: &mut &str) -> ModalResult<()> {
    multispace0.void().parse_next(input)
}

fn expected(description: &'static str) -> StrContext {
    StrContext::Expected(StrContextValue::Description(description))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn statements(input: &str) -> Vec<Statement<'_>> {
        parse_program(input)
            .unwrap_or_else(|err| panic!("{input:?}: {err}"))
            .statements
    }

    fn args(input: &str) -> Vec<Expr<'_>> {
        match statements(input).as_slice() {
            [Statement::Call(call)] => call.args.clone(),
            other => panic!("expected a single call, got {other:?}"),
        }
    }

    fn call<'s>(name: &'s str, args: Vec<Expr<'s>>) -> Statement<'s> {
        Statement::Call(Call { name, args })
    }

    #[test]
    fn empty_program() {
        assert!(statements("").is_empty());
        assert!(statements(" \n\t ").is_empty());
    }

    #[test]
    fn literals() {
        assert_eq!(
            args(r#"op 1 -2 03 3.5 -0.25 true false "a b" 'c"d' "it's" name-1 TRUE"#),
            vec![
                Expr::Literal(Literal::Integer("1")),
                Expr::Literal(Literal::Integer("-2")),
                Expr::Literal(Literal::Integer("03")),
                Expr::Literal(Literal::Decimal("3.5")),
                Expr::Literal(Literal::Decimal("-0.25")),
                Expr::Literal(Literal::Bool(true)),
                Expr::Literal(Literal::Bool(false)),
                Expr::Literal(Literal::String("a b")),
                Expr::Literal(Literal::String("c\"d")),
                Expr::Literal(Literal::String("it's")),
                Expr::Literal(Literal::Ident("name-1")),
                Expr::Literal(Literal::Ident("TRUE")),
            ]
        );
    }

    #[test]
    fn strings_are_raw() {
        assert_eq!(
            args(r#"op "C:\Users\a b\in.jpg" '\n'"#),
            vec![
                Expr::Literal(Literal::String(r"C:\Users\a b\in.jpg")),
                Expr::Literal(Literal::String(r"\n")),
            ]
        );
    }

    #[test]
    fn value_calls() {
        assert_eq!(
            args("op rgba(1, 2,3 , 4) coord( -1 ,2 ) size() font('f.ttf')"),
            vec![
                Expr::Call(Call {
                    name: "rgba",
                    args: vec![
                        Literal::Integer("1"),
                        Literal::Integer("2"),
                        Literal::Integer("3"),
                        Literal::Integer("4"),
                    ],
                }),
                Expr::Call(Call {
                    name: "coord",
                    args: vec![Literal::Integer("-1"), Literal::Integer("2")],
                }),
                Expr::Call(Call {
                    name: "size",
                    args: vec![],
                }),
                Expr::Call(Call {
                    name: "font",
                    args: vec![Literal::String("f.ttf")],
                }),
            ]
        );
    }

    #[test]
    fn pipe_is_a_plain_separator() {
        assert_eq!(args("op 1 | 2 3 | 4"), args("op 1 2 3 4"));
        assert_eq!(args("op 1 | 2 3 4"), args("op 1 2 3 4"));
    }

    #[test]
    fn set_and_del() {
        assert_eq!(
            statements(
                "set sampling-filter nearest; SET preserve-aspect-ratio true; del sampling-filter"
            ),
            vec![
                Statement::Set {
                    option: "sampling-filter",
                    value: Expr::Literal(Literal::Ident("nearest")),
                },
                Statement::Set {
                    option: "preserve-aspect-ratio",
                    value: Expr::Literal(Literal::Bool(true)),
                },
                Statement::Del {
                    option: "sampling-filter"
                },
            ]
        );
    }

    #[test]
    fn keywords_are_whole_words() {
        assert_eq!(
            statements("settings 1"),
            vec![call("settings", vec![Expr::Literal(Literal::Integer("1"))])]
        );
    }

    #[test]
    fn statement_separators() {
        let expected = vec![call("a", vec![]), call("b", vec![])];

        for input in ["a;b", "a; b", "a ;b;", " a ;\n\tb ;\r\n", "a;\n\nb   "] {
            assert_eq!(statements(input), expected, "{input:?}");
        }
    }

    ide!();

    #[parameterized(
        input = {
            ";",
            "15",
            "blur 15.",
            "blur 15.;",
            "blur 15.7.",
            "blur 1;;",
            "blur 1 blur 2;;",
            "blur 1 |",
            "blur 1 | ",
            "blur 1|2",
            "blur 1 |2",
            "blur 1| 2",
            "blur | 1",
            "blur\t1",
            "blur\n1",
            "blur 1\nblur 2",
            "blur 'x",
            "blur \"x'",
            "blur -x",
            "blur_x",
            "op rgba(1, 2",
            "op rgba(1, 2,)",
            "op rgba(size(1))",
            "op (1, 2)",
            "op 1, 2",
            "set",
            "set x",
            "set x;",
            "del",
            "del;",
        }
    )]
    fn rejects(input: &str) {
        assert!(
            parse_program(input).is_err(),
            "{input:?} should be rejected"
        );
    }

    #[test]
    fn error_position() {
        let err = parse_program("blur 1;\nblur 15.").unwrap_err();
        assert_eq!(
            err.to_string(),
            "syntax error at line 2, column 9: expected a digit after '.', found end of input"
        );
    }

    #[test]
    fn missing_separator_error() {
        let err = parse_program("blur 1\nblur 2").unwrap_err();
        assert_eq!(
            err.to_string(),
            "syntax error at line 2, column 1: expected ';' or end of input, found 'b'"
        );
    }

    #[test]
    fn expr_from_word() {
        assert_eq!(
            parse_expr("coord(0, 1)").unwrap(),
            Expr::Call(Call {
                name: "coord",
                args: vec![Literal::Integer("0"), Literal::Integer("1")],
            })
        );
        assert!(parse_expr("my text").is_err());
        assert!(parse_expr("").is_err());
    }
}
