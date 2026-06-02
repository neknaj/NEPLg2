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
use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use nepl_core::{
    resource::{ResourceSummaryProofArtifact, ResourceSummaryValueCacheStats},
    BuildProfile, CompileTarget, ResourceSummaryProofArtifactPreseedReport,
};

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
        let stage = proof_cache_stage_start();
        if proof_cache_disabled() {
            proof_cache_stage_finish("proof_cache_new_disabled", stage);
            return None;
        }
        let path_stage = proof_cache_stage_start();
        let Some(path) = Self::path_for_input(input_path, std_root, target, profile) else {
            proof_cache_stage_finish("proof_cache_path", path_stage);
            proof_cache_stage_finish("proof_cache_new_no_path", stage);
            return None;
        };
        proof_cache_stage_finish("proof_cache_path", path_stage);
        let read_stage = proof_cache_stage_start();
        let preseed_bytes = fs::read(&path).ok();
        proof_cache_stage_finish("proof_cache_read", read_stage);
        let bootstrap_on_miss =
            preseed_bytes.is_none() && std::env::var_os(NEPL_BOOTSTRAP_PROOF_CACHE_ENV).is_some();
        proof_cache_stage_finish("proof_cache_new", stage);
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

    /// Returns whether the proof artifact should be rewritten after a check.
    ///
    /// A preseeded run that only replayed existing proof information does not
    /// make the on-disk artifact more authoritative. Rewriting the same
    /// multi-megabyte payload in that case only adds host I/O and also widens
    /// the window for concurrent native checks to race on the same file. A
    /// bootstrap run still stores unconditionally because it has no prior bytes
    /// to reuse. A preseeded run stores again only when Resource checking
    /// observed new stable proof information that can make a later run faster.
    pub(crate) fn should_store_artifact_after_check(
        &self,
        preseed_report: Option<ResourceSummaryProofArtifactPreseedReport>,
        stats: ResourceSummaryValueCacheStats,
    ) -> bool {
        if self.preseed_bytes.is_none() {
            return true;
        }
        let Some(report) = preseed_report else {
            return true;
        };
        if !report.has_usable_entries()
            || report.rejected_conflict_entries > 0
            || report.compatibility_reject.is_some()
            || report.codec_error.is_some()
        {
            return true;
        }
        proof_stats_observed_new_stable_entries(stats)
            || proof_stats_observed_recomputed_stable_work(stats)
    }

    pub(crate) fn store_artifact(&self, artifact: &ResourceSummaryProofArtifact) -> Result<()> {
        if artifact.counts().total_entries() == 0 {
            return Ok(());
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let encode_stage = proof_cache_stage_start();
        let bytes = artifact
            .to_neplproof_bytes()
            .map_err(|err| anyhow::anyhow!("failed to encode .neplproof artifact: {err:?}"))?;
        proof_cache_stage_finish("proof_cache_encode", encode_stage);
        let tmp = self.path.with_extension("neplproof.tmp");
        let write_stage = proof_cache_stage_start();
        fs::write(&tmp, bytes)?;
        match fs::remove_file(&self.path) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err.into()),
        }
        fs::rename(tmp, &self.path)?;
        proof_cache_stage_finish("proof_cache_write", write_stage);
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

fn proof_cache_stage_start() -> Option<Instant> {
    proof_cache_stage_timing_enabled().then(Instant::now)
}

fn proof_cache_stage_finish(stage: &str, start: Option<Instant>) {
    if let Some(start) = start {
        eprintln!("[cli-stage] {}={}us", stage, start.elapsed().as_micros());
    }
}

fn proof_cache_stage_timing_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("NEPL_CLI_STAGE_TIMING").is_some()
            || std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some()
    })
}

fn proof_stats_observed_new_stable_entries(stats: ResourceSummaryValueCacheStats) -> bool {
    stats.resource_summary_value_drop_traversal_forall_stores > 0
        || stats.resource_summary_value_raw_alias_return_entry_stores > 0
        || stats.resource_summary_value_i32_scalar_return_facts_stores > 0
        || stats.resource_summary_value_initialized_function_check_stores > 0
        || stats.resource_summary_value_owner_obligation_check_stores > 0
        || stats.resource_summary_value_raw_init_param_facts_stores > 0
}

fn proof_stats_observed_recomputed_stable_work(stats: ResourceSummaryValueCacheStats) -> bool {
    stats.resource_summary_value_recomputed_ops > 0
        || stats.resource_summary_value_drop_traversal_forall_recomputed_ops > 0
        || stats.resource_summary_value_raw_alias_return_entry_recomputed_ops > 0
        || stats.resource_summary_value_i32_scalar_return_facts_recomputed_ops > 0
        || stats.resource_summary_value_raw_init_param_facts_recomputed_ops > 0
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

#[cfg(test)]
mod tests {
    use super::*;

    fn probe_with_preseed(preseeded: bool) -> ResourceProofCacheProbe {
        ResourceProofCacheProbe {
            path: PathBuf::from("cache.neplproof"),
            preseed_bytes: preseeded.then(Vec::new),
            bootstrap_on_miss: !preseeded,
        }
    }

    #[test]
    fn proof_cache_bootstrap_store_does_not_depend_on_stats() {
        let probe = probe_with_preseed(false);

        assert!(probe
            .should_store_artifact_after_check(None, ResourceSummaryValueCacheStats::default()));
    }

    #[test]
    fn proof_cache_preseed_replay_without_new_entries_skips_rewrite() {
        let probe = probe_with_preseed(true);
        let mut stats = ResourceSummaryValueCacheStats::default();
        stats.resource_summary_value_replay_hits = 100;
        stats.resource_summary_value_lazy_pass_hits = 20;

        assert!(!probe.should_store_artifact_after_check(Some(usable_preseed_report()), stats));
    }

    #[test]
    fn proof_cache_preseed_stores_when_new_stable_entries_appear() {
        let probe = probe_with_preseed(true);
        let mut stats = ResourceSummaryValueCacheStats::default();
        stats.resource_summary_value_owner_obligation_check_stores = 1;

        assert!(probe.should_store_artifact_after_check(Some(usable_preseed_report()), stats));
    }

    #[test]
    fn proof_cache_preseed_stores_when_recomputed_work_may_refresh_snapshots() {
        let probe = probe_with_preseed(true);
        let mut stats = ResourceSummaryValueCacheStats::default();
        stats.resource_summary_value_recomputed_ops = 1;

        assert!(probe.should_store_artifact_after_check(Some(usable_preseed_report()), stats));
    }

    #[test]
    fn proof_cache_preseed_reject_rewrites_artifact() {
        let probe = probe_with_preseed(true);
        let report = ResourceSummaryProofArtifactPreseedReport {
            codec_error: Some(nepl_core::resource::ResourceSummaryProofArtifactCodecError::Decode),
            ..ResourceSummaryProofArtifactPreseedReport::default()
        };

        assert!(probe.should_store_artifact_after_check(
            Some(report),
            ResourceSummaryValueCacheStats::default()
        ));
    }

    fn usable_preseed_report() -> ResourceSummaryProofArtifactPreseedReport {
        ResourceSummaryProofArtifactPreseedReport {
            accepted_entries: 1,
            ..ResourceSummaryProofArtifactPreseedReport::default()
        }
    }
}
