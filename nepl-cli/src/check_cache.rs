//! Exact success cache for native `--check`.
//!
//! This module deliberately stores only the fact that a previous full check
//! succeeded for an identical compiler binary, target/profile, stdlib root, and
//! loaded source set. It is not a Resource IR proof cache. Any mismatch or
//! unreadable manifest must fall back to the normal compiler pipeline so the
//! cache never weakens static checking.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use nepl_core::{loader::SourceMap, BuildProfile, CompileTarget};

const NEPL_CHECK_CACHE_SCHEMA: &str = "neplcheck-v1";
const NEPL_CHECK_CACHE_DIR_ENV: &str = "NEPL_CHECK_CACHE_DIR";

pub(crate) struct ExactCheckCacheProbe {
    path: PathBuf,
    fingerprint: String,
    entries: Vec<ExactCheckCacheEntry>,
}

struct ExactCheckCacheEntry {
    path: String,
    source_len: u64,
    source_hash: u64,
}

impl ExactCheckCacheProbe {
    /// Builds the manifest that will be written after a successful full check.
    ///
    /// The source map is already the authority for the files the loader actually
    /// read. Sorting paths makes the fingerprint deterministic across loader
    /// traversal order changes.
    pub(crate) fn new(
        source_map: &SourceMap,
        _target: CompileTarget,
        _profile: BuildProfile,
        precomputed_path: Option<PathBuf>,
    ) -> Option<Self> {
        if exact_check_cache_disabled() {
            return None;
        }
        let path = precomputed_path?;
        let mut hash = fnv1a64(NEPL_CHECK_CACHE_SCHEMA.as_bytes());
        let mut source_entries = source_map
            .iter_paths()
            .filter_map(|(file_id, path)| {
                source_map
                    .get(file_id)
                    .map(|source| (path.as_str().to_owned(), source.as_bytes()))
            })
            .collect::<Vec<_>>();
        source_entries.sort_by(|left, right| left.0.cmp(&right.0));
        let mut entries = Vec::with_capacity(source_entries.len());
        exact_check_hash_u64(&mut hash, source_entries.len() as u64);
        for (path, source) in source_entries {
            let source_hash = fnv1a64(source);
            let source_len = source.len() as u64;
            exact_check_hash_entry(&mut hash, &path, source_len, source_hash);
            entries.push(ExactCheckCacheEntry {
                path,
                source_len,
                source_hash,
            });
        }
        let fingerprint = format!("{:016x}", hash);
        Some(Self {
            path,
            fingerprint,
            entries,
        })
    }

    pub(crate) fn hit(&self) -> bool {
        Self::hit_manifest_at(&self.path)
    }

    /// Persists only successful full-check results.
    ///
    /// A temporary file plus rename keeps interrupted writes from creating a
    /// valid-looking manifest with a truncated source set.
    pub(crate) fn store_success(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = self.path.with_extension("neplcheck.tmp");
        let mut body = format!(
            "{}\n{}\nsuccess\n{}\n",
            NEPL_CHECK_CACHE_SCHEMA,
            self.fingerprint,
            self.entries.len()
        );
        for entry in &self.entries {
            body.push_str("entry ");
            body.push_str(&entry.path.len().to_string());
            body.push(' ');
            body.push_str(&entry.source_len.to_string());
            body.push(' ');
            body.push_str(&format!("{:016x}", entry.source_hash));
            body.push('\n');
            body.push_str(&entry.path);
            body.push('\n');
        }
        fs::write(&tmp, body)?;
        match fs::remove_file(&self.path) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err.into()),
        }
        fs::rename(tmp, &self.path)?;
        Ok(())
    }

    /// Computes the pre-load manifest path from stable run inputs.
    ///
    /// The manifest payload still verifies every loaded source file. The path
    /// key only decides which candidate manifest is cheap to probe before the
    /// loader starts.
    pub(crate) fn path_for_input(
        input_path: &Path,
        std_root: &Path,
        target: CompileTarget,
        profile: BuildProfile,
    ) -> Option<PathBuf> {
        if exact_check_cache_disabled() {
            return None;
        }
        let mut hash = fnv1a64(NEPL_CHECK_CACHE_SCHEMA.as_bytes());
        exact_check_hash_str(&mut hash, "input-manifest");
        exact_check_hash_str(&mut hash, target_cache_tag(target));
        exact_check_hash_str(&mut hash, profile_cache_tag(profile));
        exact_check_hash_current_executable(&mut hash)?;
        exact_check_hash_str(&mut hash, &stable_path_for_cache(input_path));
        exact_check_hash_str(&mut hash, &stable_path_for_cache(std_root));
        Some(exact_check_cache_dir()?.join(format!("{:016x}.neplcheck", hash)))
    }

    /// Replays a manifest by reading every recorded source file and rebuilding
    /// the source-set fingerprint.
    ///
    /// The function returns `false` for every malformed or stale condition. That
    /// fail-closed behavior is the important safety property of this auxiliary
    /// cache.
    pub(crate) fn hit_manifest_at(path: &Path) -> bool {
        let Ok(stored) = fs::read_to_string(path) else {
            return false;
        };
        let mut lines = stored.lines();
        if !matches!(lines.next(), Some(schema) if schema == NEPL_CHECK_CACHE_SCHEMA) {
            return false;
        }
        let Some(fingerprint) = lines.next() else {
            return false;
        };
        if !matches!(lines.next(), Some("success")) {
            return false;
        }
        let Some(count_line) = lines.next() else {
            return false;
        };
        let Ok(count) = count_line.parse::<usize>() else {
            return false;
        };
        let mut hash = fnv1a64(NEPL_CHECK_CACHE_SCHEMA.as_bytes());
        exact_check_hash_u64(&mut hash, count as u64);
        for _ in 0..count {
            let Some(header) = lines.next() else {
                return false;
            };
            let Some(path_line) = lines.next() else {
                return false;
            };
            let Some((path_len, source_len, source_hash)) =
                parse_exact_check_cache_entry_header(header)
            else {
                return false;
            };
            if path_line.len() != path_len {
                return false;
            }
            let Ok(source) = fs::read(path_line) else {
                return false;
            };
            if source.len() as u64 != source_len || fnv1a64(&source) != source_hash {
                return false;
            }
            exact_check_hash_entry(&mut hash, path_line, source_len, source_hash);
        }
        format!("{:016x}", hash) == fingerprint
    }
}

fn exact_check_cache_disabled() -> bool {
    std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some()
        || std::env::var_os("NEPL_RESOURCE_PER_FUNCTION_TIMING").is_some()
        || std::env::var_os("NEPL_RESOURCE_OP_TIMING").is_some()
        || std::env::var_os("NEPL_DISABLE_CHECK_CACHE").is_some()
}

fn exact_check_cache_dir() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(NEPL_CHECK_CACHE_DIR_ENV) {
        return Some(PathBuf::from(path));
    }
    std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join("target").join("neplg2").join("check-cache-v1"))
}

fn exact_check_hash_current_executable(hash: &mut u64) -> Option<()> {
    let exe = std::env::current_exe().ok()?;
    exact_check_hash_str(hash, exe.to_string_lossy().as_ref());
    let metadata = fs::metadata(exe).ok()?;
    exact_check_hash_u64(hash, metadata.len());
    exact_check_hash_system_time(hash, metadata.modified().ok()?)?;
    Some(())
}

fn stable_path_for_cache(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}

fn parse_exact_check_cache_entry_header(header: &str) -> Option<(usize, u64, u64)> {
    let mut parts = header.split(' ');
    if parts.next()? != "entry" {
        return None;
    }
    let path_len = parts.next()?.parse::<usize>().ok()?;
    let source_len = parts.next()?.parse::<u64>().ok()?;
    let source_hash = u64::from_str_radix(parts.next()?, 16).ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((path_len, source_len, source_hash))
}

fn exact_check_hash_entry(hash: &mut u64, path: &str, source_len: u64, source_hash: u64) {
    exact_check_hash_str(hash, path);
    exact_check_hash_u64(hash, source_len);
    exact_check_hash_u64(hash, source_hash);
}

fn exact_check_hash_system_time(hash: &mut u64, time: SystemTime) -> Option<()> {
    let duration = time.duration_since(UNIX_EPOCH).ok()?;
    exact_check_hash_u64(hash, duration.as_secs());
    exact_check_hash_u64(hash, u64::from(duration.subsec_nanos()));
    Some(())
}

fn target_cache_tag(target: CompileTarget) -> &'static str {
    match target {
        CompileTarget::Wasm => "wasm",
        CompileTarget::Wasi => "wasi",
        CompileTarget::Wasix => "wasix",
        CompileTarget::Llvm => "llvm",
    }
}

fn profile_cache_tag(profile: BuildProfile) -> &'static str {
    match profile {
        BuildProfile::Debug => "debug",
        BuildProfile::Release => "release",
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    exact_check_hash_bytes(&mut hash, bytes);
    hash
}

fn exact_check_hash_str(hash: &mut u64, value: &str) {
    exact_check_hash_u64(hash, value.len() as u64);
    exact_check_hash_bytes(hash, value.as_bytes());
}

fn exact_check_hash_bytes(hash: &mut u64, bytes: &[u8]) {
    exact_check_hash_u64(hash, bytes.len() as u64);
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

fn exact_check_hash_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}
