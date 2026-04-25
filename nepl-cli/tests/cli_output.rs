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

fn stderr_fd_write_source() -> &'static str {
    r#"#target wasi
#entry main
#indent 4

#import "core/mem" as *
#extern "wasi_snapshot_preview1" "fd_write" fn fd_write <(i32,i32,i32,i32)*>i32>

fn main <()*>i32> ():
    store_u8 1024 101;
    store_u8 1025 114;
    store_u8 1026 114;
    store_i32 1000 1024;
    store_i32 1004 3;
    store_i32 1008 0;
    let errno <i32> fd_write 2 1000 1 1008;
    if:
        cond:
            eq errno 0
        then:
            if:
                cond:
                    eq load_i32 1008 3
                then:
                    0
                else:
                    98
        else:
            99
"#
}

fn write_source(dir: &TempDir, name: &str, source: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, source).expect("write source");
    path
}

fn write_ok_source(dir: &TempDir, name: &str) -> PathBuf {
    write_source(dir, name, ok_source())
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

#[test]
fn run_wasi_fd_write_supports_stderr() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source = write_source(&temp, "stderr.nepl", stderr_fd_write_source());

    let output = cli_command()
        .args(["--run", "-i"])
        .arg(&source)
        .args(["--target", "wasi"])
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert!(
        output.stdout.is_empty(),
        "unexpected stdout:\n{}",
        output_text(&output.stdout)
    );
    assert_eq!(output_text(&output.stderr), "err");
}
