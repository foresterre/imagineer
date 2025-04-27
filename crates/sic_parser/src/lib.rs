#![deny(clippy::all)]
#![allow(clippy::unnecessary_wraps, clippy::upper_case_acronyms)]

#[macro_use]
extern crate pest_derive;

#[cfg(test)]
#[macro_use]
extern crate parameterized;

use pest::Parser;

use crate::errors::SicParserError;
use crate::rule_parser::parse_image_operations;
use sic_image_engine::engine::Instr;

pub mod errors;
pub mod named_value;
pub mod rule_parser;
pub mod value_parser;

const PARSER_RULE: Rule = Rule::main;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct SICParser;

pub fn parse_script(script: &str) -> Result<Vec<Instr>, SicParserError> {
    let parsed_script = SICParser::parse(PARSER_RULE, script);

    parsed_script
        .map_err(|err| SicParserError::PestGrammarError(err.to_string()))
        .and_then(parse_image_operations)
}

#[cfg(test)]
mod tests {
    use sic_image_engine::ImgOp;
    use sic_image_engine::engine::Instr;

    use super::*;

    #[test]
    fn test_too_many_args() {
        let input = "blur 15 28;";
        let parsed = SICParser::parse(Rule::main, input);

        // Manually constructing a pest::Error::ParsingError is a hell, because of Position, which,
        // except for the from_start() method is private.
        assert!(parsed.is_err());
    }

    #[test]
    fn test_parsed() {
        let input = "blur 15; flip-vertical";
        let parsed = parse_script(input);

        assert!(parsed.is_ok());

        assert_eq!(
            parsed.unwrap(),
            vec![
                Instr::Operation(ImgOp::Blur(15.0)),
                Instr::Operation(ImgOp::FlipVertical)
            ]
        );
    }

    #[test]
    fn test_parsed_fail() {
        let input = "blur 15.7.; flipv";
        let parsed = parse_script(input);

        assert!(parsed.is_err());
    }
}
