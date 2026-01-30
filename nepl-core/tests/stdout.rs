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
