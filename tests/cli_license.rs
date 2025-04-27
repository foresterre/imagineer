#![deny(clippy::all)]

use std::process::{Command, Output};

fn run_license_command() -> Output {
    Command::new("cargo")
        .args(["run", "--", "--license"])
        .output()
        .expect("Running test failed")
}

// This test just ensures the license is included within the binary.
#[test]
fn cli_license_starts_with() {
    let res = run_license_command();

    let begin_text = "imagineer image tools license:";

    assert!(res.status.success());
    assert!(
        std::str::from_utf8(&res.stdout)
            .unwrap()
            .starts_with(begin_text)
    );
}
