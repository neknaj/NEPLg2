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
        let strict = env_flag("NEPL_LLVM_STRICT").unwrap_or(false);
        let legacy_require_linux = env_flag("NEPL_LLVM_REQUIRE_LINUX").unwrap_or(false);
        let required_version_exact = std::env::var("NEPL_LLVM_CLANG_VERSION")
            .ok()
            .or_else(|| strict.then(|| "21.1.0".to_string()));
        let required_version_prefix = std::env::var("NEPL_LLVM_CLANG_VERSION_PREFIX").ok();
        let required_host_os = std::env::var("NEPL_LLVM_REQUIRED_HOST_OS")
            .ok()
            .or_else(|| legacy_require_linux.then(|| "linux".to_string()));
        let triple_must_contain = std::env::var("NEPL_LLVM_TRIPLE_CONTAINS")
            .ok()
            .or_else(|| legacy_require_linux.then(|| "linux".to_string()));
        Self {
            clang_bin,
            required_version_exact,
            required_version_prefix,
            required_host_os,
            triple_must_contain,
        }
    }
}

fn env_flag(name: &str) -> Option<bool> {
    let value = std::env::var(name).ok()?;
    let normalized = value.trim().to_ascii_lowercase();
    Some(!matches!(
        normalized.as_str(),
        "" | "0" | "false" | "no" | "off"
    ))
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
/// 既定値では `clang` が実行でき、target triple を取得できることだけを検証する。
/// CI などで厳密な toolchain を固定したい場合は以下の環境変数で条件を明示する:
/// - `NEPL_LLVM_CLANG_BIN`
/// - `NEPL_LLVM_STRICT` (`NEPL_LLVM_CLANG_VERSION` 未指定時に 21.1.0 exact match)
/// - `NEPL_LLVM_CLANG_VERSION` (exact match)
/// - `NEPL_LLVM_CLANG_VERSION_PREFIX` (prefix match)
/// - `NEPL_LLVM_REQUIRED_HOST_OS`
/// - `NEPL_LLVM_REQUIRE_LINUX` (legacy opt-in)
/// - `NEPL_LLVM_TRIPLE_CONTAINS`
pub fn ensure_llvm_toolchain_from_env() -> Result<()> {
    let cfg = LlvmToolchainConfig::current_default();
    ensure_llvm_toolchain(&cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    const LLVM_ENV_KEYS: [&str; 7] = [
        "NEPL_LLVM_CLANG_BIN",
        "NEPL_LLVM_STRICT",
        "NEPL_LLVM_CLANG_VERSION",
        "NEPL_LLVM_CLANG_VERSION_PREFIX",
        "NEPL_LLVM_REQUIRED_HOST_OS",
        "NEPL_LLVM_REQUIRE_LINUX",
        "NEPL_LLVM_TRIPLE_CONTAINS",
    ];

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvSnapshot {
        values: Vec<(&'static str, Option<String>)>,
    }

    impl EnvSnapshot {
        fn capture() -> Self {
            Self {
                values: LLVM_ENV_KEYS
                    .iter()
                    .map(|key| (*key, std::env::var(key).ok()))
                    .collect(),
            }
        }
    }

    impl Drop for EnvSnapshot {
        fn drop(&mut self) {
            for key in LLVM_ENV_KEYS {
                std::env::remove_var(key);
            }
            for (key, value) in &self.values {
                if let Some(value) = value {
                    std::env::set_var(key, value);
                }
            }
        }
    }

    fn with_llvm_env<R>(vars: &[(&str, &str)], f: impl FnOnce() -> R) -> R {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let _snapshot = EnvSnapshot::capture();
        for key in LLVM_ENV_KEYS {
            std::env::remove_var(key);
        }
        for (key, value) in vars {
            std::env::set_var(key, value);
        }
        f()
    }

    #[test]
    fn default_config_is_relaxed() {
        with_llvm_env(&[], || {
            let cfg = LlvmToolchainConfig::current_default();

            assert_eq!(cfg.clang_bin, "clang");
            assert_eq!(cfg.required_version_exact, None);
            assert_eq!(cfg.required_version_prefix, None);
            assert_eq!(cfg.required_host_os, None);
            assert_eq!(cfg.triple_must_contain, None);
        });
    }

    #[test]
    fn strict_mode_keeps_ci_version_contract() {
        with_llvm_env(&[("NEPL_LLVM_STRICT", "1")], || {
            let cfg = LlvmToolchainConfig::current_default();

            assert_eq!(cfg.required_version_exact.as_deref(), Some("21.1.0"));
            assert_eq!(cfg.required_host_os, None);
            assert_eq!(cfg.triple_must_contain, None);
        });
    }

    #[test]
    fn explicit_toolchain_constraints_are_preserved() {
        with_llvm_env(
            &[
                ("NEPL_LLVM_CLANG_BIN", "custom-clang"),
                ("NEPL_LLVM_CLANG_VERSION", "20.1.2"),
                ("NEPL_LLVM_CLANG_VERSION_PREFIX", "20."),
                ("NEPL_LLVM_REQUIRED_HOST_OS", "windows"),
                ("NEPL_LLVM_TRIPLE_CONTAINS", "msvc"),
            ],
            || {
                let cfg = LlvmToolchainConfig::current_default();

                assert_eq!(cfg.clang_bin, "custom-clang");
                assert_eq!(cfg.required_version_exact.as_deref(), Some("20.1.2"));
                assert_eq!(cfg.required_version_prefix.as_deref(), Some("20."));
                assert_eq!(cfg.required_host_os.as_deref(), Some("windows"));
                assert_eq!(cfg.triple_must_contain.as_deref(), Some("msvc"));
            },
        );
    }

    #[test]
    fn legacy_require_linux_is_explicit_opt_in() {
        with_llvm_env(&[("NEPL_LLVM_REQUIRE_LINUX", "1")], || {
            let cfg = LlvmToolchainConfig::current_default();

            assert_eq!(cfg.required_host_os.as_deref(), Some("linux"));
            assert_eq!(cfg.triple_must_contain.as_deref(), Some("linux"));
        });
    }
}
