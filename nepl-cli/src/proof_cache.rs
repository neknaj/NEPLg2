//! Disk cache for Resource summary proof artifacts used by native `--check`.
//!
//! This cache stores `.neplproof` bytes keyed by compiler binary, target/profile,
//! input path, and stdlib root. The source text itself is intentionally not part
//! of the path key: changed function bodies should reuse still-compatible proof
//! entries and reject stale entries by their per-function body hash inside
//! `nepl-core`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use nepl_core::{resource::ResourceSummaryProofArtifact, BuildProfile, CompileTarget};

const NEPL_PROOF_CACHE_SCHEMA: &str = "neplproof-cache-v1";
const NEPL_PROOF_CACHE_DIR_ENV: &str = "NEPL_PROOF_CACHE_DIR";
const NEPL_DISABLE_PROOF_CACHE_ENV: &str = "NEPL_DISABLE_PROOF_CACHE";
const NEPL_BOOTSTRAP_PROOF_CACHE_ENV: &str = "NEPL_BOOTSTRAP_PROOF_CACHE";

pub(crate) struct ResourceProofCacheProbe {
    path: PathBuf,
    preseed_bytes: Option<Vec<u8>>,
    bootstrap_on_miss: bool,
}

impl ResourceProofCacheProbe {
    /// Builds a probe for the `.neplproof` byte artifact used by `--check`.
    ///
    /// The artifact payload is still validated by `nepl-core` after typecheck
    /// computes the expected proof header. This path only selects a cheap
    /// candidate file before that typed boundary is available.
    pub(crate) fn new(
        input_path: &Path,
        std_root: &Path,
        target: CompileTarget,
        profile: BuildProfile,
    ) -> Option<Self> {
        if proof_cache_disabled() {
            return None;
        }
        let path = Self::path_for_input(input_path, std_root, target, profile)?;
        let preseed_bytes = fs::read(&path).ok();
        let bootstrap_on_miss =
            preseed_bytes.is_none() && std::env::var_os(NEPL_BOOTSTRAP_PROOF_CACHE_ENV).is_some();
        Some(Self {
            path,
            preseed_bytes,
            bootstrap_on_miss,
        })
    }

    pub(crate) fn has_preseed_bytes(&self) -> bool {
        self.preseed_bytes.is_some()
    }

    pub(crate) fn preseed_bytes(&self) -> Option<&[u8]> {
        self.preseed_bytes.as_deref()
    }

    /// Whether the caller should run an initial proof-collecting check.
    ///
    /// Empty Resource summary caches add overhead to one-shot cold checks. For
    /// that reason the cache is bootstrapped only when the user or benchmark
    /// explicitly asks for it, while existing proof bytes use the fail-closed
    /// `OnlyAfterAcceptedPreseed` path.
    pub(crate) fn should_bootstrap_on_miss(&self) -> bool {
        self.bootstrap_on_miss
    }

    pub(crate) fn store_artifact(&self, artifact: &ResourceSummaryProofArtifact) -> Result<()> {
        if artifact.counts().total_entries() == 0 {
            return Ok(());
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = artifact
            .to_neplproof_bytes()
            .map_err(|err| anyhow::anyhow!("failed to encode .neplproof artifact: {err:?}"))?;
        let tmp = self.path.with_extension("neplproof.tmp");
        fs::write(&tmp, bytes)?;
        match fs::remove_file(&self.path) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err.into()),
        }
        fs::rename(tmp, &self.path)?;
        Ok(())
    }

    fn path_for_input(
        input_path: &Path,
        std_root: &Path,
        target: CompileTarget,
        profile: BuildProfile,
    ) -> Option<PathBuf> {
        let mut hash = fnv1a64(NEPL_PROOF_CACHE_SCHEMA.as_bytes());
        proof_cache_hash_str(&mut hash, target_cache_tag(target));
        proof_cache_hash_str(&mut hash, profile_cache_tag(profile));
        proof_cache_hash_current_executable(&mut hash)?;
        proof_cache_hash_str(&mut hash, &stable_path_for_cache(input_path));
        proof_cache_hash_str(&mut hash, &stable_path_for_cache(std_root));
        Some(proof_cache_dir()?.join(format!("{:016x}.neplproof", hash)))
    }
}

fn proof_cache_disabled() -> bool {
    std::env::var_os(NEPL_DISABLE_PROOF_CACHE_ENV).is_some()
        || std::env::var_os("NEPL_RESOURCE_PER_FUNCTION_TIMING").is_some()
        || std::env::var_os("NEPL_RESOURCE_OP_TIMING").is_some()
}

fn proof_cache_dir() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(NEPL_PROOF_CACHE_DIR_ENV) {
        return Some(PathBuf::from(path));
    }
    std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join("target").join("neplg2").join("proof-cache-v1"))
}

fn proof_cache_hash_current_executable(hash: &mut u64) -> Option<()> {
    let exe = std::env::current_exe().ok()?;
    proof_cache_hash_str(hash, exe.to_string_lossy().as_ref());
    let metadata = fs::metadata(exe).ok()?;
    proof_cache_hash_u64(hash, metadata.len());
    proof_cache_hash_system_time(hash, metadata.modified().ok()?)?;
    Some(())
}

fn stable_path_for_cache(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}

fn proof_cache_hash_system_time(hash: &mut u64, time: SystemTime) -> Option<()> {
    let duration = time.duration_since(UNIX_EPOCH).ok()?;
    proof_cache_hash_u64(hash, duration.as_secs());
    proof_cache_hash_u64(hash, u64::from(duration.subsec_nanos()));
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

fn proof_cache_hash_str(hash: &mut u64, value: &str) {
    proof_cache_hash_u64(hash, value.len() as u64);
    *hash = fnv1a64_with_seed(*hash, value.as_bytes());
}

fn proof_cache_hash_u64(hash: &mut u64, value: u64) {
    *hash = fnv1a64_with_seed(*hash, &value.to_le_bytes());
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    fnv1a64_with_seed(0xcbf29ce484222325, bytes)
}

fn fnv1a64_with_seed(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
