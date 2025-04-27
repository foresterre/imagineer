#![deny(clippy::all)]

#[macro_use]
pub mod common;

use common::{SicTestCommandBuilder, setup_output_path};
use std::ffi::OsStr;
use std::path::Path;

#[test]
fn keep_format() {
    const OUT: &str = "globtest/unmodified_format";

    let mut process = SicTestCommandBuilder::new()
        .glob_input_from_resources("*.png")
        .glob_output_in_target(OUT)
        .spawn_child();

    let exit_status = process.wait().unwrap();

    assert!(exit_status.success());

    let check_path = setup_output_path(OUT);
    assert!(count_files_with_ext_in_folder(check_path, "png") >= 12)
}

#[test]
fn modify_format() {
    const OUT: &str = "globtest/modified_format_and_ext/";

    let mut process = SicTestCommandBuilder::new()
        .glob_input_from_resources("*.png")
        .glob_output_in_target(OUT)
        .with_args(["--output-format", "jpg"])
        .spawn_child();

    let exit_status = process.wait().unwrap();
    assert!(exit_status.success());

    let check_path = setup_output_path(OUT);
    assert!(count_files_with_ext_in_folder(check_path, "jpg") >= 12)
}

fn count_files_with_ext_in_folder<P: AsRef<Path>, S: AsRef<OsStr>>(path: P, ext: S) -> usize {
    std::fs::read_dir(path)
        .unwrap()
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                if let Some(found_ext) = entry.path().extension() {
                    if found_ext == ext.as_ref() {
                        return Some(found_ext.to_os_string());
                    }
                }
            }
            None
        })
        .count()
}
