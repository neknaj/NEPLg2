use std::collections::HashSet;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use nepl_core::span::FileId;
use nepl_core::{compile_wasm, CompilationArtifact, CompileOptions, CompileTarget};
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
    let source = match cli.input {
        Some(path) => {
            let root = Path::new(&path)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            read_with_imports(
                Path::new(&path),
                &root,
                &stdlib_root()?,
                &mut HashSet::new(),
            )?
        }
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };

    let file_id = FileId(0);

    let target = match cli.target.as_str() {
        "wasi" => CompileTarget::Wasi,
        _ => CompileTarget::Wasm,
    };
    let options = CompileOptions { target };

    match cli.emit.as_str() {
        "wasm" => {
            let artifact =
                compile_wasm(file_id, &source, options).map_err(|e| anyhow::anyhow!("{e:?}"))?;
            if let Some(out) = &cli.output {
                write_output(out, &artifact.wasm)?;
            }
            if cli.run {
                if matches!(target, CompileTarget::Wasi) {
                    return Err(anyhow::anyhow!("--run with target=wasi is not supported in embedded runner; use a WASI runtime (e.g., wasmtime)"));
                }
                let result = run_wasm(&artifact)?;
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

fn run_wasm(artifact: &CompilationArtifact) -> Result<i32> {
    let engine = Engine::default();
    let module = Module::new(&engine, artifact.wasm.as_slice())
        .context("failed to compile wasm artifact")?;
    let mut linker = Linker::new(&engine);
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
                panic!("print_str: pointer out of bounds");
            }
            let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            let start = offset + 4;
            if start + len > data.len() {
                panic!("print_str: slice out of bounds");
            }
            let bytes = &data[start..start + len];
            let text = std::str::from_utf8(bytes).unwrap_or("<invalid utf8>");
            print!("{text}");
        },
    )?;
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

fn read_with_imports(
    path: &Path,
    base: &Path,
    stdlib_root: &Path,
    seen: &mut HashSet<PathBuf>,
) -> Result<String> {
    let canon = fs::canonicalize(path).with_context(|| format!("canonicalize {:?}", path))?;
    if !seen.insert(canon.clone()) {
        return Err(anyhow::anyhow!("circular import detected: {:?}", path));
    }
    let content =
        fs::read_to_string(&canon).with_context(|| format!("failed to read {:?}", path))?;
    let mut out = String::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#import") {
            // format: #import "path"
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    let rel = &trimmed[start + 1..start + 1 + end];
                    let mut import_path = if rel.starts_with("std/") {
                        stdlib_root.join(rel)
                    } else {
                        base.join(rel)
                    };
                    if import_path.extension().is_none() {
                        import_path = import_path.with_extension("nepl");
                    }
                    let imported = read_with_imports(
                        &import_path,
                        import_path.parent().unwrap_or(base),
                        stdlib_root,
                        seen,
                    )?;
                    out.push_str(&imported);
                    out.push('\n');
                    continue;
                }
            }
            return Err(anyhow::anyhow!("invalid #import line: {}", line));
        } else if trimmed.starts_with("#use") {
            // inlined import makes #use unnecessary; skip line
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    seen.remove(&canon);
    Ok(out)
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
