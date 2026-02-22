use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use nepl_core::{
    compile_module,
    diagnostic::{Diagnostic, Severity},
    error::CoreError,
    loader::{Loader, SourceMap},
    BuildProfile, CompilationArtifact, CompileOptions, CompileTarget,
};
use wasmi::{Caller, Engine, Linker, Module, Store};
use wasmprinter::print_bytes;

mod codegen_llvm;

#[derive(Default)]
struct AllocState {
    // head of free list (address in linear memory), 0 == null
    free_head: u32,
    stdin: Vec<u8>,
    stdin_pos: usize,
    stdin_eof: bool,
    args: Vec<Vec<u8>>,
    files: BTreeMap<i32, FileState>,
    next_fd: i32,
}

struct FileState {
    data: Vec<u8>,
    pos: usize,
}

/// コマンドライン引数を定義するための構造体
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(short, long)]
    input: Option<String>,

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
    #[arg(long, help = "Attach the input source as WAT comments at the top of wat/wat-min outputs")]
    attach_source: bool,

    #[arg(long, help = "Run the code if the output format is wasm")]
    run: bool,
    #[arg(
        value_name = "ARGS",
        num_args = 0..,
        trailing_var_arg = true,
        help = "Arguments passed to the WASI program after --"
    )]
    run_args: Vec<String>,
    #[arg(
        long,
        help = "Compile as library (do not wrap top-level in an implicit main)"
    )]
    lib: bool,

    #[arg(long, value_name = "TARGET", value_parser = ["wasm", "wasi", "llvm", "core", "std"], help = "Compilation target: wasm, wasi, llvm, core(alias wasm), std(alias wasi)")]
    target: Option<String>,

    #[arg(short, long, global = true, help = "Enable verbose compiler logging")]
    verbose: bool,

    #[arg(long, value_enum, value_name = "PROFILE", help = "Compile profile: debug or release")]
    profile: Option<ProfileArg>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum)]
enum Emit {
    Wasm,
    Wat,
    #[value(name = "wat-min")]
    WatMin,
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
    #[arg(long, default_value = "tests", help = "Relative path inside stdlib to scan for .nepl tests")]
    dir: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    execute(cli)
}

fn execute(cli: Cli) -> Result<()> {
    if let Some(Command::Test(args)) = cli.command {
        return run_tests(args, cli.verbose);
    }
    if !cli.run && cli.output.is_none() {
        return Err(anyhow::anyhow!("Either --run or --output is required"));
    }
    let emits = expand_emits(&cli.emit);
    let program_name = cli
        .input
        .clone()
        .unwrap_or_else(|| "<stdin>".to_string());
    let input_path = cli.input.clone();
    let (module, source_map) = match &cli.input {
        Some(path) => {
            eprintln!("DEBUG: Creating Loader for path: {}", path);
            let mut loader = Loader::new(stdlib_root()?);
            eprintln!("DEBUG: Loader created, starting load");
            // Loader::load は &PathBuf を要求するため、入力パス(String)を PathBuf に変換して渡す
            let entry: PathBuf = PathBuf::from(path);
            match loader.load(&entry) {
                Ok(res) => {
                    eprintln!("DEBUG: Load successful");
                    (res.module, loader.source_map().clone())
                },
                Err(e) => {
                    eprintln!("DEBUG: Load failed: {:?}", e);
                    if let nepl_core::loader::LoaderError::Core(CoreError::Diagnostics(diags)) = &e {
                        render_diagnostics(diags, loader.source_map());
                        std::process::exit(1);
                    }
                    return Err(anyhow::anyhow!(e.to_string()));
                }
            }
        }
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            eprintln!("DEBUG: Creating Loader for stdin");
            let mut loader = Loader::new(stdlib_root()?);
            eprintln!("DEBUG: Loader created, starting load_inline");
            match loader.load_inline(PathBuf::from("<stdin>"), buffer) {
                Ok(res) => {
                    eprintln!("DEBUG: Load inline successful");
                    (res.module, loader.source_map().clone())
                },
                Err(e) => {
                    eprintln!("DEBUG: Load inline failed: {:?}", e);
                    if let nepl_core::loader::LoaderError::Core(CoreError::Diagnostics(diags)) = &e {
                        render_diagnostics(diags, loader.source_map());
                        std::process::exit(1);
                    }
                    return Err(anyhow::anyhow!(e.to_string()));
                }
            }
        }
    };

    let cli_target = cli.target.as_deref().map(|t| match t {
        "wasi" | "std" => CompileTarget::Wasi,
        "llvm" => CompileTarget::Llvm,
        "wasm" | "core" => CompileTarget::Wasm,
        _ => CompileTarget::Wasm,
    });
    let target_override = cli_target;
    let module_decl_target = detect_module_target(&module);
    let run_target = target_override
        .or(module_decl_target)
        .unwrap_or(CompileTarget::Wasm);
    let profile = cli.profile.map(|p| match p {
        ProfileArg::Debug => BuildProfile::Debug,
        ProfileArg::Release => BuildProfile::Release,
    });
    let active_profile = profile.unwrap_or(BuildProfile::detect());
    if matches!(run_target, CompileTarget::Llvm) {
        if cli.run {
            return Err(anyhow::anyhow!(
                "--run is not supported for --target llvm (emit .ll and execute with clang/lli)"
            ));
        }
        codegen_llvm::ensure_clang_21_linux_native()?;
        let llvm_ir = nepl_core::codegen_llvm::emit_ll_from_module_for_target(
            &module,
            run_target,
            active_profile,
        )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let output = cli
            .output
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--output is required for --target llvm"))?;
        let ll_path = output_base_from_arg(output).with_extension("ll");
        write_bytes(&ll_path, llvm_ir.as_bytes())?;
        return Ok(());
    }
    let options = CompileOptions {
        target: target_override,
        verbose: cli.verbose,
        profile,
    };

    eprintln!("DEBUG: Calling compile_module");
    let artifact = match compile_module(module, options) {
        Ok(a) => {
            eprintln!("DEBUG: compile_module returned Ok");
            a
        },
        Err(CoreError::Diagnostics(diags)) => {
            eprintln!("DEBUG: compile_module returned Diagnostics");
            render_diagnostics(&diags, &source_map);
            return Err(anyhow::anyhow!("compilation failed"));
        }
        Err(e) => {
            eprintln!("DEBUG: compile_module returned Err: {:?}", e);
            return Err(anyhow::anyhow!(e.to_string()));
        },
    };
    if let Some(out) = &cli.output {
        let base = output_base_from_arg(out);

        // --attach-source が true の場合、wat / wat-min の先頭に入力ソースをコメントとして付加する
        // stdin から読み込んだ場合（--input が無い）は付加できないのでエラーにする
        let attached_source = if cli.attach_source
            && (emits.contains(&Emit::Wat) || emits.contains(&Emit::WatMin))
        {
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

    if cli.lib {
        eprintln!("--lib is acknowledged but not yet implemented in the placeholder pipeline");
    }

    Ok(())
}

fn run_tests(args: TestArgs, verbose: bool) -> Result<()> {
    const ANSI_RESET: &str = "\x1b[0m";
    const ANSI_GREEN: &str = "\x1b[32m";
    const ANSI_RED: &str = "\x1b[31m";
    const ANSI_CYAN: &str = "\x1b[36m";

    let std_root = stdlib_root()?;
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
    println!("[nepl-cli] run_test_file: loading {}", path.display());
    let res = match loader.load(&path.to_path_buf()) {
        Ok(res) => res,
        Err(nepl_core::loader::LoaderError::Core(CoreError::Diagnostics(diags))) => {
            render_diagnostics(&diags, loader.source_map());
            return Err(anyhow::anyhow!("parsing failed"));
        }
        Err(e) => return Err(anyhow::anyhow!(e.to_string())),
    };
    println!("[nepl-cli] compile_module for {}", path.display());
    let artifact = match compile_module(
        res.module,
        CompileOptions {
            target: Some(CompileTarget::Wasi),
            verbose,
            profile: None,
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
const NEPLG2_COMPILER_COMMIT_URL: &str = concat!("https://github.com/neknaj/NEPLg2/commit/", env!("NEPLG2_COMPILER_COMMIT"));

struct AttachedSource {
    path: PathBuf,
    text: String,
}

fn read_attached_source(path: &Path) -> Result<AttachedSource> {
    // 入力ファイル（-i）の内容を読み込み、WAT のコメントとして埋め込める形にする
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read input file {}", path.display()))?;
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
        Emit::All => base.to_path_buf(),
    }
}

fn write_outputs(
    base: &Path,
    wasm: &[u8],
    wat_debug: &str,
    emits: &BTreeSet<Emit>,
    attached_source: Option<&AttachedSource>,
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
        CompileTarget::Wasi => {
            for import in module.imports() {
                if import.module() != "wasi_snapshot_preview1" {
                    return Err(anyhow::anyhow!(
                        "unsupported non-WASI import {}::{} (only wasi_snapshot_preview1 is allowed)",
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
    if matches!(target, CompileTarget::Wasi) {
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
                let buf_size: u32 = caller
                    .data()
                    .args
                    .iter()
                    .map(|a| a.len() as u32)
                    .sum();
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
                    if memory
                        .write(&mut caller, argv_offset, &ptr_bytes)
                        .is_err()
                    {
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
             _dirfd: i32,
             _dirflags: i32,
             path_ptr: i32,
             path_len: i32,
             _oflags: i32,
             _rights_base: i64,
             _rights_inherit: i64,
             _fdflags: i32,
             fd_out: i32|
             -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return 21,
                };
                if path_ptr < 0 || path_len < 0 || fd_out < 0 {
                    return 21;
                }
                let mem = memory.data(&caller);
                let start = path_ptr as usize;
                let end = start.saturating_add(path_len as usize);
                if end > mem.len() || (fd_out as usize) + 4 > mem.len() {
                    return 21;
                }
                let path = std::str::from_utf8(&mem[start..end]).unwrap_or("");
                let data = match fs::read(path) {
                    Ok(d) => d,
                    Err(_) => return 44,
                };
                let fd = caller.data().next_fd;
                caller.data_mut().next_fd += 1;
                caller
                    .data_mut()
                    .files
                    .insert(fd, FileState { data, pos: 0 });
                let fd_bytes = (fd as u32).to_le_bytes();
                if memory
                    .write(&mut caller, fd_out as usize, &fd_bytes)
                    .is_err()
                {
                    return 21;
                }
                0
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
                                None => return 8,
                            };
                            if file.pos >= file.data.len() {
                                (0, Vec::new())
                            } else {
                                let avail = file.data.len() - file.pos;
                                let take = if len < avail { len } else { avail };
                                let chunk = file.data[file.pos..file.pos + take].to_vec();
                                file.pos += take;
                                (take, chunk)
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
        // Minimal wasi fd_write implementation for stdout (fd 1)
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "fd_write",
            |mut caller: Caller<'_, AllocState>,
             fd: i32,
             iovs: i32,
             iovs_len: i32,
             nwritten: i32|
             -> i32 {
                if fd != 1 {
                    return 8; // badf
                }
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return 21, // enomem-ish
                };
                let data_snapshot = memory.data(&caller).to_vec(); // snapshot to avoid alias issues
                let mut total = 0usize;
                let mut offset = iovs as usize;
                let mut stdout = io::stdout().lock();
                for _ in 0..iovs_len {
                    if offset + 8 > data_snapshot.len() {
                        return 21;
                    }
                    let base =
                        u32::from_le_bytes(data_snapshot[offset..offset + 4].try_into().unwrap())
                            as usize;
                    let len = u32::from_le_bytes(
                        data_snapshot[offset + 4..offset + 8].try_into().unwrap(),
                    ) as usize;
                    offset += 8;
                    if base + len > data_snapshot.len() {
                        return 21;
                    }
                    let slice = &data_snapshot[base..base + len];
                    if stdout.write_all(slice).is_err() {
                        return 21;
                    }
                    total += len;
                }
                let _ = stdout.flush();
                // write nwritten
                if let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                    let bytes = (total as u32).to_le_bytes();
                    if (nwritten as usize) + 4 <= mem.data(&caller).len() {
                        mem.write(&mut caller, nwritten as usize, &bytes).ok();
                    }
                }
                0
            },
        )?;
    }
    let mut store = Store::new(
        &engine,
        AllocState {
            free_head: 0,
            stdin: Vec::new(),
            stdin_pos: 0,
            stdin_eof: false,
            args: args_bytes,
            files: BTreeMap::new(),
            next_fd: 4,
        },
    );
    let instance_pre = linker
        .instantiate(&mut store, &module)
        .context("failed to instantiate module")?;
    let instance = instance_pre
        .start(&mut store)
        .context("failed to start module")?;
    if let Ok(main) = instance.get_typed_func::<(), i32>(&store, "main") {
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
    }
}

fn detect_module_target(module: &nepl_core::ast::Module) -> Option<CompileTarget> {
    if let Some(target) = module.directives.iter().find_map(|d| {
        if let nepl_core::ast::Directive::Target { target, .. } = d {
            match target.as_str() {
                "wasi" | "std" => Some(CompileTarget::Wasi),
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
        if let nepl_core::ast::Stmt::Directive(nepl_core::ast::Directive::Target { target, .. }) =
            stmt
        {
            match target.as_str() {
                "wasi" | "std" => Some(CompileTarget::Wasi),
                "wasm" | "core" => Some(CompileTarget::Wasm),
                "llvm" => Some(CompileTarget::Llvm),
                _ => None,
            }
        } else {
            None
        }
    })
}

fn stdlib_root() -> Result<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib");
    path.canonicalize()
        .context(format!("stdlib directory not found at {}", path.display()))
}

fn render_diagnostics(diags: &[Diagnostic], sm: &SourceMap) {
    for d in diags {
        let severity = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let code = d.code.unwrap_or("");
        let primary = &d.primary;
        let (line, col) = sm
            .line_col(primary.span.file_id, primary.span.start)
            .unwrap_or((0, 0));
        let path = sm
            .path(primary.span.file_id)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<unknown>".into());
        let code_display = if code.is_empty() {
            String::new()
        } else {
            format!("[{code}]")
        };
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
        eprintln!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_defaults() {
        let cli = Cli::parse_from(["nepl-cli", "--run"]);
        assert_eq!(cli.emit, vec![Emit::Wasm]);
        assert!(cli.run);
        assert!(cli.output.is_none());
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
        assert_eq!(
            output_base_from_arg("out/a.wasm"),
            PathBuf::from("out/a")
        );
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

        write_outputs(&base, wasm, "", &emits, None).expect("write outputs");

        let wasm_path = base.with_extension("wasm");
        let wat_path = base.with_extension("wat");
        let wat_min_path = PathBuf::from(format!("{}.min.wat", base.display()));
        assert!(wasm_path.exists());
        assert!(wat_path.exists());
        assert!(wat_min_path.exists());
        assert!(!fs::read_to_string(wat_path).unwrap_or_default().is_empty());
        assert!(!fs::read_to_string(wat_min_path).unwrap_or_default().is_empty());
    }
}
