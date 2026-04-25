use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use tempfile::TempDir;

fn cli_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_nepl-cli"))
}

fn ok_source() -> &'static str {
    "#entry main\n#indent 4\n\nfn main <()->i32> ():\n    0\n"
}

fn write_ok_source(dir: &TempDir, name: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, ok_source()).expect("write source");
    path
}

fn output_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        output_text(&output.stdout),
        output_text(&output.stderr)
    );
}

fn assert_clean_stderr(output: &Output) {
    let stderr = output_text(&output.stderr);
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
    assert!(!stderr.contains("DEBUG:"), "debug output leaked:\n{stderr}");
}

#[test]
fn check_success_keeps_stderr_empty_without_verbose() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source = write_ok_source(&temp, "ok.nepl");

    let output = cli_command()
        .args(["--check", "-i"])
        .arg(&source)
        .args(["--target", "core"])
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert_clean_stderr(&output);
    assert!(output_text(&output.stdout).contains("Check successful"));
}

#[test]
fn compile_success_keeps_output_channels_clean_without_verbose() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source = write_ok_source(&temp, "ok.nepl");
    let output_base = temp.path().join("out");

    let output = cli_command()
        .arg("-i")
        .arg(&source)
        .args(["--target", "core", "-o"])
        .arg(&output_base)
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert_clean_stderr(&output);
    assert!(
        output.stdout.is_empty(),
        "unexpected stdout:\n{}",
        output_text(&output.stdout)
    );
    assert!(output_base.with_extension("wasm").exists());
}

#[test]
fn verbose_check_still_exposes_cli_debug_logs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source = write_ok_source(&temp, "ok.nepl");

    let output = cli_command()
        .args(["--verbose", "--check", "-i"])
        .arg(&source)
        .args(["--target", "core"])
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    let stderr = output_text(&output.stderr);
    assert!(
        stderr.contains("DEBUG: Creating Loader for path:"),
        "verbose stderr did not contain CLI loader debug:\n{stderr}"
    );
    assert!(
        stderr.contains("[Loader]"),
        "verbose stderr did not contain core loader debug:\n{stderr}"
    );
}

#[test]
fn test_subcommand_hides_internal_debug_without_verbose() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_ok_source(&temp, "case.nepl");

    let output = cli_command()
        .arg("test")
        .arg("--dir")
        .arg(temp.path())
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert_clean_stderr(&output);
    let stdout = output_text(&output.stdout);
    assert!(
        !stdout.contains("[nepl-cli]"),
        "internal debug output leaked to stdout:\n{stdout}"
    );
}
