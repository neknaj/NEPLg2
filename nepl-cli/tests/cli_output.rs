use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

fn cli_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_nepl-cli"));
    command.env_remove("NEPL_STDLIB_ROOT");
    command
}

fn ok_source() -> &'static str {
    "#entry main\n#indent 4\n\nfn main <()->i32> ():\n    0\n"
}

fn llvm_source() -> &'static str {
    "#target llvm\n#entry main\n#indent 4\n\nfn main <()->i32> ():\n    0\n"
}

fn stderr_fd_write_source() -> &'static str {
    r#"#target std
#entry main
#indent 4

#import "core/result" as *
#import "std/stdio" as *

fn main <()*>i32> ():
    match stdio_write_stderr_str_result "err":
        Result::Ok _:
            0
        Result::Err _e:
            99
"#
}

fn fs_read_allowed_source() -> &'static str {
    r#"#target std
#entry main
#indent 4

#import "alloc/string" as *
#import "core/result" as *
#import "std/fs" as *

fn main <()*>i32> ():
    match fs_read_to_string "allowed.txt":
        Result::Ok text:
            if str_eq text "sandbox-ok" 0 91
        Result::Err e:
            fs_error_to_errno e
"#
}

fn fs_read_parent_escape_source() -> &'static str {
    r#"#target std
#entry main
#indent 4

#import "core/math" as *
#import "core/result" as *
#import "std/fs" as *

fn main <()*>i32> ():
    match fs_read_to_string "../outside.txt":
        Result::Ok _content:
            95
        Result::Err e:
            let errno <i32> fs_error_to_errno e;
            if eq errno 76 0 95
"#
}

fn fs_read_missing_source() -> &'static str {
    r#"#target std
#entry main
#indent 4

#import "core/math" as *
#import "core/result" as *
#import "std/fs" as *

fn main <()*>i32> ():
    match fs_read_to_string "missing.txt":
        Result::Ok _content:
            94
        Result::Err e:
            let errno <i32> fs_error_to_errno e;
            if eq errno 44 0 94
"#
}

fn fs_write_create_source() -> &'static str {
    r#"#target std
#entry main
#indent 4

#import "core/result" as *
#import "std/fs" as *

fn main <()*>i32> ():
    match fs_write_to_string "out.txt" "made":
        Result::Ok _:
            0
        Result::Err e:
            fs_error_to_errno e
"#
}

fn fs_write_parent_escape_source() -> &'static str {
    r#"#target std
#entry main
#indent 4

#import "core/math" as *
#import "core/result" as *
#import "std/fs" as *

fn main <()*>i32> ():
    match fs_write_to_string "../outside.txt" "changed":
        Result::Ok _:
            95
        Result::Err e:
            let errno <i32> fs_error_to_errno e;
            if eq errno 76 0 95
"#
}

fn fs_path_filestat_source() -> &'static str {
    r#"#target std
#entry main
#indent 4

#import "core/math" as *
#import "core/result" as *
#import "std/fs" as *

fn main <()*>i32> ():
    let ok_dir <bool> fs_is_dir "dir";
    let ok_file <bool> fs_is_file "dir/alpha.nepl";
    let ok_escape <bool> match fs_path_filetype "../outside.txt":
        Result::Ok _filetype:
            false
        Result::Err e:
            let errno <i32> fs_error_to_errno e;
            eq errno 76
    if and ok_dir and ok_file ok_escape 0 91
"#
}

fn fs_readdir_source() -> &'static str {
    r#"#target std
#entry main
#indent 4

#import "alloc/collections/vec" as v
#import "alloc/string" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/fs" as *

fn main <()*>i32> ():
    match fs_read_dir "dir":
        Result::Err e:
            fs_error_to_errno e
        Result::Ok entries:
            let ok_len <bool> eq v::len<str> &entries 3;
            let ok0 <bool> match v::get<str> &entries 0:
                Option::Some entry0:
                    str_eq entry0 "alpha.nepl"
                Option::None:
                    false
            let ok1 <bool> match v::get<str> &entries 1:
                Option::Some entry1:
                    str_eq entry1 "beta.n.md"
                Option::None:
                    false
            let ok2 <bool> match v::get<str> &entries 2:
                Option::Some entry2:
                    str_eq entry2 "zeta.txt"
                Option::None:
                    false
            v::free<str> entries;
            if and ok_len and ok0 and ok1 ok2 0 92
"#
}

fn fs_std_read_dir_source() -> &'static str {
    r#"#target std
#entry main
#indent 4

#import "std/fs" as *
#import "core/result" as *
#import "core/math" as *
#import "core/option" as *
#import "alloc/collections/vec" as v
#import "alloc/string" as *

fn main <()*>i32> ():
    match fs_read_dir "dir":
        Result::Err e:
            let errno <i32> fs_error_to_errno e;
            add 90 errno
        Result::Ok entries:
            let ok_len <bool> eq v::len<str> &entries 3;
            let ok0 <bool> match v::get<str> &entries 0:
                Option::Some entry0:
                    str_eq entry0 "alpha.nepl"
                Option::None:
                    false
            let ok1 <bool> match v::get<str> &entries 1:
                Option::Some entry1:
                    str_eq entry1 "beta.n.md"
                Option::None:
                    false
            let ok2 <bool> match v::get<str> &entries 2:
                Option::Some entry2:
                    str_eq entry2 "zeta.txt"
                Option::None:
                    false
            v::free<str> entries;
            if and ok_len and ok0 and ok1 ok2 0 93
"#
}

fn extra_answer_source() -> &'static str {
    "#indent 4\n\npub fn answer <()->i32> ():\n    42\n"
}

fn source_using_extra_stdlib() -> &'static str {
    "#entry main\n#indent 4\n\n#import \"extra/answer\" as *\n\nfn main <()->i32> ():\n    answer\n"
}

fn test_source_using_extra_stdlib() -> &'static str {
    "#entry main\n#indent 4\n\n#import \"extra/answer\" as *\n#import \"core/math\" as *\n\nfn main <()*>i32> ():\n    if eq answer 42 0 1\n"
}

fn write_source_at(dir: &Path, name: &str, source: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, source).expect("write source");
    path
}

fn write_source(dir: &TempDir, name: &str, source: &str) -> PathBuf {
    write_source_at(dir.path(), name, source)
}

fn write_ok_source(dir: &TempDir, name: &str) -> PathBuf {
    write_source(dir, name, ok_source())
}

fn write_fake_clang(dir: &TempDir, version: &str, triple: &str) -> PathBuf {
    #[cfg(windows)]
    {
        let path = dir.path().join("fake-clang.cmd");
        let script = format!(
            "@echo off\r\nif \"%1\"==\"--version\" (\r\n  echo clang version {version}\r\n  exit /b 0\r\n)\r\nif \"%1\"==\"-dumpmachine\" (\r\n  echo {triple}\r\n  exit /b 0\r\n)\r\nexit /b 2\r\n"
        );
        fs::write(&path, script).expect("write fake clang");
        path
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = dir.path().join("fake-clang");
        let script = format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo 'clang version {version}'\n  exit 0\nfi\nif [ \"$1\" = \"-dumpmachine\" ]; then\n  echo '{triple}'\n  exit 0\nfi\nexit 2\n"
        );
        fs::write(&path, script).expect("write fake clang");
        let mut perms = fs::metadata(&path)
            .expect("fake clang metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("make fake clang executable");
        path
    }

    #[cfg(all(not(unix), not(windows)))]
    {
        let path = dir.path().join("fake-clang");
        let script = format!(
            "if [ \"$1\" = \"--version\" ]; then\n  echo 'clang version {version}'\n  exit 0\nfi\nif [ \"$1\" = \"-dumpmachine\" ]; then\n  echo '{triple}'\n  exit 0\nfi\nexit 2\n"
        );
        fs::write(&path, script).expect("write fake clang");
        path
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create destination dir");
    for entry in fs::read_dir(src).expect("read source dir") {
        let entry = entry.expect("read dir entry");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let file_type = entry.file_type().expect("read file type");
        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path);
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path).expect("copy file");
        }
    }
}

fn custom_stdlib_with_extra(dir: &TempDir) -> PathBuf {
    let custom = dir.path().join("custom_stdlib");
    let workspace_stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib");
    copy_dir_recursive(&workspace_stdlib, &custom);
    let extra_dir = custom.join("extra");
    fs::create_dir_all(&extra_dir).expect("create extra stdlib dir");
    fs::write(extra_dir.join("answer.nepl"), extra_answer_source()).expect("write extra module");
    custom
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

fn assert_no_internal_debug_stderr(output: &Output) {
    let stderr = output_text(&output.stderr);
    assert!(!stderr.contains("DEBUG:"), "debug output leaked:\n{stderr}");
    assert!(
        !stderr.contains("[Loader]"),
        "core loader debug output leaked:\n{stderr}"
    );
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
    assert_no_internal_debug_stderr(&output);
    assert!(output_text(&output.stdout).contains("Check successful"));
}

#[test]
fn check_uses_explicit_stdlib_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let custom_stdlib = custom_stdlib_with_extra(&temp);
    let source = write_source(&temp, "uses_extra.nepl", source_using_extra_stdlib());

    let output = cli_command()
        .args(["--check", "-i"])
        .arg(&source)
        .arg("--stdlib-root")
        .arg(&custom_stdlib)
        .args(["--target", "core"])
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert_no_internal_debug_stderr(&output);
    assert!(output_text(&output.stdout).contains("Check successful"));
}

#[test]
fn check_uses_env_stdlib_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let custom_stdlib = custom_stdlib_with_extra(&temp);
    let source = write_source(&temp, "uses_extra_env.nepl", source_using_extra_stdlib());

    let output = cli_command()
        .env("NEPL_STDLIB_ROOT", &custom_stdlib)
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
fn test_subcommand_uses_explicit_stdlib_root() {
    let temp = tempfile::tempdir().expect("tempdir");
    let custom_stdlib = custom_stdlib_with_extra(&temp);
    let tests_dir = temp.path().join("cases");
    fs::create_dir(&tests_dir).expect("create tests dir");
    write_source_at(
        &tests_dir,
        "uses_extra.nepl",
        test_source_using_extra_stdlib(),
    );

    let output = cli_command()
        .arg("--stdlib-root")
        .arg(&custom_stdlib)
        .arg("test")
        .arg("--dir")
        .arg(&tests_dir)
        .output()
        .expect("run nepl-cli test");

    assert_success(&output);
    assert_no_internal_debug_stderr(&output);
    assert!(
        output_text(&output.stdout).contains("uses_extra.nepl"),
        "test output did not include case name:\n{}",
        output_text(&output.stdout)
    );
}

#[test]
fn invalid_explicit_stdlib_root_reports_tried_candidate() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source = write_ok_source(&temp, "ok.nepl");
    let missing = temp.path().join("missing_stdlib");

    let output = cli_command()
        .args(["--check", "-i"])
        .arg(&source)
        .arg("--stdlib-root")
        .arg(&missing)
        .args(["--target", "core"])
        .output()
        .expect("run nepl-cli");

    assert!(
        !output.status.success(),
        "unexpected success\nstdout:\n{}\nstderr:\n{}",
        output_text(&output.stdout),
        output_text(&output.stderr)
    );
    let stderr = output_text(&output.stderr);
    assert!(
        stderr.contains("stdlib directory not found. Tried:"),
        "missing diagnostic header:\n{stderr}"
    );
    assert!(
        stderr.contains("--stdlib-root") && stderr.contains("missing_stdlib"),
        "missing tried candidate:\n{stderr}"
    );
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
fn llvm_target_default_accepts_available_clang_without_fixed_linux_version() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source = write_source(&temp, "llvm.nepl", llvm_source());
    let output_base = temp.path().join("llvm_out");
    let fake_clang = write_fake_clang(&temp, "17.0.6", "x86_64-pc-windows-msvc");

    let output = cli_command()
        .env("NEPL_LLVM_CLANG_BIN", &fake_clang)
        .env_remove("NEPL_LLVM_STRICT")
        .env_remove("NEPL_LLVM_CLANG_VERSION")
        .env_remove("NEPL_LLVM_CLANG_VERSION_PREFIX")
        .env_remove("NEPL_LLVM_REQUIRED_HOST_OS")
        .env_remove("NEPL_LLVM_REQUIRE_LINUX")
        .env_remove("NEPL_LLVM_TRIPLE_CONTAINS")
        .arg("-i")
        .arg(&source)
        .args(["--target", "llvm", "-o"])
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
    assert!(output_base.with_extension("ll").exists());
}

#[test]
fn lib_mode_fails_until_artifact_contract_exists() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source = write_ok_source(&temp, "lib.nepl");
    let output_base = temp.path().join("lib_out");

    let output = cli_command()
        .arg("--lib")
        .arg("-i")
        .arg(&source)
        .args(["--target", "core", "-o"])
        .arg(&output_base)
        .output()
        .expect("run nepl-cli");

    assert!(
        !output.status.success(),
        "unexpected success\nstdout:\n{}\nstderr:\n{}",
        output_text(&output.stdout),
        output_text(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "unexpected stdout:\n{}",
        output_text(&output.stdout)
    );
    let stderr = output_text(&output.stderr);
    assert!(
        stderr.contains("--lib is not supported yet: library artifact contract is not implemented"),
        "missing unsupported diagnostic:\n{stderr}"
    );
    assert!(
        !stderr.contains("placeholder pipeline"),
        "placeholder warning leaked:\n{stderr}"
    );
    assert!(
        !output_base.with_extension("wasm").exists(),
        "--lib should fail before writing a wasm artifact"
    );
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
    assert_no_internal_debug_stderr(&output);
    assert!(
        output.stdout.is_empty(),
        "unexpected stdout:\n{}",
        output_text(&output.stdout)
    );
    assert!(
        output_text(&output.stderr).ends_with("err"),
        "program stderr did not end with expected payload:\n{}",
        output_text(&output.stderr)
    );
}

#[test]
fn run_wasi_path_open_reads_preopen_relative_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("allowed.txt"), "sandbox-ok").expect("write allowed");
    let source = write_source(&temp, "read_allowed.nepl", fs_read_allowed_source());

    let output = cli_command()
        .current_dir(temp.path())
        .args(["--run", "-i"])
        .arg(&source)
        .args(["--target", "wasi"])
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert_no_internal_debug_stderr(&output);
    assert!(
        output.stdout.is_empty(),
        "unexpected stdout:\n{}",
        output_text(&output.stdout)
    );
}

#[test]
fn run_wasi_path_open_rejects_parent_escape() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("root");
    fs::create_dir(&root).expect("create preopen root");
    fs::write(temp.path().join("outside.txt"), "secret").expect("write outside");
    let source = write_source_at(&root, "parent_escape.nepl", fs_read_parent_escape_source());

    let output = cli_command()
        .current_dir(&root)
        .args(["--run", "-i"])
        .arg(&source)
        .args(["--target", "wasi"])
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert_no_internal_debug_stderr(&output);
    assert!(
        output.stdout.is_empty(),
        "unexpected stdout:\n{}",
        output_text(&output.stdout)
    );
}

#[test]
fn run_wasi_path_open_reports_missing_file_noent() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source = write_source(&temp, "missing.nepl", fs_read_missing_source());

    let output = cli_command()
        .current_dir(temp.path())
        .args(["--run", "-i"])
        .arg(&source)
        .args(["--target", "wasi"])
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert_no_internal_debug_stderr(&output);
    assert!(
        output.stdout.is_empty(),
        "unexpected stdout:\n{}",
        output_text(&output.stdout)
    );
}

#[test]
fn run_wasi_path_open_creates_truncates_and_writes_preopen_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("out.txt"), "old-longer").expect("write old output");
    let source = write_source(&temp, "write_file.nepl", fs_write_create_source());

    let output = cli_command()
        .current_dir(temp.path())
        .args(["--run", "-i"])
        .arg(&source)
        .args(["--target", "wasi"])
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert_no_internal_debug_stderr(&output);
    assert!(
        output.stdout.is_empty(),
        "unexpected stdout:\n{}",
        output_text(&output.stdout)
    );
    let written = fs::read_to_string(temp.path().join("out.txt")).expect("read output");
    assert_eq!(written, "made");
}

#[test]
fn run_wasi_path_open_rejects_write_parent_escape() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("root");
    fs::create_dir(&root).expect("create preopen root");
    fs::write(temp.path().join("outside.txt"), "secret").expect("write outside");
    let source = write_source_at(
        &root,
        "write_parent_escape.nepl",
        fs_write_parent_escape_source(),
    );

    let output = cli_command()
        .current_dir(&root)
        .args(["--run", "-i"])
        .arg(&source)
        .args(["--target", "wasi"])
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert_no_internal_debug_stderr(&output);
    assert!(
        output.stdout.is_empty(),
        "unexpected stdout:\n{}",
        output_text(&output.stdout)
    );
    let outside = fs::read_to_string(temp.path().join("outside.txt")).expect("read outside");
    assert_eq!(outside, "secret");
}

#[test]
fn run_wasi_path_filestat_get_reports_file_kinds_and_rejects_escape() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("root");
    let dir = root.join("dir");
    fs::create_dir_all(&dir).expect("create fixture dir");
    fs::write(dir.join("alpha.nepl"), "module").expect("write file");
    fs::write(temp.path().join("outside.txt"), "secret").expect("write outside");
    let source = write_source_at(&root, "stat.nepl", fs_path_filestat_source());

    let output = cli_command()
        .current_dir(&root)
        .args(["--run", "-i"])
        .arg(&source)
        .args(["--target", "wasi"])
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert_no_internal_debug_stderr(&output);
    assert!(
        output.stdout.is_empty(),
        "unexpected stdout:\n{}",
        output_text(&output.stdout)
    );
}

#[test]
fn run_wasi_fd_readdir_returns_stable_directory_entries() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path().join("dir");
    fs::create_dir(&dir).expect("create fixture dir");
    fs::write(dir.join("zeta.txt"), "z").expect("write zeta");
    fs::write(dir.join("alpha.nepl"), "a").expect("write alpha");
    fs::write(dir.join("beta.n.md"), "b").expect("write beta");
    let source = write_source(&temp, "readdir.nepl", fs_readdir_source());

    let output = cli_command()
        .current_dir(temp.path())
        .args(["--run", "-i"])
        .arg(&source)
        .args(["--target", "wasi"])
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert_no_internal_debug_stderr(&output);
    assert!(
        output.stdout.is_empty(),
        "unexpected stdout:\n{}",
        output_text(&output.stdout)
    );
}

#[test]
fn run_wasi_std_fs_read_dir_returns_stable_directory_entries() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path().join("dir");
    fs::create_dir(&dir).expect("create fixture dir");
    fs::write(dir.join("zeta.txt"), "z").expect("write zeta");
    fs::write(dir.join("alpha.nepl"), "a").expect("write alpha");
    fs::write(dir.join("beta.n.md"), "b").expect("write beta");
    let source = write_source(&temp, "std_readdir.nepl", fs_std_read_dir_source());

    let output = cli_command()
        .current_dir(temp.path())
        .args(["--run", "-i"])
        .arg(&source)
        .args(["--target", "wasi"])
        .output()
        .expect("run nepl-cli");

    assert_success(&output);
    assert_no_internal_debug_stderr(&output);
    assert!(
        output.stdout.is_empty(),
        "unexpected stdout:\n{}",
        output_text(&output.stdout)
    );
}
