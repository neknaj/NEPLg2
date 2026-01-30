mod harness;

use harness::run_main_capture_stdout;
use std::fs;
use std::path::PathBuf;

#[test]
fn stdout_concat_is_stable() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("stdout.nepl");
    let src = fs::read_to_string(path).expect("read stdout fixture");
    let out = run_main_capture_stdout(&src);
    assert_eq!(out, "a:12:b");
}

#[test]
fn println_appends_newline() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("println.nepl");
    let src = fs::read_to_string(path).expect("read println fixture");
    let out = run_main_capture_stdout(&src);
    assert_eq!(out, "ab\nc");
}

#[test]
fn println_i32_appends_newline() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("println_i32.nepl");
    let src = fs::read_to_string(path).expect("read println_i32 fixture");
    let out = run_main_capture_stdout(&src);
    assert_eq!(out, "12\n3");
}
