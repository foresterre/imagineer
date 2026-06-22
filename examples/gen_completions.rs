extern crate imagineer;

const ABOUT: &str = include_str!("../resources/help-pages/about.txt");
const HELP_OPERATIONS_AVAILABLE: &str =
    include_str!("../resources/help-pages/image_operations.txt");
const VERSION: &str = env!("CARGO_PKG_VERSION");

use clap_complete::{Shell, generate_to};
use imagineer::cli::app::create_app;

fn main() {
    let mut cli = create_app(VERSION, ABOUT, HELP_OPERATIONS_AVAILABLE);

    let program_name = option_env!("SIC_COMPLETIONS_APP_NAME").unwrap_or("ig");

    let out = option_env!("SIC_COMPLETIONS_OUT_DIR")
        .map(From::from)
        .or_else(|| std::env::args_os().nth(1))
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("Please supply a valid output folder")
                .into_os_string()
        });

    println!("using output folder '{}'", out.to_string_lossy());

    for shell in [
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Elvish,
    ] {
        generate_to(shell, &mut cli, program_name, &out)
            .unwrap_or_else(|_| panic!("Could not generate completions for shell {}", shell));
        println!("generated completions for: {}", shell);
    }
}
