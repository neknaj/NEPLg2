use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use nepl_core::{
    compile_module,
    diagnostic::{Diagnostic, Severity},
    error::CoreError,
    loader::{Loader, SourceMap},
    CompilationArtifact, CompileOptions, CompileTarget,
};
use wasmi::{Caller, Engine, Linker, Module, Store};

/// コマンドライン引数を定義するための構造体
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
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

    #[arg(long, value_name = "TARGET", default_value = "wasm", value_parser = ["wasm", "wasi"], help = "Compilation target: wasm or wasi")]
    target: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    execute(cli)
}

fn execute(cli: Cli) -> Result<()> {
    if !cli.run && cli.output.is_none() {
        return Err(anyhow::anyhow!("Either --run or --output is required"));
    }
    let load_result = match cli.input {
        Some(path) => {
            let loader = Loader::new(stdlib_root()?);
            loader
                .load(&PathBuf::from(path))
                .map_err(|e| anyhow::anyhow!(e.to_string()))?
        }
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            let loader = Loader::new(stdlib_root()?);
            loader
                .load_inline(PathBuf::from("<stdin>"), buffer)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?
        }
    };
    // Auto-upgrade to WASI if stdio is imported and user did not explicitly pick wasi.
    let has_stdio_import = load_result
        .module
        .directives
        .iter()
        .any(|d| matches!(d, nepl_core::ast::Directive::Import { path, .. } if path == "std/stdio"));
    let module = load_result.module;
    let source_map = load_result.source_map;

    let mut target = match cli.target.as_str() {
        "wasi" => CompileTarget::Wasi,
        _ => CompileTarget::Wasm,
    };
    if matches!(target, CompileTarget::Wasm) && has_stdio_import {
        target = CompileTarget::Wasi;
    }
    let options = CompileOptions { target };

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
                let result = run_wasm(&artifact, target)?;
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

fn write_output(path: &str, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = PathBuf::from(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {parent:?}"))?;
        }
    }
    fs::write(path, bytes).with_context(|| format!("failed to write output file {path}"))?;
    Ok(())
}

fn run_wasm(artifact: &CompilationArtifact, target: CompileTarget) -> Result<i32> {
    let engine = Engine::default();
    let module = Module::new(&engine, artifact.wasm.as_slice())
        .context("failed to compile wasm artifact")?;

    let mut linker = Linker::new(&engine);
    // Env prints for legacy wasm target
    linker.func_wrap("env", "print_i32", |x: i32| {
        println!("{x}");
    })?;
    linker.func_wrap(
        "env",
        "print_str",
        |mut caller: Caller<'_, ()>, ptr: i32| {
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
    if matches!(target, CompileTarget::Wasi) {
        // Minimal wasi fd_write implementation for stdout (fd 1)
        linker.func_wrap(
            "wasi_snapshot_preview1",
            "fd_write",
            |mut caller: Caller<'_, ()>, fd: i32, iovs: i32, iovs_len: i32, nwritten: i32| -> i32 {
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
    let mut store = Store::new(&engine, ());
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
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib"))
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
            eprintln!("  {line_num:>4} | {text}", line_num = line + 1, text = line_str);
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
            let msg = label
                .message
                .as_ref()
                .map(|m| m.as_str())
                .unwrap_or("");
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
