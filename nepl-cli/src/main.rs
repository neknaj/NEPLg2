use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use nepl_core::{
    compile_module,
    diagnostic::{Diagnostic, Severity},
    error::CoreError,
    loader::{Loader, SourceMap},
    CompilationArtifact, CompileOptions, CompileTarget,
};
use wasmi::{Caller, Engine, Linker, Module, Store};
use wasmprinter::print_bytes;

#[derive(Default)]
struct AllocState {
    // head of free list (address in linear memory), 0 == null
    free_head: u32,
    stdin: Vec<u8>,
    stdin_pos: usize,
    stdin_eof: bool,
}

/// コマンドライン引数を定義するための構造体
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(short, long)]
    input: Option<String>,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(
        long,
        value_name = "FORMAT",
        default_value = "wasm",
        help = "Output format: wasm"
    )]
    emit: String,

    #[arg(long, help = "Run the code if the output format is wasm")]
    run: bool,
    #[arg(
        long,
        help = "Compile as library (do not wrap top-level in an implicit main)"
    )]
    lib: bool,

    #[arg(long, value_name = "TARGET", value_parser = ["wasm", "wasi"], help = "Compilation target: wasm or wasi (overrides #target)")]
    target: Option<String>,

    #[arg(short, long, global = true, help = "Enable verbose compiler logging")]
    verbose: bool,
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
    let (module, source_map) = match cli.input {
        Some(path) => {
            let mut loader = Loader::new(stdlib_root()?);
            match loader.load(&PathBuf::from(path)) {
                Ok(m) => (m, loader.source_map().clone()),
                Err(e) => {
                    if let CoreError::Diagnostics(diags) = &e {
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
            let mut loader = Loader::new(stdlib_root()?);
            match loader.load_inline(PathBuf::from("<stdin>"), buffer) {
                Ok(m) => (m, loader.source_map().clone()),
                Err(e) => {
                    if let CoreError::Diagnostics(diags) = &e {
                        render_diagnostics(diags, loader.source_map());
                        std::process::exit(1);
                    }
                    return Err(anyhow::anyhow!(e.to_string()));
                }
            }
        }
    };

    // Auto-upgrade to WASI if stdio is imported and user did not explicitly pick wasm/wasi.
    let has_stdio_import = module.directives.iter().any(
        |d| matches!(d, nepl_core::ast::Directive::Import { path, .. } if path == "std/stdio"),
    );


    let cli_target = cli.target.as_deref().map(|t| {
        if t == "wasi" {
            CompileTarget::Wasi
        } else {
            CompileTarget::Wasm
        }
    });
    let target_override = cli_target.or_else(|| {
        if has_stdio_import {
            Some(CompileTarget::Wasi)
        } else {
            None
        }
    });
    let module_decl_target = module.directives.iter().find_map(|d| {
        if let nepl_core::ast::Directive::Target { target, .. } = d {
            match target.as_str() {
                "wasi" => Some(CompileTarget::Wasi),
                "wasm" => Some(CompileTarget::Wasm),
                _ => None,
            }
        } else {
            None
        }
    });
    let run_target = target_override
        .or(module_decl_target)
        .unwrap_or(CompileTarget::Wasm);
    let options = CompileOptions {
        target: target_override,
        verbose: cli.verbose,
    };

    match cli.emit.as_str() {
        "wasm" => {
            let artifact = match compile_module(module, options) {
                Ok(a) => a,
                Err(CoreError::Diagnostics(diags)) => {
                    render_diagnostics(&diags, &source_map);
                    return Err(anyhow::anyhow!("compilation failed"));
                }
                Err(e) => return Err(anyhow::anyhow!(e.to_string())),
            };
            if let Some(out) = &cli.output {
                write_output(out, &artifact.wasm)?;
            }
            if cli.run {
                let result = run_wasm(&artifact, run_target)?;
                println!("Program exited with {result}");
            }
        }
        other => return Err(anyhow::anyhow!("unsupported emit format: {other}")),
    }

    if cli.lib {
        eprintln!("--lib is acknowledged but not yet implemented in the placeholder pipeline");
    }

    Ok(())
}

fn run_tests(args: TestArgs, verbose: bool) -> Result<()> {
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
        print!("test {name} ... ");
        match run_test_file(&file, &std_root, verbose) {
            Ok(()) => {
                println!("ok");
            }
            Err(e) => {
                println!("FAILED");
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
    let module = match loader.load(&path.to_path_buf()) {
        Ok(m) => m,
        Err(CoreError::Diagnostics(diags)) => {
            render_diagnostics(&diags, loader.source_map());
            return Err(anyhow::anyhow!("parsing failed"));
        }
        Err(e) => return Err(anyhow::anyhow!(e.to_string())),
    };
    println!("[nepl-cli] compile_module for {}", path.display());
    let artifact = match compile_module(
        module,
        CompileOptions {
            target: Some(CompileTarget::Wasi),
            verbose,
        },
    ) {
        Ok(a) => a,
        Err(CoreError::Diagnostics(diags)) => {
            render_diagnostics(&diags, loader.source_map());
            return Err(anyhow::anyhow!("compilation failed"));
        }
        Err(e) => return Err(anyhow::anyhow!(e.to_string())),
    };
    let result = run_wasm(&artifact, CompileTarget::Wasi)?;
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

fn write_output(path: &str, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = PathBuf::from(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {parent:?}"))?;
        }
    }
    fs::write(path, bytes).with_context(|| format!("failed to write output file {path}"))?;

    // Write WAT (WebAssembly Text Format) as well
    if path.ends_with(".wasm") {
        let wat_path = path.trim_end_matches(".wasm").to_string() + ".wat";
        let wat_text = print_bytes(bytes)
            .with_context(|| "failed to convert wasm to wat")?;
        fs::write(&wat_path, wat_text.as_bytes())
            .with_context(|| format!("failed to write WAT file {wat_path}"))?;
    }

    Ok(())
}

fn run_wasm(artifact: &CompilationArtifact, target: CompileTarget) -> Result<i32> {
    let engine = Engine::default();
    let module = Module::new(&engine, artifact.wasm.as_slice())
        .context("failed to compile wasm artifact")?;

    let mut linker: Linker<AllocState> = Linker::new(&engine);
    // Env prints for legacy wasm target
    linker.func_wrap("env", "print_i32", |x: i32| {
        println!("{x}");
    })?;
    linker.func_wrap(
        "env",
        "print_str",
        |caller: Caller<'_, AllocState>, ptr: i32| {
            let memory = caller
                .get_export("memory")
                .and_then(|e| e.into_memory())
                .expect("memory export not found");
            let data = memory.data(&caller);
            let offset = ptr as usize;
            if offset + 4 > data.len() {
                return;
            }
            let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            let start = offset + 4;
            if start + len > data.len() {
                return;
            }
            let bytes = &data[start..start + len];
            let text = std::str::from_utf8(bytes).unwrap_or("<invalid utf8>");
            print!("{text}");
        },
    )?;
    // Provide host-side allocator exports under `nepl_alloc` for development/runtime.
    // Simple free-list allocator using two conventions:
    // - linear memory[0..4): heap_ptr (u32)
    // - linear memory[4..8): free_list_head (u32) -- we manage this in host state as well
    linker.func_wrap(
        "nepl_alloc",
        "alloc",
        |mut caller: Caller<'_, AllocState>, size: i32| -> i32 {
            let header = 8u32; // header stores [size:u32][next:u32]
            let size = size as u32;
            let total = ((size + header + 7) / 8) * 8; // align to 8
            let mem = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                Some(m) => m,
                None => return -1,
            };

            // Snapshot memory for reads
            let data = mem.data(&caller);

            // Traverse free list
            let mut prev_ptr: Option<u32> = None;
            let mut cur = {
                // read free_head from memory if present, else from host state
                if data.len() >= 8 {
                    u32::from_le_bytes(data[4..8].try_into().unwrap())
                } else {
                    caller.data().free_head
                }
            };
            while cur != 0 {
                if (cur as usize) + 4 > data.len() {
                    break;
                }
                let blk_sz =
                    u32::from_le_bytes(data[cur as usize..cur as usize + 4].try_into().unwrap());
                let next = u32::from_le_bytes(
                    data[cur as usize + 4..cur as usize + 8].try_into().unwrap(),
                );
                if blk_sz >= total {
                    // remove from free list
                    if let Some(prev) = prev_ptr {
                        let bytes = next.to_le_bytes();
                        mem.write(&mut caller, (prev + 4) as usize, &bytes).ok();
                    } else {
                        // update head
                        let bytes = next.to_le_bytes();
                        mem.write(&mut caller, 4usize, &bytes).ok();
                    }
                    // possibly split: if remaining space large enough, create new free block
                    let remain = blk_sz - total;
                    if remain >= 16 {
                        let new_blk = cur + total;
                        let new_sz_bytes = remain.to_le_bytes();
                        mem.write(&mut caller, new_blk as usize, &new_sz_bytes).ok();
                        let new_next = next.to_le_bytes();
                        mem.write(&mut caller, (new_blk + 4) as usize, &new_next)
                            .ok();
                        // set allocated block size to total
                        let total_bytes = total.to_le_bytes();
                        mem.write(&mut caller, cur as usize, &total_bytes).ok();
                    }
                    // return payload pointer
                    let payload = cur + header;
                    return payload as i32;
                }
                prev_ptr = Some(cur);
                cur = next;
            }

            // No free block found — bump allocate from heap_ptr at addr 0
            let data2 = mem.data(&caller);
            if data2.len() < 4 {
                return 0;
            }
            let heap_ptr = u32::from_le_bytes(data2[0..4].try_into().unwrap());
            let alloc_start = ((heap_ptr + 7) / 8) * 8;
            let new_heap = alloc_start.saturating_add(total);
            if new_heap as usize > data2.len() {
                // Out of memory in host-managed environment — failure
                return 0;
            }
            // write header size
            let total_bytes = total.to_le_bytes();
            mem.write(&mut caller, alloc_start as usize, &total_bytes)
                .ok();
            // store new heap_ptr
            let nb = new_heap.to_le_bytes();
            mem.write(&mut caller, 0usize, &nb).ok();
            return (alloc_start + header) as i32;
        },
    )?;

    linker.func_wrap(
        "nepl_alloc",
        "dealloc",
        |mut caller: Caller<'_, AllocState>, ptr: i32, size: i32| {
            let header = 8u32;
            let ptr = ptr as u32;
            let _size = size as u32;
            let mem = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                Some(m) => m,
                None => return,
            };
            if ptr < header {
                return;
            }
            let header_ptr = ptr - header;
            // read current free_head
            let data = mem.data(&caller);
            let cur_head = if data.len() >= 8 {
                u32::from_le_bytes(data[4..8].try_into().unwrap())
            } else {
                caller.data().free_head
            };
            // write block header: size and next
            let sz_bytes = ((_size + header + 7) / 8 * 8).to_le_bytes();
            mem.write(&mut caller, header_ptr as usize, &sz_bytes).ok();
            let next_bytes = cur_head.to_le_bytes();
            mem.write(&mut caller, (header_ptr + 4) as usize, &next_bytes)
                .ok();
            // update free_head in memory
            mem.write(&mut caller, 4usize, &header_ptr.to_le_bytes())
                .ok();
        },
    )?;

    linker.func_wrap(
        "nepl_alloc",
        "realloc",
        |mut caller: Caller<'_, AllocState>, ptr: i32, old_size: i32, new_size: i32| -> i32 {
            let ptr = ptr as u32;
            let old = old_size as u32;
            let new = new_size as u32;
            let mem = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                Some(m) => m,
                None => return -1,
            };
            // allocate new block
            let header = 8u32;
            let total_new = ((new + header + 7) / 8) * 8;
            let data = mem.data(&caller);
            let heap_ptr = if data.len() >= 4 {
                u32::from_le_bytes(data[0..4].try_into().unwrap())
            } else {
                0
            };
            let alloc_start = ((heap_ptr + 7) / 8) * 8;
            let new_heap = alloc_start.saturating_add(total_new);
            if new_heap as usize > data.len() {
                return 0;
            }
            mem.write(&mut caller, alloc_start as usize, &total_new.to_le_bytes())
                .ok();
            mem.write(&mut caller, 0usize, &new_heap.to_le_bytes()).ok();
            let new_ptr = alloc_start + header;
            // copy min(old,new)
            let copy_len = core::cmp::min(old, new) as usize;
            if copy_len > 0 {
                let src = ptr as usize;
                let dst = new_ptr as usize;
                let snapshot = mem.data(&caller).to_vec();
                if src + copy_len <= snapshot.len() && dst + copy_len <= snapshot.len() {
                    let slice = &snapshot[src..src + copy_len];
                    mem.write(&mut caller, dst, slice).ok();
                }
            }
            // dealloc old
            if ptr != 0 {
                let hdr = ptr - header;
                let sz = if (hdr as usize) + 4 <= mem.data(&caller).len() {
                    u32::from_le_bytes(
                        mem.data(&caller)[hdr as usize..hdr as usize + 4]
                            .try_into()
                            .unwrap(),
                    )
                } else {
                    0
                };
                let sz_bytes = sz.to_le_bytes();
                // push to free list
                let cur_head = if mem.data(&caller).len() >= 8 {
                    u32::from_le_bytes(mem.data(&caller)[4..8].try_into().unwrap())
                } else {
                    caller.data().free_head
                };
                mem.write(&mut caller, (hdr + 4) as usize, &cur_head.to_le_bytes())
                    .ok();
                mem.write(&mut caller, hdr as usize, &sz_bytes).ok();
                mem.write(&mut caller, 4usize, &hdr.to_le_bytes()).ok();
            }
            new_ptr as i32
        },
    )?;

    if matches!(target, CompileTarget::Wasi) {
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "fd_read",
            |mut caller: Caller<'_, AllocState>,
             fd: i32,
             iovs: i32,
             iovs_len: i32,
             nread: i32|
             -> i32 {
                if fd != 0 {
                    return 8;
                }
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return 21,
                };
                let data_snapshot = memory.data(&caller).to_vec();
                let mut total = 0usize;
                let mut offset = iovs as usize;
                let count = if iovs_len > 0 { iovs_len as usize } else { 0 };
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
                if let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                    let bytes = (total as u32).to_le_bytes();
                    if (nread as usize) + 4 <= mem.data(&caller).len() {
                        mem.write(&mut caller, nread as usize, &bytes).ok();
                    }
                }
                0
            },
        )?;
        // Minimal wasi fd_write implementation for stdout (fd 1)
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "fd_write",
            |caller: Caller<'_, AllocState>,
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
                    match std::str::from_utf8(slice) {
                        Ok(s) => {
                            print!("{s}");
                        }
                        Err(_) => {
                            return 21;
                        }
                    }
                    total += len;
                }
                // write nwritten
                if let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                    let mut caller = caller;
                    let bytes = (total as u32).to_le_bytes();
                    if (nwritten as usize) + 4 <= mem.data(&caller).len() {
                        mem.write(&mut caller, nwritten as usize, &bytes).ok();
                    }
                }
                0
            },
        )?;
    } else {
        // If wasm target imports WASI, warn and fail fast
        for import in module.imports() {
            if import.module() == "wasi_snapshot_preview1" {
                return Err(anyhow::anyhow!(
                    "module requires WASI (import {}::{}) – re-run with --target wasi",
                    import.module(),
                    import.name()
                ));
            }
        }
    }
    let mut store = Store::new(
        &engine,
        AllocState {
            free_head: 0,
            stdin: Vec::new(),
            stdin_pos: 0,
            stdin_eof: false,
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
            let caret_pos = col;
            eprintln!(
                "       | {spaces}{carets}",
                spaces = " ".repeat(caret_pos),
                carets = "^".repeat(primary.span.len().max(1) as usize)
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
        assert_eq!(cli.emit, "wasm");
        assert!(cli.run);
        assert!(cli.output.is_none());
    }
}
