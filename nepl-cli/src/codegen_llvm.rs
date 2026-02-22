use std::process::Command;

use anyhow::{anyhow, Context, Result};
use nepl_core::ast::{FnBody, Module, Stmt};

#[derive(Debug, Clone)]
struct LlvmToolchainRequirement {
    clang_version: String,
    require_linux_native: bool,
    triple_must_contain: String,
}

impl LlvmToolchainRequirement {
    fn current_default() -> Self {
        Self {
            clang_version: std::env::var("NEPL_LLVM_CLANG_VERSION")
                .unwrap_or_else(|_| "21.1.0".to_string()),
            require_linux_native: std::env::var("NEPL_LLVM_REQUIRE_LINUX")
                .ok()
                .map(|v| v != "0")
                .unwrap_or(true),
            triple_must_contain: std::env::var("NEPL_LLVM_TRIPLE_CONTAINS")
                .unwrap_or_else(|_| "linux".to_string()),
        }
    }
}

/// clang 21.1.0 の Linux native toolchain が利用可能かを検証する。
pub fn ensure_clang_21_linux_native() -> Result<()> {
    let req = LlvmToolchainRequirement::current_default();

    if req.require_linux_native && std::env::consts::OS != "linux" {
        return Err(anyhow!(
            "llvm target requires linux native host; current host is {}",
            std::env::consts::OS
        ));
    }

    let version_out = Command::new("clang")
        .arg("--version")
        .output()
        .context("failed to execute clang --version")?;
    if !version_out.status.success() {
        return Err(anyhow!(
            "clang --version failed with status {}",
            version_out.status
        ));
    }
    let version_text = String::from_utf8_lossy(&version_out.stdout);
    let first_line = version_text.lines().next().unwrap_or_default().trim();
    if !first_line.contains(&format!("clang version {}", req.clang_version)) {
        return Err(anyhow!(
            "llvm target requires clang version {}, but got: {}",
            req.clang_version,
            first_line,
        ));
    }

    let triple_out = Command::new("clang")
        .arg("-dumpmachine")
        .output()
        .context("failed to execute clang -dumpmachine")?;
    if !triple_out.status.success() {
        return Err(anyhow!(
            "clang -dumpmachine failed with status {}",
            triple_out.status
        ));
    }
    let triple = String::from_utf8_lossy(&triple_out.stdout).trim().to_string();
    if !triple.contains(&req.triple_must_contain) {
        return Err(anyhow!(
            "llvm target requires clang triple containing '{}', but got: {}",
            req.triple_must_contain,
            triple,
        ));
    }
    Ok(())
}

/// `#llvmir` ブロックを連結して LLVM IR テキストを生成する。
///
/// 現段階では、LLVM target は手書き `#llvmir` ブロックを入力として扱う。
/// 通常の NEPL 関数本体（Parsed/Wasm）は LLVM backend 未実装のためエラーにする。
pub fn emit_ll_from_module(module: &Module) -> Result<String> {
    let mut out = String::new();
    let mut saw_llvmir = false;

    for stmt in &module.root.items {
        match stmt {
            Stmt::LlvmIr(block) => {
                for line in &block.lines {
                    out.push_str(line);
                    out.push('\n');
                }
                saw_llvmir = true;
                out.push('\n');
            }
            Stmt::FnDef(def) => match &def.body {
                FnBody::LlvmIr(block) => {
                    for line in &block.lines {
                        out.push_str(line);
                        out.push('\n');
                    }
                    saw_llvmir = true;
                    out.push('\n');
                }
                FnBody::Parsed(_) | FnBody::Wasm(_) => {
                    return Err(anyhow!(
                        "llvm target currently supports only functions written with #llvmir blocks; function '{}' has a non-llvmir body",
                        def.name.name
                    ));
                }
            },
            _ => {}
        }
    }

    if !saw_llvmir {
        return Err(anyhow!(
            "llvm target requires at least one #llvmir block in module/function body"
        ));
    }

    Ok(out)
}
