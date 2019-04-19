#[macro_use]
extern crate pest_derive;

pub mod app_cli;
pub mod app_custom_config;
pub mod app_run;
pub mod help;
pub mod parser;
pub mod sic_processor;

pub fn get_tool_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}
