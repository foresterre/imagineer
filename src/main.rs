#![deny(clippy::all)]

use imagineer::cli::app;
use imagineer::cli::app::arg_names::{ARG_DEP_LICENSES, ARG_LICENSE};
use imagineer::cli::app::build_app_config;
use imagineer::cli::config::InputOutputMode;
use imagineer::cli::license::LicenseTexts;
use imagineer::cli::pipeline::{run_display_licenses, run_with_devices};

const LICENSE_SELF: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/LICENSE-MIT"));

const ABOUT: &str = include_str!("../resources/help-pages/about.txt");
const HELP_OPERATIONS_AVAILABLE: &str =
    include_str!("../resources/help-pages/image_operations.txt");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> anyhow::Result<()> {
    let app = app::create_app(VERSION, ABOUT, HELP_OPERATIONS_AVAILABLE);
    let matches = app.get_matches();

    let license_display = matches.is_present(ARG_LICENSE) || matches.is_present(ARG_DEP_LICENSES);

    let configuration = build_app_config(&matches)?;

    if license_display {
        run_display_licenses(&configuration, &LicenseTexts::new(LICENSE_SELF))
    } else {
        let io_device = InputOutputMode::try_from_matches(&matches)?;
        run_with_devices(io_device, &configuration)
    }
}
