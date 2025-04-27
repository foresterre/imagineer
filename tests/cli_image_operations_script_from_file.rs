#![deny(clippy::all)]

#[macro_use]
pub mod common;

use common::{DEFAULT_IN, SicTestCommandBuilder};

#[test]
fn script_from_file_ok() {
    let script_file = &[env!("CARGO_MANIFEST_DIR"), "/resources/script/emboss.sic"].concat();

    let mut process = SicTestCommandBuilder::new()
        .input_from_resources(DEFAULT_IN)
        .output_in_target("cio_script_from_file__ok.png")
        .with_args(["--operations-script", script_file])
        .spawn_child();

    let result = process.wait().unwrap();
    assert!(result.success());
}

#[test]
fn script_from_file_where_file_not_found() {
    let script = r#""_.sic""#;

    let mut process = SicTestCommandBuilder::new()
        .input_from_resources(DEFAULT_IN)
        .output_in_target("cio_script_from_file__file_not_found.png")
        .with_args(["--operations-script", script])
        .spawn_child();

    let result = process.wait();
    assert!(result.is_ok());
    assert_not!(result.unwrap().success());
}

#[test]
fn script_from_file_conflicting_args() {
    let script_file = &[env!("CARGO_MANIFEST_DIR"), "/resources/script/emboss.sic"].concat();
    let script = r#""blur 1""#;

    let mut process = SicTestCommandBuilder::new()
        .input_from_resources(DEFAULT_IN)
        .output_in_target("cio_script_from_file__conflicted_args.png")
        .with_args([
            "--operations-script",
            script_file,
            "--apply-operations",
            script,
        ])
        .spawn_child();

    let result = process.wait().unwrap();
    assert_not!(result.success());
}
