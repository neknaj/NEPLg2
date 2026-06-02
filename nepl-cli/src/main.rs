use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Read, Write};
#[cfg(unix)]
use std::os::fd::AsRawFd;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use check_cache::ExactCheckCacheProbe;
use clap::{Args, Parser, Subcommand, ValueEnum};
use nepl_core::{
    check_module_with_source_map,
    check_module_with_source_map_resource_summary_value_cache_and_neplproof,
    compile_module_with_source_map_and_artifact_options,
    diagnostic::{Diagnostic, Severity},
    error::CoreError,
    loader::{Loader, SourceMap},
    resource::ResourceSummaryValueCache,
    BuildProfile, CompilationArtifact, CompilationArtifactOptions, CompileOptions, CompileTarget,
    ResourceSummaryProofArtifactCacheOptions,
};
use proof_cache::ResourceProofCacheProbe;
use wasmi::{Caller, Engine, Linker, Module, Store};
use wasmprinter::print_bytes;

mod check_cache;
mod codegen_llvm;
mod proof_cache;

macro_rules! cli_verbose {
    ($enabled:expr, $($arg:tt)*) => {
        if $enabled {
            eprintln!($($arg)*);
        }
    };
}

const WASI_ERRNO_BADF: i32 = 8;
const WASI_ERRNO_FAULT: i32 = 21;
const WASI_ERRNO_INVAL: i32 = 28;
const WASI_ERRNO_NOENT: i32 = 44;
const WASI_ERRNO_NOTCAPABLE: i32 = 76;
const WASI_RIGHT_FD_READ: i64 = 1 << 1;
const WASI_RIGHT_FD_WRITE: i64 = 1 << 6;
const WASI_RIGHT_FD_READDIR: i64 = 1 << 14;
const WASI_OFLAGS_CREAT: i32 = 1 << 0;
const WASI_OFLAGS_DIRECTORY: i32 = 1 << 1;
const WASI_OFLAGS_TRUNC: i32 = 1 << 3;
const WASI_FILETYPE_UNKNOWN: u8 = 0;
const WASI_FILETYPE_DIRECTORY: u8 = 3;
const WASI_FILETYPE_REGULAR_FILE: u8 = 4;
const WASI_FILETYPE_SYMBOLIC_LINK: u8 = 7;
const NEPL_STDLIB_ROOT_ENV: &str = "NEPL_STDLIB_ROOT";
const CLI_COMPILER_STACK_SIZE: usize = 32 * 1024 * 1024;

struct AllocState {
    stdin: Vec<u8>,
    stdin_pos: usize,
    stdin_eof: bool,
    args: Vec<Vec<u8>>,
    preopens: BTreeMap<i32, PathBuf>,
    files: BTreeMap<i32, FileState>,
    next_fd: i32,
    tty_cols: u32,
    tty_rows: u32,
    tty_width: u32,
    tty_height: u32,
    tty_stdin_tty: bool,
    tty_stdout_tty: bool,
    tty_stderr_tty: bool,
    tty_echo: bool,
    tty_line_buffered: bool,
    stdout_buf: Vec<u8>,
    stdout_last_flush: Instant,
    #[cfg(unix)]
    tty_saved: bool,
    #[cfg(unix)]
    tty_original: libc::termios,
}

enum FileState {
    Read { data: Vec<u8>, pos: usize },
    Write { file: fs::File },
    Dir { entries: Vec<DirEntryState> },
}

#[derive(Clone)]
struct DirEntryState {
    name: Vec<u8>,
    filetype: u8,
}

enum PathOpenMode {
    Read,
    WriteCreateTruncate,
    ReadDir,
}

#[cfg(unix)]
fn current_terminal_size() -> Option<(u32, u32)> {
    let fd = io::stdout().as_raw_fd();
    let mut ws = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    // SAFETY: ioctl writes into provided winsize buffer when fd is valid.
    let rc = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) };
    if rc == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
        Some((ws.ws_col as u32, ws.ws_row as u32))
    } else {
        None
    }
}

#[cfg(not(unix))]
fn current_terminal_size() -> Option<(u32, u32)> {
    None
}

#[cfg(unix)]
fn apply_host_tty_mode(state: &mut AllocState) -> i32 {
    let fd = io::stdin().as_raw_fd();
    // SAFETY: isatty is pure for fd validity check.
    if unsafe { libc::isatty(fd) } != 1 {
        return 0;
    }
    if !state.tty_saved {
        // SAFETY: zeroed is immediately filled by tcgetattr before use.
        let mut term: libc::termios = unsafe { std::mem::zeroed() };
        // SAFETY: tcgetattr writes current terminal settings to `term`.
        if unsafe { libc::tcgetattr(fd, &mut term) } != 0 {
            return 5;
        }
        state.tty_original = term;
        state.tty_saved = true;
    }

    let target_raw = !state.tty_echo || !state.tty_line_buffered;
    if target_raw {
        let mut term = state.tty_original;
        term.c_lflag &= !(libc::ECHO | libc::ICANON);
        term.c_cc[libc::VMIN] = 1;
        term.c_cc[libc::VTIME] = 0;
        // SAFETY: tcsetattr applies a valid termios struct to terminal fd.
        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &term) } != 0 {
            return 5;
        }
    } else if state.tty_saved {
        // SAFETY: restoring previously captured termios is valid.
        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &state.tty_original) } != 0 {
            return 5;
        }
    }
    0
}

#[cfg(not(unix))]
fn apply_host_tty_mode(_state: &mut AllocState) -> i32 {
    0
}

#[cfg(unix)]
fn restore_host_tty(state: &AllocState) {
    if !state.tty_saved {
        return;
    }
    let fd = io::stdin().as_raw_fd();
    // SAFETY: best-effort restore of previously saved terminal settings.
    let _ = unsafe { libc::tcsetattr(fd, libc::TCSANOW, &state.tty_original) };
}

#[cfg(not(unix))]
fn restore_host_tty(_state: &AllocState) {}

fn flush_stdout_buffer(state: &mut AllocState) -> io::Result<()> {
    if state.stdout_buf.is_empty() {
        return Ok(());
    }
    let mut out = io::stdout().lock();
    out.write_all(&state.stdout_buf)?;
    out.flush()?;
    state.stdout_buf.clear();
    state.stdout_last_flush = Instant::now();
    Ok(())
}

fn read_wasi_iov_bytes(
    memory: wasmi::Memory,
    caller: &Caller<'_, AllocState>,
    iovs: i32,
    iovs_len: i32,
) -> Result<(Vec<u8>, bool), i32> {
    if iovs < 0 || iovs_len < 0 {
        return Err(WASI_ERRNO_FAULT);
    }
    let data_snapshot = memory.data(caller).to_vec();
    let count = usize::try_from(iovs_len).map_err(|_| WASI_ERRNO_FAULT)?;
    let table_len = count.checked_mul(8).ok_or(WASI_ERRNO_FAULT)?;
    let mut offset = iovs as usize;
    if offset
        .checked_add(table_len)
        .is_none_or(|end| end > data_snapshot.len())
    {
        return Err(WASI_ERRNO_FAULT);
    }

    let mut bytes = Vec::new();
    let mut saw_newline = false;
    for _ in 0..count {
        let base =
            u32::from_le_bytes(data_snapshot[offset..offset + 4].try_into().unwrap()) as usize;
        let len =
            u32::from_le_bytes(data_snapshot[offset + 4..offset + 8].try_into().unwrap()) as usize;
        offset += 8;
        if base
            .checked_add(len)
            .is_none_or(|end| end > data_snapshot.len())
        {
            return Err(WASI_ERRNO_FAULT);
        }
        let slice = &data_snapshot[base..base + len];
        if slice.contains(&b'\n') {
            saw_newline = true;
        }
        bytes.extend_from_slice(slice);
    }
    Ok((bytes, saw_newline))
}

fn write_wasi_u32(
    memory: wasmi::Memory,
    caller: &mut Caller<'_, AllocState>,
    ptr: i32,
    value: u32,
) -> i32 {
    if ptr < 0 {
        return WASI_ERRNO_FAULT;
    }
    let offset = ptr as usize;
    if offset
        .checked_add(4)
        .is_none_or(|end| end > memory.data(&*caller).len())
    {
        return WASI_ERRNO_FAULT;
    }
    if memory.write(caller, offset, &value.to_le_bytes()).is_err() {
        return WASI_ERRNO_FAULT;
    }
    0
}

fn read_wasi_guest_path(
    memory: wasmi::Memory,
    caller: &Caller<'_, AllocState>,
    path_ptr: i32,
    path_len: i32,
) -> Result<String, i32> {
    if path_ptr < 0 || path_len < 0 {
        return Err(WASI_ERRNO_FAULT);
    }
    let mem = memory.data(caller);
    let start = path_ptr as usize;
    let end = start
        .checked_add(path_len as usize)
        .ok_or(WASI_ERRNO_FAULT)?;
    if end > mem.len() {
        return Err(WASI_ERRNO_FAULT);
    }
    std::str::from_utf8(&mem[start..end])
        .map(|path| path.to_string())
        .map_err(|_| WASI_ERRNO_INVAL)
}

fn filetype_from_metadata(metadata: &fs::Metadata) -> u8 {
    let ty = metadata.file_type();
    if ty.is_dir() {
        WASI_FILETYPE_DIRECTORY
    } else if ty.is_file() {
        WASI_FILETYPE_REGULAR_FILE
    } else if ty.is_symlink() {
        WASI_FILETYPE_SYMBOLIC_LINK
    } else {
        WASI_FILETYPE_UNKNOWN
    }
}

fn write_host_stderr(bytes: &[u8]) -> io::Result<()> {
    let mut err = io::stderr().lock();
    err.write_all(bytes)?;
    err.flush()
}

fn default_preopens() -> BTreeMap<i32, PathBuf> {
    let root = std::env::current_dir()
        .ok()
        .and_then(|p| p.canonicalize().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    let mut preopens = BTreeMap::new();
    preopens.insert(3, root);
    preopens
}

fn path_open_rights_within(rights_base: i64, rights_inherit: i64, allowed: i64) -> bool {
    let base_ok = rights_base == 0 || (rights_base & !allowed) == 0;
    let inherit_ok = rights_inherit == 0 || (rights_inherit & !allowed) == 0;
    base_ok && inherit_ok
}

fn path_open_mode(oflags: i32, rights_base: i64, rights_inherit: i64) -> Result<PathOpenMode, i32> {
    if oflags == 0 && path_open_rights_within(rights_base, rights_inherit, WASI_RIGHT_FD_READ) {
        return Ok(PathOpenMode::Read);
    }

    if (oflags == 0 || oflags == WASI_OFLAGS_DIRECTORY)
        && (rights_base & WASI_RIGHT_FD_READDIR) != 0
        && path_open_rights_within(rights_base, rights_inherit, WASI_RIGHT_FD_READDIR)
    {
        return Ok(PathOpenMode::ReadDir);
    }

    let write_oflags = WASI_OFLAGS_CREAT | WASI_OFLAGS_TRUNC;
    if oflags == write_oflags
        && (rights_base & WASI_RIGHT_FD_WRITE) != 0
        && path_open_rights_within(rights_base, rights_inherit, WASI_RIGHT_FD_WRITE)
    {
        return Ok(PathOpenMode::WriteCreateTruncate);
    }

    if oflags == 0 || oflags == WASI_OFLAGS_DIRECTORY || oflags == write_oflags {
        Err(WASI_ERRNO_NOTCAPABLE)
    } else {
        Err(WASI_ERRNO_INVAL)
    }
}

fn resolve_preopen_guest_path(root: &Path, guest_path: &str) -> Result<PathBuf, i32> {
    let path = Path::new(guest_path);
    if path.as_os_str().is_empty()
        || path.components().any(|component| {
            matches!(
                component,
                Component::Prefix(_) | Component::RootDir | Component::ParentDir
            )
        })
    {
        return Err(WASI_ERRNO_NOTCAPABLE);
    }

    Ok(root.join(path))
}

fn resolve_preopen_stat_path(root: &Path, guest_path: &str) -> Result<PathBuf, i32> {
    let candidate = resolve_preopen_guest_path(root, guest_path)?;
    let canonical = candidate.canonicalize().map_err(|_| WASI_ERRNO_NOENT)?;
    if !canonical.starts_with(root) {
        return Err(WASI_ERRNO_NOTCAPABLE);
    }
    Ok(canonical)
}

fn resolve_preopen_read_path(root: &Path, guest_path: &str) -> Result<PathBuf, i32> {
    let canonical = resolve_preopen_stat_path(root, guest_path)?;
    if !canonical.is_file() {
        return Err(WASI_ERRNO_NOTCAPABLE);
    }
    Ok(canonical)
}

fn resolve_preopen_dir_path(root: &Path, guest_path: &str) -> Result<PathBuf, i32> {
    let canonical = resolve_preopen_stat_path(root, guest_path)?;
    if !canonical.is_dir() {
        return Err(WASI_ERRNO_NOTCAPABLE);
    }
    Ok(canonical)
}

fn resolve_preopen_write_path(root: &Path, guest_path: &str) -> Result<PathBuf, i32> {
    let candidate = resolve_preopen_guest_path(root, guest_path)?;
    if let Ok(metadata) = candidate.symlink_metadata() {
        if metadata.file_type().is_symlink() || metadata.is_dir() {
            return Err(WASI_ERRNO_NOTCAPABLE);
        }
    }

    let parent = candidate.parent().ok_or(WASI_ERRNO_NOTCAPABLE)?;
    let canonical_parent = parent.canonicalize().map_err(|_| WASI_ERRNO_NOENT)?;
    if !canonical_parent.starts_with(root) || !canonical_parent.is_dir() {
        return Err(WASI_ERRNO_NOTCAPABLE);
    }
    let file_name = candidate.file_name().ok_or(WASI_ERRNO_NOTCAPABLE)?;
    Ok(canonical_parent.join(file_name))
}

/// コマンドライン引数を定義するための構造体
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(short, long)]
    input: Option<String>,

    #[arg(
        long,
        value_name = "DIR",
        global = true,
        help = "Override stdlib root directory (also supported by NEPL_STDLIB_ROOT)"
    )]
    stdlib_root: Option<PathBuf>,

    #[arg(short, long, help = "Output base path (extensionless recommended)")]
    output: Option<String>,

    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_value = "wasm",
        help = "Output formats: wasm, wat, wat-min, all"
    )]
    emit: Vec<Emit>,

    // WAT 出力（wat / wat-min）の先頭に、-i で指定した入力ソースをコメントとして付加する
    #[arg(
        long,
        help = "Attach the input source as WAT comments at the top of wat/wat-min outputs"
    )]
    attach_source: bool,

    #[arg(long, help = "Run the code if the output format is wasm")]
    run: bool,

    #[arg(
        long,
        help = "Only check the code for errors without generating output"
    )]
    check: bool,
    #[arg(
        value_name = "ARGS",
        num_args = 0..,
        trailing_var_arg = true,
        help = "Arguments passed to the WASI program after --"
    )]
    run_args: Vec<String>,
    #[arg(
        long,
        help = "Compile as library (currently unsupported; exits with an error)"
    )]
    lib: bool,

    #[arg(long, value_name = "TARGET", value_parser = ["wasm", "wasi", "wasix", "llvm", "core", "std"], help = "Compilation target: wasm, wasi, wasix, llvm, core(alias wasm), std(alias wasi)")]
    target: Option<String>,

    #[arg(short, long, global = true, help = "Enable verbose compiler logging")]
    verbose: bool,

    #[arg(
        long,
        value_enum,
        value_name = "PROFILE",
        help = "Compile profile: debug or release"
    )]
    profile: Option<ProfileArg>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum)]
enum Emit {
    Wasm,
    Wat,
    #[value(name = "wat-min")]
    WatMin,
    Llvm,
    #[value(name = "llvm-min")]
    LlvmMin,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ProfileArg {
    Debug,
    Release,
}

#[derive(Subcommand, Debug)]
enum Command {
    Test(TestArgs),
}

#[derive(Args, Debug)]
struct TestArgs {
    #[arg(value_name = "FILTER")]
    filter: Option<String>,
    #[arg(
        long,
        default_value = "tests",
        help = "Relative path inside stdlib to scan for .nepl tests"
    )]
    dir: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    execute(cli)
}

fn execute(cli: Cli) -> Result<()> {
    let worker = std::thread::Builder::new()
        .name("nepl-cli-compiler".to_string())
        .stack_size(CLI_COMPILER_STACK_SIZE)
        .spawn(move || execute_inner(cli))
        .context("failed to start nepl-cli compiler worker thread")?;
    match worker.join() {
        Ok(result) => result,
        Err(payload) => Err(anyhow::anyhow!(
            "nepl-cli compiler worker thread panicked: {}",
            panic_payload_message(payload.as_ref())
        )),
    }
}

fn panic_payload_message(payload: &(dyn std::any::Any + Send + 'static)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

fn execute_inner(cli: Cli) -> Result<()> {
    nepl_core::log::set_verbose(cli.verbose);
    if let Some(Command::Test(args)) = cli.command {
        return run_tests(args, cli.verbose, cli.stdlib_root.as_deref());
    }
    if cli.lib {
        return Err(anyhow::anyhow!(
            "--lib is not supported yet: library artifact contract is not implemented"
        ));
    }
    if !cli.run && !cli.check && cli.output.is_none() {
        return Err(anyhow::anyhow!(
            "Either --run, --check or --output is required"
        ));
    }
    let std_root = stdlib_root(cli.stdlib_root.as_deref())?;
    let target_override = cli.target.as_deref().map(|t| match t {
        "wasm" | "core" => CompileTarget::Wasm,
        "wasi" | "std" => CompileTarget::Wasi,
        "wasix" => CompileTarget::Wasix,
        "llvm" => CompileTarget::Llvm,
        _ => unreachable!(),
    });
    let profile = cli.profile.map(|p| match p {
        ProfileArg::Debug => BuildProfile::Debug,
        ProfileArg::Release => BuildProfile::Release,
    });
    let active_profile = profile.unwrap_or(BuildProfile::default_source_profile());
    let program_name = cli.input.clone().unwrap_or_else(|| "<stdin>".to_string());
    let input_path = cli.input.clone();
    let pre_load_check_cache_path = if cli.check {
        match (&input_path, target_override) {
            (Some(path), Some(target)) => ExactCheckCacheProbe::path_for_input(
                Path::new(path),
                &std_root,
                target,
                active_profile,
            ),
            _ => None,
        }
    } else {
        None
    };
    if let Some(path) = pre_load_check_cache_path.as_ref() {
        if ExactCheckCacheProbe::hit_manifest_at(path) {
            cli_verbose!(cli.verbose, "DEBUG: pre-load exact --check cache hit");
            println!("Check successful");
            return Ok(());
        }
    }
    let (module, source_map) = match &cli.input {
        Some(path) => {
            cli_verbose!(cli.verbose, "DEBUG: Creating Loader for path: {}", path);
            let mut loader = Loader::new(std_root.clone());
            cli_verbose!(cli.verbose, "DEBUG: Loader created, starting load");
            let entry = PathBuf::from(path);
            match loader.load(&entry) {
                Ok(res) => (res.module, loader.source_map().clone()),
                Err(e) => {
                    if let nepl_core::loader::LoaderError::Core(CoreError::Diagnostics(diags)) = &e
                    {
                        render_diagnostics(diags, loader.source_map());
                        std::process::exit(1);
                    }
                    return Err(anyhow::anyhow!(e.to_string()));
                }
            }
        }
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            let mut loader = Loader::new(std_root.clone());
            match loader.load_inline(PathBuf::from("<stdin>"), buf) {
                Ok(res) => (res.module, loader.source_map().clone()),
                Err(e) => {
                    if let nepl_core::loader::LoaderError::Core(CoreError::Diagnostics(diags)) = &e
                    {
                        render_diagnostics(diags, loader.source_map());
                        std::process::exit(1);
                    }
                    return Err(anyhow::anyhow!(e.to_string()));
                }
            }
        }
    };

    let mut emits = expand_emits(&cli.emit);

    // If target is llvm, and no specific llvm-ish emits are requested,
    // add Llvm to emits. This handles the case where --emit defaults to wasm.
    if matches!(target_override, Some(CompileTarget::Llvm)) {
        if !emits.contains(&Emit::Llvm) && !emits.contains(&Emit::LlvmMin) {
            emits.insert(Emit::Llvm);
        }
    }

    let module_decl_target = detect_module_target(&module);
    let run_target = target_override
        .or(module_decl_target)
        .unwrap_or(CompileTarget::Wasm);

    let is_check = cli.check;
    let options = CompileOptions {
        target: target_override,
        verbose: cli.verbose,
        profile,
    };
    if is_check {
        let exact_check_cache = ExactCheckCacheProbe::new(
            &source_map,
            run_target,
            active_profile,
            pre_load_check_cache_path,
        );
        if exact_check_cache
            .as_ref()
            .is_some_and(ExactCheckCacheProbe::hit)
        {
            cli_verbose!(cli.verbose, "DEBUG: exact --check cache hit");
            println!("Check successful");
            return Ok(());
        }
        let proof_cache = input_path.as_ref().and_then(|path| {
            ResourceProofCacheProbe::new(Path::new(path), &std_root, run_target, active_profile)
        });
        let use_proof_cache = proof_cache
            .as_ref()
            .is_some_and(|cache| cache.has_preseed_bytes() || cache.should_bootstrap_on_miss());
        if use_proof_cache {
            let mut resource_summary_value_cache = ResourceSummaryValueCache::new();
            resource_summary_value_cache.disable_raw_alias_return_entry_collection();
            let proof_options = match proof_cache
                .as_ref()
                .and_then(ResourceProofCacheProbe::preseed_bytes)
            {
                Some(bytes) => ResourceSummaryProofArtifactCacheOptions::preseed_bytes(bytes, None)
                    .only_after_accepted_preseed(),
                None => ResourceSummaryProofArtifactCacheOptions::none(),
            };
            match check_module_with_source_map_resource_summary_value_cache_and_neplproof(
                module,
                Some(&source_map),
                options,
                Some(&mut resource_summary_value_cache),
                proof_options,
            ) {
                Ok(result) => {
                    if let Some(report) = result.resource_summary_proof_preseed_report {
                        cli_verbose!(
                            cli.verbose,
                            "DEBUG: .neplproof preseed accepted={} existing={} conflicts={} compatibility_reject={:?} codec_error={:?}",
                            report.accepted_entries,
                            report.existing_matching_entries,
                            report.rejected_conflict_entries,
                            report.compatibility_reject,
                            report.codec_error
                        );
                    }
                    if let Some(cache) = proof_cache.as_ref() {
                        let cache_stats = resource_summary_value_cache.stats();
                        if cache.should_store_artifact_after_check(
                            result.resource_summary_proof_preseed_report,
                            cache_stats,
                        ) {
                            let artifact = resource_summary_value_cache
                                .export_neplproof_artifact(result.resource_summary_proof_header);
                            cache.store_artifact(&artifact)?;
                        } else {
                            cli_verbose!(
                                cli.verbose,
                                "DEBUG: .neplproof store skipped because preseed artifact remained current"
                            );
                        }
                    }
                    if let Some(cache) = exact_check_cache.as_ref() {
                        cache.store_success()?;
                    }
                    println!("Check successful");
                    return Ok(());
                }
                Err(CoreError::Diagnostics(diags)) => {
                    render_diagnostics(&diags, &source_map);
                    return Err(anyhow::anyhow!("compilation failed"));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(e.to_string()));
                }
            }
        }
        match check_module_with_source_map(module, Some(&source_map), options) {
            Ok(()) => {
                if let Some(cache) = exact_check_cache.as_ref() {
                    cache.store_success()?;
                }
                println!("Check successful");
                return Ok(());
            }
            Err(CoreError::Diagnostics(diags)) => {
                render_diagnostics(&diags, &source_map);
                return Err(anyhow::anyhow!("compilation failed"));
            }
            Err(e) => {
                return Err(anyhow::anyhow!(e.to_string()));
            }
        }
    }
    if matches!(run_target, CompileTarget::Llvm) {
        if cli.run {
            return Err(anyhow::anyhow!(
                "--run is not supported for --target llvm (emit .ll and execute with clang/lli)"
            ));
        }
        codegen_llvm::ensure_llvm_toolchain_from_env()?;
        let output = cli
            .output
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--output is required for --target llvm"))?;
        let base = output_base_from_arg(output);

        if emits.contains(&Emit::Llvm) {
            let ir = nepl_core::codegen_llvm::emit_ll_from_module_for_target_with_source_map(
                &module,
                run_target,
                active_profile,
                false,
                Some(&source_map),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            write_bytes(&base.with_extension("ll"), ir.as_bytes())?;
        }
        if emits.contains(&Emit::LlvmMin) {
            let ir = nepl_core::codegen_llvm::emit_ll_from_module_for_target_with_source_map(
                &module,
                run_target,
                active_profile,
                true,
                Some(&source_map),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            write_bytes(&output_path(&base, Emit::LlvmMin), ir.as_bytes())?;
        }
        return Ok(());
    }

    cli_verbose!(cli.verbose, "DEBUG: Calling compile_module");
    let include_wat_comments = emits.contains(&Emit::Wat);
    let artifact = match compile_module_with_source_map_and_artifact_options(
        module,
        Some(&source_map),
        options,
        CompilationArtifactOptions {
            include_wat_comments,
        },
    ) {
        Ok(a) => {
            cli_verbose!(cli.verbose, "DEBUG: compile_module returned Ok");
            a
        }
        Err(CoreError::Diagnostics(diags)) => {
            cli_verbose!(cli.verbose, "DEBUG: compile_module returned Diagnostics");
            render_diagnostics(&diags, &source_map);
            return Err(anyhow::anyhow!("compilation failed"));
        }
        Err(e) => {
            cli_verbose!(cli.verbose, "DEBUG: compile_module returned Err: {:?}", e);
            return Err(anyhow::anyhow!(e.to_string()));
        }
    };

    if let Some(out) = &cli.output {
        let base = output_base_from_arg(out);

        // --attach-source が true の場合、wat / wat-min の先頭に入力ソースをコメントとして付加する
        // stdin から読み込んだ場合（--input が無い）は付加できないのでエラーにする
        let attached_source =
            if cli.attach_source && (emits.contains(&Emit::Wat) || emits.contains(&Emit::WatMin)) {
                let input = input_path
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("--attach-source requires --input"))?;
                Some(read_attached_source(Path::new(input))?)
            } else {
                None
            };

        write_outputs(
            &base,
            &artifact.wasm,
            &artifact.wat_comments,
            &emits,
            attached_source.as_ref(),
            None,
            None,
        )?;
    }
    if cli.run {
        let mut wasm_args = Vec::new();
        wasm_args.push(program_name);
        wasm_args.extend(cli.run_args.clone());
        let result = run_wasm(&artifact, run_target, wasm_args)?;
        if result != 0 {
            println!("Program exited with {result}");
        }
    }
    Ok(())
}

fn run_tests(args: TestArgs, verbose: bool, stdlib_root_override: Option<&Path>) -> Result<()> {
    const ANSI_RESET: &str = "\x1b[0m";
    const ANSI_GREEN: &str = "\x1b[32m";
    const ANSI_RED: &str = "\x1b[31m";
    const ANSI_CYAN: &str = "\x1b[36m";

    let std_root = stdlib_root(stdlib_root_override)?;
    let dir = PathBuf::from(&args.dir);
    let base = if dir.is_absolute() {
        dir
    } else {
        std_root.join(dir)
    };
    if !base.exists() {
        return Err(anyhow::anyhow!(
            "tests directory not found: {}",
            base.display()
        ));
    }
    let mut files = Vec::new();
    collect_nepl_files(&base, &mut files)?;
    files.sort();
    if let Some(filter) = &args.filter {
        files.retain(|p| p.display().to_string().contains(filter));
    }
    if files.is_empty() {
        return Err(anyhow::anyhow!("no tests found"));
    }

    let mut failed = 0usize;
    for file in files {
        let name = file
            .strip_prefix(&base)
            .unwrap_or(&file)
            .display()
            .to_string();
        print!("{ANSI_CYAN}test{ANSI_RESET} {name} ... ");
        match run_test_file(&file, &std_root, verbose) {
            Ok(()) => {
                println!("{ANSI_GREEN}ok{ANSI_RESET}");
            }
            Err(e) => {
                println!("{ANSI_RED}FAILED{ANSI_RESET}");
                eprintln!("{e}");
                failed += 1;
            }
        }
    }

    if failed > 0 {
        Err(anyhow::anyhow!("{failed} tests failed"))
    } else {
        Ok(())
    }
}

fn run_test_file(path: &Path, std_root: &Path, verbose: bool) -> Result<()> {
    let mut loader = Loader::new(std_root.to_path_buf());
    cli_verbose!(
        verbose,
        "[nepl-cli] run_test_file: loading {}",
        path.display()
    );
    let res = match loader.load(&path.to_path_buf()) {
        Ok(res) => res,
        Err(nepl_core::loader::LoaderError::Core(CoreError::Diagnostics(diags))) => {
            render_diagnostics(&diags, loader.source_map());
            return Err(anyhow::anyhow!("parsing failed"));
        }
        Err(e) => return Err(anyhow::anyhow!(e.to_string())),
    };
    cli_verbose!(verbose, "[nepl-cli] compile_module for {}", path.display());
    let artifact = match compile_module_with_source_map_and_artifact_options(
        res.module,
        Some(loader.source_map()),
        CompileOptions {
            target: Some(CompileTarget::Wasi),
            verbose,
            profile: None,
        },
        CompilationArtifactOptions {
            include_wat_comments: false,
        },
    ) {
        Ok(a) => a,
        Err(CoreError::Diagnostics(diags)) => {
            render_diagnostics(&diags, loader.source_map());
            return Err(anyhow::anyhow!("compilation failed"));
        }
        Err(e) => return Err(anyhow::anyhow!(e.to_string())),
    };
    let mut wasm_args = Vec::new();
    wasm_args.push(path.display().to_string());
    wasm_args.push("--flag".to_string());
    wasm_args.push("value".to_string());
    let result = run_wasm(&artifact, CompileTarget::Wasi, wasm_args)?;
    if result != 0 {
        return Err(anyhow::anyhow!("non-zero exit code: {result}"));
    }
    Ok(())
}

fn collect_nepl_files(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_nepl_files(&path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("nepl") {
            out.push(path);
        }
    }
    Ok(())
}

fn output_base_from_arg(output: &str) -> PathBuf {
    if output.ends_with(".min.wat") {
        return PathBuf::from(output.trim_end_matches(".min.wat"));
    }
    let path = PathBuf::from(output);
    match path.extension().and_then(|s| s.to_str()) {
        Some("wasm") | Some("wat") => path.with_extension(""),
        _ => path,
    }
}

fn expand_emits(emits: &[Emit]) -> BTreeSet<Emit> {
    let mut set = BTreeSet::new();
    for emit in emits {
        match emit {
            Emit::All => {
                set.insert(Emit::Wasm);
                set.insert(Emit::Wat);
                set.insert(Emit::WatMin);
                set.insert(Emit::Llvm);
                set.insert(Emit::LlvmMin);
            }
            other => {
                set.insert(*other);
            }
        }
    }
    set
}

// このラッパーが依存している NEPLg2 コンパイラの情報（固定値）
// 生成した WAT に「どのコンパイラで生成したか」を残すために使う
const NEPLG2_REPO_URL: &str = "https://github.com/neknaj/NEPLg2/";
const NEPLG2_COMPILER_COMMIT: &str = env!("NEPLG2_COMPILER_COMMIT");
const NEPLG2_COMPILER_COMMIT_URL: &str = concat!(
    "https://github.com/neknaj/NEPLg2/commit/",
    env!("NEPLG2_COMPILER_COMMIT")
);

struct AttachedSource {
    path: PathBuf,
    text: String,
}

fn read_attached_source(path: &Path) -> Result<AttachedSource> {
    // 入力ファイル（-i）の内容を読み込み、WAT のコメントとして埋め込める形にする
    let bytes =
        fs::read(path).with_context(|| format!("failed to read input file {}", path.display()))?;
    let text = String::from_utf8_lossy(&bytes).to_string();
    Ok(AttachedSource {
        path: path.to_path_buf(),
        text,
    })
}

fn prepend_compiler_info_as_wat_comment(wat: &str) -> String {
    // WAT の行コメント ";;" を使って、コンパイラ情報を先頭に付加する
    // 仕様上、コメントはトークン間の空白として扱われるため、(module ...) の前に置ける
    let mut out = String::new();
    out.push_str(";; compiler: NEPLg2 ");
    out.push_str(NEPLG2_REPO_URL);
    out.push('\n');
    out.push_str(";; compiler commit: ");
    out.push_str(NEPLG2_COMPILER_COMMIT);
    out.push('\n');
    out.push_str(";; compiler commit url: ");
    out.push_str(NEPLG2_COMPILER_COMMIT_URL);
    out.push_str("\n\n");
    out.push_str(wat);
    out
}

fn prepend_attached_source_as_wat_comment(wat: &str, attached: &AttachedSource) -> String {
    // WAT の行コメント ";;" を使って、任意のテキストを安全にコメント化する
    // 各行を ";; " で始めることで、入力テキストに何が含まれていても「コメントから抜けない」
    let mut out = String::new();
    out.push_str(";; ---- BEGIN ATTACHED SOURCE ----\n");
    out.push_str(";; path: ");
    out.push_str(&attached.path.display().to_string());
    out.push('\n');

    for (idx, chunk) in attached.text.split_inclusive('\n').enumerate() {
        let line = chunk.strip_suffix('\n').unwrap_or(chunk);
        out.push_str(";; ");
        out.push_str(&format!("{:04}: ", idx + 1));
        out.push_str(line);
        out.push('\n');
    }
    if !attached.text.ends_with('\n') {
        // split_inclusive が末尾行に改行を含めない場合があるので、見た目を揃える
        out.push('\n');
    }
    out.push_str(";; ---- END ATTACHED SOURCE ----\n\n");
    out.push_str(wat);
    out
}

fn prepend_nepl_wat_debug_as_comment(wat: &str, debug_text: &str) -> String {
    if debug_text.trim().is_empty() {
        return wat.to_string();
    }
    let mut out = String::new();
    out.push_str(";; ---- BEGIN NEPL WAT DEBUG ----\n");
    for line in debug_text.lines() {
        out.push_str(";; ");
        out.push_str(line);
        out.push('\n');
    }
    out.push_str(";; ---- END NEPL WAT DEBUG ----\n\n");
    out.push_str(wat);
    out
}

fn output_path(base: &Path, emit: Emit) -> PathBuf {
    match emit {
        Emit::Wasm => base.with_extension("wasm"),
        Emit::Wat => base.with_extension("wat"),
        Emit::WatMin => PathBuf::from(format!("{}.min.wat", base.display())),
        Emit::Llvm => base.with_extension("ll"),
        Emit::LlvmMin => PathBuf::from(format!("{}.min.ll", base.display())),
        Emit::All => base.to_path_buf(),
    }
}

fn write_outputs(
    base: &Path,
    wasm: &[u8],
    wat_debug: &str,
    emits: &BTreeSet<Emit>,
    attached_source: Option<&AttachedSource>,
    llvm_ir: Option<&str>,
    llvm_ir_min: Option<&str>,
) -> Result<()> {
    if emits.contains(&Emit::Wasm) {
        let path = output_path(base, Emit::Wasm);
        write_bytes(&path, wasm)?;
    }
    if emits.contains(&Emit::Wat) {
        let path = output_path(base, Emit::Wat);
        let mut wat_text = make_wat_pretty(wasm)?;
        wat_text = prepend_nepl_wat_debug_as_comment(&wat_text, wat_debug);
        if let Some(attached) = attached_source {
            wat_text = prepend_attached_source_as_wat_comment(&wat_text, attached);
        }
        wat_text = prepend_compiler_info_as_wat_comment(&wat_text);
        write_bytes(&path, wat_text.as_bytes())?;
    }
    if emits.contains(&Emit::WatMin) {
        let path = output_path(base, Emit::WatMin);
        let mut wat_text = make_wat_min(wasm)?;
        if let Some(attached) = attached_source {
            wat_text = prepend_attached_source_as_wat_comment(&wat_text, attached);
        }
        wat_text = prepend_compiler_info_as_wat_comment(&wat_text);
        write_bytes(&path, wat_text.as_bytes())?;
    }
    if emits.contains(&Emit::Llvm) {
        if let Some(ir) = llvm_ir {
            let path = output_path(base, Emit::Llvm);
            write_bytes(&path, ir.as_bytes())?;
        }
    }
    if emits.contains(&Emit::LlvmMin) {
        if let Some(ir) = llvm_ir_min {
            let path = output_path(base, Emit::LlvmMin);
            write_bytes(&path, ir.as_bytes())?;
        }
    }
    Ok(())
}
fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {parent:?}"))?;
        }
    }
    fs::write(path, bytes)
        .with_context(|| format!("failed to write output file {}", path.display()))
}

fn make_wat_pretty(wasm: &[u8]) -> Result<String> {
    print_bytes(wasm).with_context(|| "failed to convert wasm to wat")
}

fn make_wat_min(wasm: &[u8]) -> Result<String> {
    let out = print_bytes(wasm).with_context(|| "failed to convert wasm to wat")?;
    Ok(minify_wat_text(&out))
}

fn minify_wat_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut comment_depth = 0usize;
    let mut prev_space = false;

    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            continue;
        }

        if comment_depth > 0 {
            if c == '(' && chars.peek() == Some(&';') {
                chars.next();
                comment_depth += 1;
                continue;
            }
            if c == ';' && chars.peek() == Some(&')') {
                chars.next();
                comment_depth = comment_depth.saturating_sub(1);
                if comment_depth == 0 && !prev_space && !out.is_empty() {
                    out.push(' ');
                    prev_space = true;
                }
                continue;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            out.push(c);
            prev_space = false;
            continue;
        }
        if c == ';' && chars.peek() == Some(&';') {
            chars.next();
            while let Some(next) = chars.next() {
                if next == '\n' {
                    break;
                }
            }
            if !prev_space && !out.is_empty() {
                out.push(' ');
                prev_space = true;
            }
            continue;
        }
        if c == '(' && chars.peek() == Some(&';') {
            chars.next();
            comment_depth = 1;
            continue;
        }
        if c.is_whitespace() {
            if !prev_space && !out.is_empty() {
                out.push(' ');
                prev_space = true;
            }
            continue;
        }
        if c == '(' {
            if out.ends_with(' ') {
                out.pop();
            }
            out.push('(');
            prev_space = false;
            continue;
        }
        if c == ')' {
            if out.ends_with(' ') {
                out.pop();
            }
            out.push(')');
            prev_space = false;
            continue;
        }
        out.push(c);
        prev_space = false;
    }

    out.trim().to_string()
}

fn run_wasm(
    artifact: &CompilationArtifact,
    target: CompileTarget,
    args: Vec<String>,
) -> Result<i32> {
    let engine = Engine::default();
    let module = Module::new(&engine, artifact.wasm.as_slice())
        .context("failed to compile wasm artifact")?;
    let args_bytes: Vec<Vec<u8>> = args
        .into_iter()
        .map(|s| {
            let mut b = s.into_bytes();
            b.push(0);
            b
        })
        .collect();

    let mut linker: Linker<AllocState> = Linker::new(&engine);
    match target {
        CompileTarget::Wasi | CompileTarget::Wasix => {
            for import in module.imports() {
                if import.module() != "wasi_snapshot_preview1" && import.module() != "wasix_32v1" {
                    return Err(anyhow::anyhow!(
                        "unsupported import {}::{} (only wasi_snapshot_preview1 or wasix_32v1 are allowed for wasi/wasix targets)",
                        import.module(),
                        import.name()
                    ));
                }
            }
        }
        CompileTarget::Wasm => {
            if let Some(import) = module.imports().next() {
                return Err(anyhow::anyhow!(
                    "wasm target does not allow host imports during run: {}::{} (use #target wasi or --target wasi)",
                    import.module(),
                    import.name()
                ));
            }
        }
        CompileTarget::Llvm => {
            return Err(anyhow::anyhow!(
                "LLVM target cannot be executed by the wasm runner"
            ));
        }
    }
    if matches!(target, CompileTarget::Wasi | CompileTarget::Wasix) {
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "args_sizes_get",
            |mut caller: Caller<'_, AllocState>, argc_ptr: i32, argv_buf_size_ptr: i32| -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return 21,
                };
                if argc_ptr < 0 || argv_buf_size_ptr < 0 {
                    return 21;
                }
                let argc = caller.data().args.len() as u32;
                let buf_size: u32 = caller.data().args.iter().map(|a| a.len() as u32).sum();
                let mem_len = memory.data(&caller).len();
                let argc_offset = argc_ptr as usize;
                let buf_offset = argv_buf_size_ptr as usize;
                if argc_offset + 4 > mem_len || buf_offset + 4 > mem_len {
                    return 21;
                }
                if memory
                    .write(&mut caller, argc_offset, &argc.to_le_bytes())
                    .is_err()
                {
                    return 21;
                }
                if memory
                    .write(&mut caller, buf_offset, &buf_size.to_le_bytes())
                    .is_err()
                {
                    return 21;
                }
                0
            },
        )?;
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "args_get",
            |mut caller: Caller<'_, AllocState>, argv: i32, argv_buf: i32| -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return 21,
                };
                if argv < 0 || argv_buf < 0 {
                    return 21;
                }
                let mem_len = memory.data(&caller).len();
                let args = caller.data().args.clone();
                let mut argv_offset = argv as usize;
                let mut buf_offset = argv_buf as usize;
                for arg in args.iter() {
                    if argv_offset + 4 > mem_len {
                        return 21;
                    }
                    let ptr_bytes = (buf_offset as u32).to_le_bytes();
                    if memory.write(&mut caller, argv_offset, &ptr_bytes).is_err() {
                        return 21;
                    }
                    if buf_offset + arg.len() > mem_len {
                        return 21;
                    }
                    if memory.write(&mut caller, buf_offset, arg).is_err() {
                        return 21;
                    }
                    argv_offset += 4;
                    buf_offset += arg.len();
                }
                0
            },
        )?;
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "path_open",
            |mut caller: Caller<'_, AllocState>,
             dirfd: i32,
             dirflags: i32,
             path_ptr: i32,
             path_len: i32,
             oflags: i32,
             rights_base: i64,
             rights_inherit: i64,
             fdflags: i32,
             fd_out: i32|
             -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return WASI_ERRNO_FAULT,
                };
                if dirflags != 0 || fdflags != 0 {
                    return WASI_ERRNO_INVAL;
                }
                let open_mode = match path_open_mode(oflags, rights_base, rights_inherit) {
                    Ok(mode) => mode,
                    Err(errno) => return errno,
                };
                if fd_out < 0 {
                    return WASI_ERRNO_FAULT;
                }
                let root = match caller.data().preopens.get(&dirfd) {
                    Some(root) => root.clone(),
                    None => return WASI_ERRNO_BADF,
                };
                if fd_out as usize > memory.data(&caller).len().saturating_sub(4) {
                    return WASI_ERRNO_FAULT;
                }
                let path = match read_wasi_guest_path(memory, &caller, path_ptr, path_len) {
                    Ok(path) => path,
                    Err(errno) => return errno,
                };
                let file_state = match open_mode {
                    PathOpenMode::Read => {
                        let path = match resolve_preopen_read_path(&root, &path) {
                            Ok(path) => path,
                            Err(errno) => return errno,
                        };
                        let data = match fs::read(path) {
                            Ok(d) => d,
                            Err(_) => return WASI_ERRNO_NOENT,
                        };
                        FileState::Read { data, pos: 0 }
                    }
                    PathOpenMode::WriteCreateTruncate => {
                        let path = match resolve_preopen_write_path(&root, &path) {
                            Ok(path) => path,
                            Err(errno) => return errno,
                        };
                        let file = match fs::OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(true)
                            .open(path)
                        {
                            Ok(file) => file,
                            Err(_) => return WASI_ERRNO_NOENT,
                        };
                        FileState::Write { file }
                    }
                    PathOpenMode::ReadDir => {
                        let path = match resolve_preopen_dir_path(&root, &path) {
                            Ok(path) => path,
                            Err(errno) => return errno,
                        };
                        let mut entries = Vec::new();
                        let read_dir = match fs::read_dir(path) {
                            Ok(read_dir) => read_dir,
                            Err(_) => return WASI_ERRNO_NOENT,
                        };
                        for entry in read_dir {
                            let entry = match entry {
                                Ok(entry) => entry,
                                Err(_) => return WASI_ERRNO_FAULT,
                            };
                            let filetype = match entry.file_type() {
                                Ok(filetype) if filetype.is_dir() => WASI_FILETYPE_DIRECTORY,
                                Ok(filetype) if filetype.is_file() => WASI_FILETYPE_REGULAR_FILE,
                                Ok(filetype) if filetype.is_symlink() => {
                                    WASI_FILETYPE_SYMBOLIC_LINK
                                }
                                Ok(_) => WASI_FILETYPE_UNKNOWN,
                                Err(_) => return WASI_ERRNO_FAULT,
                            };
                            entries.push(DirEntryState {
                                name: entry.file_name().to_string_lossy().as_bytes().to_vec(),
                                filetype,
                            });
                        }
                        entries.sort_by(|a, b| a.name.cmp(&b.name));
                        FileState::Dir { entries }
                    }
                };
                let fd = caller.data().next_fd;
                let fd_u32 = match u32::try_from(fd) {
                    Ok(fd) => fd,
                    Err(_) => return WASI_ERRNO_FAULT,
                };
                {
                    let state = caller.data_mut();
                    state.next_fd += 1;
                    state.files.insert(fd, file_state);
                }
                write_wasi_u32(memory, &mut caller, fd_out, fd_u32)
            },
        )?;
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "path_filestat_get",
            |mut caller: Caller<'_, AllocState>,
             dirfd: i32,
             flags: i32,
             path_ptr: i32,
             path_len: i32,
             buf: i32|
             -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return WASI_ERRNO_FAULT,
                };
                if flags != 0 || buf < 0 {
                    return WASI_ERRNO_INVAL;
                }
                if (buf as usize)
                    .checked_add(64)
                    .is_none_or(|end| end > memory.data(&caller).len())
                {
                    return WASI_ERRNO_FAULT;
                }
                let root = match caller.data().preopens.get(&dirfd) {
                    Some(root) => root.clone(),
                    None => return WASI_ERRNO_BADF,
                };
                let path = match read_wasi_guest_path(memory, &caller, path_ptr, path_len) {
                    Ok(path) => path,
                    Err(errno) => return errno,
                };
                let path = match resolve_preopen_stat_path(&root, &path) {
                    Ok(path) => path,
                    Err(errno) => return errno,
                };
                let metadata = match fs::symlink_metadata(path) {
                    Ok(metadata) => metadata,
                    Err(_) => return WASI_ERRNO_NOENT,
                };
                let mut stat = [0u8; 64];
                stat[16] = filetype_from_metadata(&metadata);
                stat[32..40].copy_from_slice(&metadata.len().to_le_bytes());
                if memory.write(&mut caller, buf as usize, &stat).is_err() {
                    return WASI_ERRNO_FAULT;
                }
                0
            },
        )?;
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "fd_readdir",
            |mut caller: Caller<'_, AllocState>,
             fd: i32,
             buf: i32,
             buf_len: i32,
             cookie: i64,
             bufused: i32|
             -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return WASI_ERRNO_FAULT,
                };
                if buf < 0 || buf_len < 0 || bufused < 0 || cookie < 0 {
                    return WASI_ERRNO_FAULT;
                }
                let mem_len = memory.data(&caller).len();
                let start = buf as usize;
                let capacity = buf_len as usize;
                if start.checked_add(capacity).is_none_or(|end| end > mem_len)
                    || (bufused as usize)
                        .checked_add(4)
                        .is_none_or(|end| end > mem_len)
                {
                    return WASI_ERRNO_FAULT;
                }
                let entries = match caller.data().files.get(&fd) {
                    Some(FileState::Dir { entries }) => entries.clone(),
                    Some(FileState::Read { .. }) | Some(FileState::Write { .. }) => {
                        return WASI_ERRNO_BADF;
                    }
                    None => return WASI_ERRNO_BADF,
                };
                let mut written = 0usize;
                let start_index = match usize::try_from(cookie) {
                    Ok(index) => index,
                    Err(_) => return WASI_ERRNO_INVAL,
                };
                for (idx, entry) in entries.iter().enumerate().skip(start_index) {
                    let record_len = match 24usize.checked_add(entry.name.len()) {
                        Some(len) => len,
                        None => return WASI_ERRNO_FAULT,
                    };
                    if written
                        .checked_add(record_len)
                        .is_none_or(|end| end > capacity)
                    {
                        break;
                    }
                    let name_len = match u32::try_from(entry.name.len()) {
                        Ok(len) => len,
                        Err(_) => return WASI_ERRNO_INVAL,
                    };
                    let next_cookie = match u64::try_from(idx + 1) {
                        Ok(cookie) => cookie,
                        Err(_) => return WASI_ERRNO_INVAL,
                    };
                    let mut record = vec![0u8; record_len];
                    record[0..8].copy_from_slice(&next_cookie.to_le_bytes());
                    record[16..20].copy_from_slice(&name_len.to_le_bytes());
                    record[20] = entry.filetype;
                    record[24..].copy_from_slice(&entry.name);
                    if memory.write(&mut caller, start + written, &record).is_err() {
                        return WASI_ERRNO_FAULT;
                    }
                    written += record_len;
                }
                write_wasi_u32(memory, &mut caller, bufused, written as u32)
            },
        )?;
        linker.func_wrap(
            "wasix_32v1",
            "tty_get",
            |mut caller: Caller<'_, AllocState>, tty_ptr: i32| -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return 21,
                };
                if tty_ptr < 0 {
                    return 21;
                }
                let base = tty_ptr as usize;
                if base + 21 > memory.data(&caller).len() {
                    return 21;
                }
                if let Some((cols, rows)) = current_terminal_size() {
                    let state = caller.data_mut();
                    state.tty_cols = cols;
                    state.tty_rows = rows;
                    state.tty_width = cols;
                    state.tty_height = rows;
                }
                let cols = caller.data().tty_cols.to_le_bytes();
                let rows = caller.data().tty_rows.to_le_bytes();
                let width = caller.data().tty_width.to_le_bytes();
                let height = caller.data().tty_height.to_le_bytes();
                if memory.write(&mut caller, base, &cols).is_err() {
                    return 21;
                }
                if memory.write(&mut caller, base + 4, &rows).is_err() {
                    return 21;
                }
                if memory.write(&mut caller, base + 8, &width).is_err() {
                    return 21;
                }
                if memory.write(&mut caller, base + 12, &height).is_err() {
                    return 21;
                }
                let stdin_tty = [if caller.data().tty_stdin_tty { 1 } else { 0 }];
                let stdout_tty = [if caller.data().tty_stdout_tty { 1 } else { 0 }];
                let stderr_tty = [if caller.data().tty_stderr_tty { 1 } else { 0 }];
                let echo = [if caller.data().tty_echo { 1 } else { 0 }];
                let line_buffered = [if caller.data().tty_line_buffered {
                    1
                } else {
                    0
                }];
                if memory.write(&mut caller, base + 16, &stdin_tty).is_err() {
                    return 21;
                }
                if memory.write(&mut caller, base + 17, &stdout_tty).is_err() {
                    return 21;
                }
                if memory.write(&mut caller, base + 18, &stderr_tty).is_err() {
                    return 21;
                }
                if memory.write(&mut caller, base + 19, &echo).is_err() {
                    return 21;
                }
                if memory
                    .write(&mut caller, base + 20, &line_buffered)
                    .is_err()
                {
                    return 21;
                }
                0
            },
        )?;
        linker.func_wrap(
            "wasix_32v1",
            "tty_set",
            |mut caller: Caller<'_, AllocState>, tty_ptr: i32| -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return 21,
                };
                if tty_ptr < 0 {
                    return 21;
                }
                let base = tty_ptr as usize;
                let data = memory.data(&caller);
                if base + 21 > data.len() {
                    return 21;
                }
                let cols = u32::from_le_bytes(data[base..base + 4].try_into().unwrap());
                let rows = u32::from_le_bytes(data[base + 4..base + 8].try_into().unwrap());
                let width = u32::from_le_bytes(data[base + 8..base + 12].try_into().unwrap());
                let height = u32::from_le_bytes(data[base + 12..base + 16].try_into().unwrap());
                let stdin_tty = data[base + 16] != 0;
                let stdout_tty = data[base + 17] != 0;
                let stderr_tty = data[base + 18] != 0;
                let echo = data[base + 19] != 0;
                let line_buffered = data[base + 20] != 0;
                let state = caller.data_mut();
                state.tty_cols = cols;
                state.tty_rows = rows;
                state.tty_width = width;
                state.tty_height = height;
                state.tty_stdin_tty = stdin_tty;
                state.tty_stdout_tty = stdout_tty;
                state.tty_stderr_tty = stderr_tty;
                state.tty_echo = echo;
                state.tty_line_buffered = line_buffered;
                apply_host_tty_mode(state)
            },
        )?;
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "fd_read",
            |mut caller: Caller<'_, AllocState>,
             fd: i32,
             iovs: i32,
             iovs_len: i32,
             nread: i32|
             -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return 21,
                };
                let data_snapshot = memory.data(&caller).to_vec();
                let mut total = 0usize;
                let mut offset = iovs as usize;
                let count = if iovs_len > 0 { iovs_len as usize } else { 0 };
                if fd == 0 {
                    if caller.data().stdin_pos >= caller.data().stdin.len()
                        && !caller.data().stdin_eof
                    {
                        let mut buf = vec![0u8; 4096];
                        let read = match io::stdin().read(&mut buf) {
                            Ok(n) => n,
                            Err(_) => 0,
                        };
                        if read == 0 {
                            caller.data_mut().stdin_eof = true;
                            caller.data_mut().stdin.clear();
                            caller.data_mut().stdin_pos = 0;
                        } else {
                            caller.data_mut().stdin = buf[..read].to_vec();
                            caller.data_mut().stdin_pos = 0;
                        }
                    }
                    let stdin_snapshot = caller.data().stdin.clone();
                    let mut pos = caller.data().stdin_pos;
                    for _ in 0..count {
                        if offset + 8 > data_snapshot.len() {
                            return 21;
                        }
                        let base = u32::from_le_bytes(
                            data_snapshot[offset..offset + 4].try_into().unwrap(),
                        ) as usize;
                        let len = u32::from_le_bytes(
                            data_snapshot[offset + 4..offset + 8].try_into().unwrap(),
                        ) as usize;
                        offset += 8;
                        if base + len > data_snapshot.len() {
                            return 21;
                        }
                        if pos >= stdin_snapshot.len() {
                            break;
                        }
                        let avail = stdin_snapshot.len() - pos;
                        let take = if len < avail { len } else { avail };
                        if take == 0 {
                            break;
                        }
                        memory
                            .write(&mut caller, base, &stdin_snapshot[pos..pos + take])
                            .ok();
                        pos += take;
                        total += take;
                    }
                    caller.data_mut().stdin_pos = pos;
                } else {
                    for _ in 0..count {
                        if offset + 8 > data_snapshot.len() {
                            return 21;
                        }
                        let base = u32::from_le_bytes(
                            data_snapshot[offset..offset + 4].try_into().unwrap(),
                        ) as usize;
                        let len = u32::from_le_bytes(
                            data_snapshot[offset + 4..offset + 8].try_into().unwrap(),
                        ) as usize;
                        offset += 8;
                        if base + len > data_snapshot.len() {
                            return 21;
                        }
                        let (take, chunk) = {
                            let file = match caller.data_mut().files.get_mut(&fd) {
                                Some(f) => f,
                                None => return WASI_ERRNO_BADF,
                            };
                            match file {
                                FileState::Read { data, pos } => {
                                    if *pos >= data.len() {
                                        (0, Vec::new())
                                    } else {
                                        let avail = data.len() - *pos;
                                        let take = if len < avail { len } else { avail };
                                        let chunk = data[*pos..*pos + take].to_vec();
                                        *pos += take;
                                        (take, chunk)
                                    }
                                }
                                FileState::Write { .. } | FileState::Dir { .. } => {
                                    return WASI_ERRNO_BADF;
                                }
                            }
                        };
                        if take == 0 {
                            break;
                        }
                        memory.write(&mut caller, base, &chunk).ok();
                        total += take;
                    }
                }
                if let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                    let bytes = (total as u32).to_le_bytes();
                    if (nread as usize) + 4 <= mem.data(&caller).len() {
                        mem.write(&mut caller, nread as usize, &bytes).ok();
                    }
                }
                0
            },
        )?;
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "fd_close",
            |mut caller: Caller<'_, AllocState>, fd: i32| -> i32 {
                if fd <= 2 {
                    return 0;
                }
                if caller.data_mut().files.remove(&fd).is_none() {
                    return 8;
                }
                0
            },
        )?;
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "fd_write",
            |mut caller: Caller<'_, AllocState>,
             fd: i32,
             iovs: i32,
             iovs_len: i32,
             nwritten: i32|
             -> i32 {
                if fd != 1
                    && fd != 2
                    && !matches!(caller.data().files.get(&fd), Some(FileState::Write { .. }))
                {
                    return WASI_ERRNO_BADF;
                }
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return WASI_ERRNO_FAULT,
                };

                let (bytes, saw_newline) =
                    match read_wasi_iov_bytes(memory, &caller, iovs, iovs_len) {
                        Ok(v) => v,
                        Err(errno) => return errno,
                    };
                let total = match u32::try_from(bytes.len()) {
                    Ok(v) => v,
                    Err(_) => return WASI_ERRNO_FAULT,
                };

                if fd == 1 {
                    caller.data_mut().stdout_buf.extend_from_slice(&bytes);
                    let should_flush = {
                        let state = caller.data();
                        saw_newline
                            || state.stdout_buf.len() >= 8192
                            || state.stdout_last_flush.elapsed() >= Duration::from_millis(16)
                    };
                    if should_flush && flush_stdout_buffer(caller.data_mut()).is_err() {
                        return WASI_ERRNO_FAULT;
                    }
                } else if fd == 2 {
                    if write_host_stderr(&bytes).is_err() {
                        return WASI_ERRNO_FAULT;
                    }
                } else {
                    let result = {
                        let state = caller.data_mut();
                        match state.files.get_mut(&fd) {
                            Some(FileState::Write { file }) => file.write_all(&bytes),
                            Some(FileState::Read { .. }) | Some(FileState::Dir { .. }) => {
                                return WASI_ERRNO_BADF;
                            }
                            None => return WASI_ERRNO_BADF,
                        }
                    };
                    if result.is_err() {
                        return WASI_ERRNO_FAULT;
                    }
                }

                write_wasi_u32(memory, &mut caller, nwritten, total)
            },
        )?;
    }
    let (tty_cols, tty_rows) = current_terminal_size().unwrap_or_else(|| {
        let cols = std::env::var("COLUMNS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(80);
        let rows = std::env::var("LINES")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(25);
        (cols, rows)
    });
    let mut store = Store::new(
        &engine,
        AllocState {
            stdin: Vec::new(),
            stdin_pos: 0,
            stdin_eof: false,
            args: args_bytes,
            preopens: default_preopens(),
            files: BTreeMap::new(),
            next_fd: 4,
            tty_cols,
            tty_rows,
            tty_width: tty_cols,
            tty_height: tty_rows,
            tty_stdin_tty: true,
            tty_stdout_tty: true,
            tty_stderr_tty: true,
            tty_echo: true,
            tty_line_buffered: true,
            stdout_buf: Vec::new(),
            stdout_last_flush: Instant::now(),
            #[cfg(unix)]
            tty_saved: false,
            #[cfg(unix)]
            tty_original: unsafe { std::mem::zeroed() },
        },
    );
    let instance_pre = linker
        .instantiate(&mut store, &module)
        .context("failed to instantiate module")?;
    let instance = instance_pre
        .start(&mut store)
        .context("failed to start module")?;
    let result = if let Ok(main) = instance.get_typed_func::<(), i32>(&store, "main") {
        main.call(&mut store, ()).context("failed to execute main")
    } else if let Ok(main_unit) = instance.get_typed_func::<(), ()>(&store, "main") {
        main_unit
            .call(&mut store, ())
            .context("failed to execute main")?;
        Ok(0)
    } else {
        Err(anyhow::anyhow!(
            "exported main function missing or has wrong type"
        ))
    };
    let _ = flush_stdout_buffer(store.data_mut());
    restore_host_tty(store.data());
    result
}

fn detect_module_target(module: &nepl_core::ast::Module) -> Option<CompileTarget> {
    if let Some(target) = module.directives.iter().find_map(|d| {
        if let nepl_core::ast::Directive::Target { target, .. } = d {
            match target.as_str() {
                "wasi" | "std" => Some(CompileTarget::Wasi),
                "wasix" => Some(CompileTarget::Wasix),
                "wasm" | "core" => Some(CompileTarget::Wasm),
                "llvm" => Some(CompileTarget::Llvm),
                _ => None,
            }
        } else {
            None
        }
    }) {
        return Some(target);
    }

    module.root.items.iter().find_map(|stmt| {
        if let nepl_core::ast::Stmt::Directive(nepl_core::ast::Directive::Target {
            target, ..
        }) = stmt
        {
            match target.as_str() {
                "wasi" | "std" => Some(CompileTarget::Wasi),
                "wasix" => Some(CompileTarget::Wasix),
                "wasm" | "core" => Some(CompileTarget::Wasm),
                "llvm" => Some(CompileTarget::Llvm),
                _ => None,
            }
        } else {
            None
        }
    })
}

struct StdlibRootCandidate {
    source: String,
    path: PathBuf,
}

fn stdlib_root(override_root: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = override_root {
        return resolve_stdlib_root_candidates(vec![StdlibRootCandidate {
            source: "--stdlib-root".to_string(),
            path: path.to_path_buf(),
        }]);
    }

    if let Some(path) = std::env::var_os(NEPL_STDLIB_ROOT_ENV).filter(|v| !v.is_empty()) {
        return resolve_stdlib_root_candidates(vec![StdlibRootCandidate {
            source: NEPL_STDLIB_ROOT_ENV.to_string(),
            path: PathBuf::from(path),
        }]);
    }

    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(StdlibRootCandidate {
                source: "current executable sibling".to_string(),
                path: exe_dir.join("stdlib"),
            });
            if let Some(prefix_dir) = exe_dir.parent() {
                candidates.push(StdlibRootCandidate {
                    source: "current executable prefix".to_string(),
                    path: prefix_dir.join("stdlib"),
                });
            }
        }
    }
    candidates.push(StdlibRootCandidate {
        source: "build-time fallback".to_string(),
        path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("stdlib"),
    });
    resolve_stdlib_root_candidates(candidates)
}

fn resolve_stdlib_root_candidates(candidates: Vec<StdlibRootCandidate>) -> Result<PathBuf> {
    for candidate in &candidates {
        if let Ok(root) = candidate.path.canonicalize() {
            if root.join("core").is_dir() && root.join("std").is_dir() {
                return Ok(root);
            }
        }
    }

    let tried = candidates
        .iter()
        .map(|candidate| format!("  - {}: {}", candidate.source, candidate.path.display()))
        .collect::<Vec<_>>()
        .join("\n");
    Err(anyhow::anyhow!(
        "stdlib directory not found. Tried:\n{}",
        tried
    ))
}

fn render_diagnostics(diags: &[Diagnostic], sm: &SourceMap) {
    for d in diags {
        let severity = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let code_display = format!("[{}]", d.code.as_str());
        let primary = &d.primary;
        let (line, col) = sm
            .line_col(primary.span.file_id, primary.span.start)
            .unwrap_or((0, 0));
        let path = sm
            .path(primary.span.file_id)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<unknown>".into());
        eprintln!("{severity}{code_display}: {message}", message = d.message);
        eprintln!(" --> {path}:{line}:{col}", line = line + 1, col = col + 1);
        if let Some(line_str) = sm.line_str(primary.span.file_id, line) {
            eprintln!(
                "  {line_num:>4} | {text}",
                line_num = line + 1,
                text = line_str
            );
            let line_len = line_str.len();
            let caret_start = col.min(line_len);
            let max_len = line_len.saturating_sub(caret_start);
            let mut caret_len = primary.span.len().max(1) as usize;
            if max_len > 0 {
                caret_len = caret_len.min(max_len);
            } else {
                caret_len = 1;
            }
            eprintln!(
                "       | {spaces}{carets}",
                spaces = " ".repeat(caret_start),
                carets = "^".repeat(caret_len)
            );
        }
        for label in &d.secondary {
            let (l, c) = sm
                .line_col(label.span.file_id, label.span.start)
                .unwrap_or((0, 0));
            let p = sm
                .path(label.span.file_id)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<unknown>".into());
            let msg = label.message.as_ref().map(|m| m.as_str()).unwrap_or("");
            eprintln!(" note: {p}:{line}:{col}: {msg}", line = l + 1, col = c + 1);
        }
        for note in &d.notes {
            eprintln!(" note: {note}");
        }
        for help in &d.helps {
            eprintln!(" help: {help}");
        }
        eprintln!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deep_identity_source(call_count: usize) -> String {
        let mut source = String::from(
            "#entry main\n#indent 4\n#target core\n\nfn inc <(i32)->i32> (x):\n    x\n\nfn main <()->i32> ():\n    ",
        );
        for _ in 0..call_count {
            source.push_str("inc ");
        }
        source.push_str("0\n");
        source
    }

    #[test]
    fn cli_parses_defaults() {
        let cli = Cli::parse_from(["nepl-cli", "--run"]);
        assert_eq!(cli.emit, vec![Emit::Wasm]);
        assert!(cli.run);
        assert!(cli.output.is_none());
        assert!(cli.stdlib_root.is_none());
    }

    #[test]
    fn cli_parses_stdlib_root() {
        let cli = Cli::parse_from(["nepl-cli", "--run", "--stdlib-root", "custom/stdlib"]);
        assert_eq!(cli.stdlib_root, Some(PathBuf::from("custom/stdlib")));
    }

    #[test]
    fn cli_parses_emit_list() {
        let cli = Cli::parse_from(["nepl-cli", "--run", "--emit", "wasm,wat-min"]);
        assert_eq!(cli.emit, vec![Emit::Wasm, Emit::WatMin]);
    }

    #[test]
    fn cli_parses_profile() {
        let cli = Cli::parse_from(["nepl-cli", "--run", "--profile", "debug"]);
        assert_eq!(cli.profile, Some(ProfileArg::Debug));
    }

    #[test]
    fn output_base_handles_extensions() {
        assert_eq!(output_base_from_arg("out/a.wasm"), PathBuf::from("out/a"));
        assert_eq!(output_base_from_arg("out/a.wat"), PathBuf::from("out/a"));
        assert_eq!(
            output_base_from_arg("out/a.min.wat"),
            PathBuf::from("out/a")
        );
        assert_eq!(
            output_base_from_arg("out/a.custom"),
            PathBuf::from("out/a.custom")
        );
    }

    #[test]
    fn check_runs_compiler_diagnostics() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("check_fails.nepl");
        fs::write(
            &path,
            r#"#entry main
#target core
#indent 4

fn main <()->i32> ():
    let bad <i32> unknown_symbol;
    bad
"#,
        )
        .expect("write source");

        let input = path.to_str().expect("path utf8");
        let cli = Cli::parse_from(["nepl-cli", "--check", "-i", input]);
        let err = execute(cli).expect_err("--check must fail for compiler diagnostics");

        assert!(
            err.to_string().contains("compilation failed"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn check_runs_resource_ir_static_safety_gates() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("check_resource_fails.nepl");
        fs::write(
            &path,
            r#"#entry main
#indent 4
#target wasm

fn raw_store <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store

fn main <()->i32> ():
    raw_store 0 1
    0
"#,
        )
        .expect("write source");

        let input = path.to_str().expect("path utf8");
        let cli = Cli::parse_from(["nepl-cli", "--check", "-i", input]);
        let err = execute(cli).expect_err("--check must fail for Resource IR diagnostics");

        assert!(
            err.to_string().contains("compilation failed"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn check_accepts_deep_prefix_chain_without_codegen_stack_overflow() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("check_deep.nepl");
        fs::write(&path, deep_identity_source(1105)).expect("write source");

        let input = path.to_str().expect("path utf8");
        let cli = Cli::parse_from(["nepl-cli", "--check", "-i", input]);

        execute(cli).expect("--check should not enter recursive codegen pipeline");
    }

    #[test]
    fn minify_wat_removes_comments_and_whitespace() {
        let input = r#"
            (module
                ;; line comment
                (func (param i32) (result i32)
                    (i32.add (local.get 0) (i32.const 1))
                )
                (; block
                   comment ;)
                (export "add one" (func 0))
            )
        "#;
        let out = minify_wat_text(input);
        assert!(!out.contains(";;"));
        assert!(!out.contains("block"));
        assert!(out.contains("(module"));
        assert!(out.contains("(export \"add one\""));
        assert!(!out.contains("\n"));
    }

    #[test]
    fn write_outputs_creates_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let base = tmp.path().join("out/test");
        let wasm = b"\0asm\x01\0\0\0";
        let mut emits = BTreeSet::new();
        emits.insert(Emit::Wasm);
        emits.insert(Emit::Wat);
        emits.insert(Emit::WatMin);

        write_outputs(&base, wasm, "", &emits, None, None, None).expect("write outputs");

        let wasm_path = base.with_extension("wasm");
        let wat_path = base.with_extension("wat");
        let wat_min_path = PathBuf::from(format!("{}.min.wat", base.display()));
        assert!(wasm_path.exists());
        assert!(wat_path.exists());
        assert!(wat_min_path.exists());
        assert!(!fs::read_to_string(wat_path).unwrap_or_default().is_empty());
        assert!(!fs::read_to_string(wat_min_path)
            .unwrap_or_default()
            .is_empty());
    }
}
