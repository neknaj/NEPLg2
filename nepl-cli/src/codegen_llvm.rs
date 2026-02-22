use std::process::Command;

use anyhow::{anyhow, Context, Result};

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

fn clang_bin() -> String {
    std::env::var("NEPL_LLVM_CLANG_BIN").unwrap_or_else(|_| "clang".to_string())
}

/// clang 21.1.0 の Linux native toolchain が利用可能かを検証する。
pub fn ensure_clang_21_linux_native() -> Result<()> {
    let req = LlvmToolchainRequirement::current_default();
    let clang = clang_bin();

    if req.require_linux_native && std::env::consts::OS != "linux" {
        return Err(anyhow!(
            "llvm target requires linux native host; current host is {}",
            std::env::consts::OS
        ));
    }

    let version_out = Command::new(&clang)
        .arg("--version")
        .output()
        .with_context(|| format!("failed to execute {} --version", clang))?;
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

    let triple_out = Command::new(&clang)
        .arg("-dumpmachine")
        .output()
        .with_context(|| format!("failed to execute {} -dumpmachine", clang))?;
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
