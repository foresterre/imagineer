extern crate imagineer;

const ABOUT: &str = include_str!("../resources/help-pages/about.txt");
const HELP_OPERATIONS_AVAILABLE: &str =
    include_str!("../resources/help-pages/image_operations.txt");
const VERSION: &str = env!("CARGO_PKG_VERSION");

use imagineer::cli::app::create_app;
use std::str::FromStr;

fn main() {
    let mut cli = create_app(VERSION, ABOUT, HELP_OPERATIONS_AVAILABLE);

    let program_name = option_env!("SIC_COMPLETIONS_APP_NAME").unwrap_or("imagineer");

    let out = option_env!("SIC_COMPLETIONS_OUT_DIR")
        .map(From::from)
        .or_else(|| std::env::args_os().nth(1))
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("Please supply a valid output folder")
                .into_os_string()
        });

    println!("using output folder '{}'", out.to_string_lossy());

    for variant in clap::Shell::variants().iter() {
        cli.gen_completions(
            program_name,
            clap::Shell::from_str(variant)
                .unwrap_or_else(|_| panic!("Could not generate completions for shell {}", variant)),
            &out,
        );
        println!("generated completions for: {}", variant);
    }
}
