use std::process::Command;

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
struct LlvmToolchainConfig {
    clang_bin: String,
    required_version_exact: Option<String>,
    required_version_prefix: Option<String>,
    required_host_os: Option<String>,
    triple_must_contain: Option<String>,
}

impl LlvmToolchainConfig {
    fn current_default() -> Self {
        let clang_bin =
            std::env::var("NEPL_LLVM_CLANG_BIN").unwrap_or_else(|_| "clang".to_string());
        let required_version_exact = std::env::var("NEPL_LLVM_CLANG_VERSION").ok().or_else(|| {
            // 既定は従来どおり 21.1.0 を要求する。
            Some("21.1.0".to_string())
        });
        let required_version_prefix = std::env::var("NEPL_LLVM_CLANG_VERSION_PREFIX").ok();
        let required_host_os = std::env::var("NEPL_LLVM_REQUIRED_HOST_OS")
            .ok()
            .or_else(|| {
                let require_linux = std::env::var("NEPL_LLVM_REQUIRE_LINUX")
                    .ok()
                    .map(|v| v != "0")
                    .unwrap_or(true);
                if require_linux {
                    Some("linux".to_string())
                } else {
                    None
                }
            });
        let triple_must_contain = std::env::var("NEPL_LLVM_TRIPLE_CONTAINS")
            .ok()
            .or_else(|| required_host_os.clone());
        Self {
            clang_bin,
            required_version_exact,
            required_version_prefix,
            required_host_os,
            triple_must_contain,
        }
    }
}

fn ensure_llvm_toolchain(cfg: &LlvmToolchainConfig) -> Result<()> {
    if let Some(required_os) = &cfg.required_host_os {
        if std::env::consts::OS != required_os {
            return Err(anyhow!(
                "llvm target requires host os '{}'; current host is '{}'",
                required_os,
                std::env::consts::OS
            ));
        }
    }

    let version_out = Command::new(&cfg.clang_bin)
        .arg("--version")
        .output()
        .with_context(|| format!("failed to execute {} --version", cfg.clang_bin))?;
    if !version_out.status.success() {
        return Err(anyhow!(
            "{} --version failed with status {}",
            cfg.clang_bin,
            version_out.status
        ));
    }
    let version_text = String::from_utf8_lossy(&version_out.stdout);
    let first_line = version_text.lines().next().unwrap_or_default().trim();
    if let Some(exact) = &cfg.required_version_exact {
        if !first_line.contains(&format!("clang version {}", exact)) {
            return Err(anyhow!(
                "llvm target requires clang version {}, but got: {}",
                exact,
                first_line,
            ));
        }
    }
    if let Some(prefix) = &cfg.required_version_prefix {
        let needle = format!("clang version {}", prefix);
        if !first_line.contains(&needle) {
            return Err(anyhow!(
                "llvm target requires clang version prefix '{}', but got: {}",
                prefix,
                first_line
            ));
        }
    }

    let triple_out = Command::new(&cfg.clang_bin)
        .arg("-dumpmachine")
        .output()
        .with_context(|| format!("failed to execute {} -dumpmachine", cfg.clang_bin))?;
    if !triple_out.status.success() {
        return Err(anyhow!(
            "{} -dumpmachine failed with status {}",
            cfg.clang_bin,
            triple_out.status
        ));
    }
    let triple = String::from_utf8_lossy(&triple_out.stdout)
        .trim()
        .to_string();
    if let Some(needle) = &cfg.triple_must_contain {
        if !triple.contains(needle) {
            return Err(anyhow!(
                "llvm target requires clang triple containing '{}', but got: {}",
                needle,
                triple,
            ));
        }
    }
    Ok(())
}

/// LLVM toolchain が利用可能かを検証する。
///
/// 既定値は `clang 21.1.0 + linux native`。
/// 以下の環境変数で条件を調整できる:
/// - `NEPL_LLVM_CLANG_BIN`
/// - `NEPL_LLVM_CLANG_VERSION` (exact match)
/// - `NEPL_LLVM_CLANG_VERSION_PREFIX` (prefix match)
/// - `NEPL_LLVM_REQUIRED_HOST_OS`
/// - `NEPL_LLVM_REQUIRE_LINUX`
/// - `NEPL_LLVM_TRIPLE_CONTAINS`
pub fn ensure_llvm_toolchain_from_env() -> Result<()> {
    let cfg = LlvmToolchainConfig::current_default();
    ensure_llvm_toolchain(&cfg)
}
