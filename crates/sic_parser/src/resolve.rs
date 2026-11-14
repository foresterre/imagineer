use crate::ast::{Program, Statement};
use crate::builtins;
use crate::errors::ResolveError;
use sic_image_engine::engine::Instr;

pub fn resolve(program: &Program<'_>) -> Result<Vec<Instr>, ResolveError> {
    program.statements.iter().map(statement).collect()
}

fn statement(statement: &Statement<'_>) -> Result<Instr, ResolveError> {
    match statement {
        Statement::Call(call) => builtins::operation(call).map(Instr::Operation),
        Statement::Set { option, value } => builtins::set_option(option, value),
        Statement::Del { option } => builtins::del_option(option),
    }
}
