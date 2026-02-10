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

#[test]
fn stdout_japanese_utf8() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("stdout_japanese.nepl");
    let src = fs::read_to_string(path).expect("read stdout_japanese fixture");
    let out = run_main_capture_stdout(&src);
    assert_eq!(out, "こんにちは世界!\n");
}

#[test]
fn stdout_ansi_escape() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("stdout_ansi.nepl");
    let src = fs::read_to_string(path).expect("read stdout_ansi fixture");
    let out = run_main_capture_stdout(&src);
    assert_eq!(out, "\x1b[31mred\x1b[0m\n");
}

#[test]
fn stdout_ansi_helpers() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("stdout_color.nepl");
    let src = fs::read_to_string(path).expect("read stdout_color fixture");
    let out = run_main_capture_stdout(&src);
    assert_eq!(out, "\x1b[31mred\x1b[0m \x1b[32mgreen\x1b[0m\n");
}
