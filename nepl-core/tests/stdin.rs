mod harness;

use harness::run_main_capture_stdout_with_stdin;
use std::fs;
use std::path::PathBuf;

fn load_fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    fs::read_to_string(path).expect("read stdin fixture")
}

#[test]
fn stdin_echo_ascii() {
    let src = load_fixture("stdin_echo.nepl");
    let out = run_main_capture_stdout_with_stdin(&src, b"1 2 +\n");
    assert_eq!(out, "1 2 +\n");
}

#[test]
fn stdin_echo_japanese() {
    let src = load_fixture("stdin_echo.nepl");
    let input = "こんにちは\n";
    let out = run_main_capture_stdout_with_stdin(&src, input.as_bytes());
    assert_eq!(out, input);
}
