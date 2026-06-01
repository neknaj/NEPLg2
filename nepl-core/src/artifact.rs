extern crate alloc;

use crate::compiler::{BuildProfile, CompileTarget};
use crate::source_cache_key::compiled_source_cache_key_part;
use crate::source_map::SourceMap;
use crate::typecheck::{
    materialized_callable_symbol_for_link_symbol, public_callable_link_symbol_stable_hash,
    PublicCallableLinkSymbol, PublicSurfaceMaterializerBlocker, PublicSurfaceShape,
    TypedPublicSignatureKind, TypedPublicSignatureTable, TypedPublicSurfaceTable,
};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

const FNV1A64_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x100000001b3;
const NEPL_META_ARTIFACT_SCHEMA_VERSION: u32 = 12;
const NEPL_META_ARTIFACT_HASH_VERSION: &str = "neplg2-neplmeta-artifact-v12";
const NEPL_META_COMPILER_IDENTITY_INPUT: &str = concat!(
    "neplg2-compiler:",
    env!("CARGO_PKG_VERSION"),
    ":neplmeta-v12"
);
const NEPL_OBJ_DIRECT_CALL_SCHEMA_VERSION: u32 = 1;
const NEPL_OBJ_DIRECT_CALL_HASH_VERSION: &str = "neplg2-neplobj-direct-call-v1";

/// `.neplmeta` が保持する module dependency surface。
///
/// この値は loader の import / prelude / include 解決結果を cross-session artifact へ
/// 渡すための安定表現である。`PathBuf`、`FileId`、`Span`、`ImportResolution` は保存せず、
/// materializer が同じ module edge を current session の loader / typecheck environment へ
/// 再投影できるだけの canonical text と enum payload に正規化する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeplMetaModuleSurface {
    pub canonical_module_path: String,
    pub default_prelude_path: String,
    pub no_prelude: bool,
    pub implicit_default_prelude: bool,
    pub dependency_edges: Vec<NeplMetaModuleDependencyEdge>,
    pub stable_hash: u64,
}

impl NeplMetaModuleSurface {
    pub fn new(
        canonical_module_path: String,
        default_prelude_path: String,
        no_prelude: bool,
        implicit_default_prelude: bool,
        mut dependency_edges: Vec<NeplMetaModuleDependencyEdge>,
    ) -> Self {
        dependency_edges.sort();
        let stable_hash = nepl_meta_module_surface_hash(
            &canonical_module_path,
            &default_prelude_path,
            no_prelude,
            implicit_default_prelude,
            &dependency_edges,
        );
        Self {
            canonical_module_path,
            default_prelude_path,
            no_prelude,
            implicit_default_prelude,
            dependency_edges,
            stable_hash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NeplMetaModuleDependencyEdge {
    pub kind: NeplMetaModuleDependencyKind,
    pub target_path: String,
    pub visibility: NeplMetaVisibility,
    pub import_clause: Option<NeplMetaImportClause>,
    pub public_reexport: bool,
    pub source_order: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NeplMetaModuleDependencyKind {
    Prelude,
    Import,
    Include,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NeplMetaVisibility {
    Pub,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NeplMetaImportClause {
    DefaultAlias,
    Alias(String),
    Open,
    Selective(Vec<NeplMetaImportItem>),
    Merge,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NeplMetaImportItem {
    pub name: String,
    pub alias: Option<String>,
    pub glob: bool,
}

/// `.neplmeta` が保持する public export / re-export projection surface。
///
/// local export は現在 module が直接公開している名前を表す。re-export projection は
/// `pub #import` / `#import pub` / include 由来の公開投影を表し、target artifact を読んだ
/// materializer が後段で展開する。glob や merge をここで推測展開せず、構造として保存して
/// fail-closed materializer の入力にする。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeplMetaExportSurface {
    pub local_exports: Vec<NeplMetaExportEntry>,
    pub reexport_projections: Vec<NeplMetaReexportProjection>,
    pub stable_hash: u64,
}

impl NeplMetaExportSurface {
    pub fn new(
        mut local_exports: Vec<NeplMetaExportEntry>,
        mut reexport_projections: Vec<NeplMetaReexportProjection>,
    ) -> Self {
        local_exports.sort();
        local_exports.dedup();
        reexport_projections.sort();
        reexport_projections.dedup();
        let stable_hash = nepl_meta_export_surface_hash(&local_exports, &reexport_projections);
        Self {
            local_exports,
            reexport_projections,
            stable_hash,
        }
    }

    pub fn from_module_and_public_surface(
        module_surface: &NeplMetaModuleSurface,
        public_surface: &TypedPublicSurfaceTable,
    ) -> Self {
        let local_exports = public_surface
            .entries
            .iter()
            .filter(|entry| entry.exported)
            .filter_map(|entry| {
                let kind = match entry.kind {
                    crate::typecheck::TypedPublicSignatureKind::Callable => {
                        NeplMetaExportKind::Callable
                    }
                    crate::typecheck::TypedPublicSignatureKind::Struct => NeplMetaExportKind::Struct,
                    crate::typecheck::TypedPublicSignatureKind::Enum => NeplMetaExportKind::Enum,
                    crate::typecheck::TypedPublicSignatureKind::Trait => NeplMetaExportKind::Trait,
                    crate::typecheck::TypedPublicSignatureKind::Impl => return None,
                };
                Some(NeplMetaExportEntry {
                    exported_name: entry.name.clone(),
                    origin_module_path: module_surface.canonical_module_path.clone(),
                    origin_name: entry.name.clone(),
                    kind,
                })
            })
            .collect::<Vec<_>>();
        let reexport_projections = module_surface
            .dependency_edges
            .iter()
            .filter(|edge| edge.public_reexport)
            .map(|edge| NeplMetaReexportProjection {
                source_order: edge.source_order,
                target_module_path: edge.target_path.clone(),
                kind: edge.kind,
                import_clause: edge.import_clause.clone(),
            })
            .collect::<Vec<_>>();
        Self::new(local_exports, reexport_projections)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NeplMetaExportEntry {
    pub exported_name: String,
    pub origin_module_path: String,
    pub origin_name: String,
    pub kind: NeplMetaExportKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NeplMetaExportKind {
    Callable,
    Struct,
    Enum,
    Trait,
}

impl NeplMetaExportKind {
    fn typed_public_signature_kind(self) -> TypedPublicSignatureKind {
        match self {
            Self::Callable => TypedPublicSignatureKind::Callable,
            Self::Struct => TypedPublicSignatureKind::Struct,
            Self::Enum => TypedPublicSignatureKind::Enum,
            Self::Trait => TypedPublicSignatureKind::Trait,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NeplMetaReexportProjection {
    pub source_order: u32,
    pub target_module_path: String,
    pub kind: NeplMetaModuleDependencyKind,
    pub import_clause: Option<NeplMetaImportClause>,
}

/// `.neplmeta` artifact の invalidation envelope。
///
/// `.neplmeta` は依存側の名前解決・型検査に必要な public interface を保存するための
/// metadata artifact である。この header には `TypeId`、`Span`、`SourceMap`、typed HIR を
/// 入れない。これらは 1 回の compile session に閉じる値であり、永続 artifact や
/// cross-session cache の authority にすると stale hit の原因になる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NeplMetaArtifactHeader {
    pub schema_version: u32,
    pub compiler_identity_hash: u64,
    pub target_hash: u64,
    pub profile_hash: u64,
    pub stdlib_content_hash: Option<u64>,
    pub source_key_hash: Option<u64>,
    pub dependency_public_surface_hash: Option<u64>,
    pub typed_public_signature_hash: u64,
    pub public_entry_count: u32,
    pub module_surface_hash: Option<u64>,
    pub module_dependency_edge_count: Option<u32>,
    pub export_surface_hash: Option<u64>,
    pub local_export_count: Option<u32>,
    pub reexport_projection_count: Option<u32>,
    pub structured_public_surface_hash: u64,
    pub structured_public_surface_entry_count: u32,
    pub source_capability_policy_set_hash: Option<u64>,
    pub private_effect_policy_hash: Option<u64>,
}

/// `.neplmeta` payload を読まずに照合できる pre-typecheck envelope。
///
/// import materializer や dependency body skip は、target module の body を typecheck する前に
/// 保存済み artifact が現在 source と同じ compile context に属するかを確認する必要がある。
/// そのため、この envelope は typed public signature や structured public surface を含めず、
/// loader / source map / compile option だけで作れる field に限定する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NeplMetaArtifactPreTypecheckEnvelope {
    pub schema_version: u32,
    pub compiler_identity_hash: u64,
    pub target_hash: u64,
    pub profile_hash: u64,
    pub stdlib_content_hash: Option<u64>,
    pub source_key_hash: u64,
    pub dependency_public_surface_hash: Option<u64>,
    pub module_surface_hash: Option<u64>,
    pub module_dependency_edge_count: Option<u32>,
    pub source_capability_policy_set_hash: Option<u64>,
    pub private_effect_policy_hash: Option<u64>,
}

/// direct call 用 `.neplobj` body fragment の invalidation key。
///
/// `.neplmeta` は公開型と名前解決 surface だけを保存するため、dependency callable が
/// direct call として codegen 入力に残る場合は別途 `.neplobj` 相当の body / backend
/// fragment authority が必要になる。この key はその最小境界を固定するものであり、
/// relocatable object file 全体の仕様ではない。
///
/// key には stable link symbol、target source key、selected callable body hash、
/// generic instantiation hash、dependency public surface hash、source capability policy、
/// backend feature set を含める。これにより、body-only edit、generic 具体化、target/profile、
/// source capability の変更を別 artifact として扱える。`TypeId`、`Span`、`FileId`、
/// `SourceMap`、typed HIR body は session-local な値なので保存しない。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NeplObjDirectCallKey {
    pub schema_version: u32,
    pub compiler_identity_hash: u64,
    pub target_hash: u64,
    pub profile_hash: u64,
    pub stdlib_content_hash: Option<u64>,
    pub backend_feature_set_hash: u64,
    pub link_symbol: PublicCallableLinkSymbol,
    pub link_symbol_hash: u64,
    pub materialized_symbol: String,
    pub target_source_key_hash: u64,
    pub selected_callable_body_hash: u64,
    pub generic_instantiation_hash: u64,
    pub dependency_public_surface_hash: u64,
    pub source_capability_policy_hash: u64,
    pub private_effect_policy_hash: u64,
    pub stable_hash: u64,
}

/// direct call 用 `.neplobj` の backend payload。
///
/// key は「どの body fragment か」を判定する metadata であり、backend が実際に呼べる
/// body を提供しない。この artifact は key と payload を同じ値にまとめ、compiler pipeline が
/// key-only availability で未知 symbol を隠さないようにする。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeplObjDirectCallFragmentArtifact {
    key: NeplObjDirectCallKey,
    backend: NeplObjDirectCallBackendFragment,
    stable_hash: u64,
}

impl NeplObjDirectCallFragmentArtifact {
    pub fn new(
        key: NeplObjDirectCallKey,
        backend: NeplObjDirectCallBackendFragment,
    ) -> Self {
        let stable_hash = nepl_obj_direct_call_fragment_hash(&key, &backend);
        Self {
            key,
            backend,
            stable_hash,
        }
    }

    pub fn stable_hash(&self) -> u64 {
        self.stable_hash
    }

    pub fn key(&self) -> &NeplObjDirectCallKey {
        &self.key
    }

    pub fn backend(&self) -> &NeplObjDirectCallBackendFragment {
        &self.backend
    }

    pub fn materialized_symbol(&self) -> &str {
        self.key.materialized_symbol.as_str()
    }

    pub fn matches_direct_call_key(&self, key: &NeplObjDirectCallKey) -> bool {
        self.key.stable_hash == key.stable_hash
            && self.key.materialized_symbol == key.materialized_symbol
    }
}

/// same-session `.neplobj` direct-call fragment store の観測値。
///
/// この store は永続 object codec ではなく、Web playground や長寿命 compiler session が
/// source fallback / full compile で得た fragment を次回 compile に渡すための staging
/// boundary である。統計は「保存できたか」と「現在の import edge に対して再利用可能だったか」
/// を分け、body-missing fallback が残る理由を文字列診断に依存せず確認できるようにする。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NeplObjDirectCallFragmentStoreStats {
    pub stores: usize,
    pub duplicate_stores: usize,
    pub store_rejects: usize,
    pub lookup_attempts: usize,
    pub lookup_hits: usize,
    pub lookup_misses: usize,
    pub lookup_missing_source_policy: usize,
    pub lookup_context_rejects: usize,
    pub lookup_fragments_returned: usize,
}

/// `.neplobj` fragment を session store に保存できなかった理由。
///
/// 同じ direct-call key に異なる backend payload が入る場合は、cache の入れ替えではなく
/// compiler/object producer の不整合として扱う。key は source identity、body hash、
/// generic instantiation、policy を含むため、同じ key の payload は決定的に一致しなければ
/// ならない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeplObjDirectCallFragmentStoreReject {
    BackendPayloadConflict { key_stable_hash: u64 },
}

/// `.neplobj` direct-call fragment lookup の import edge 単位の入力。
///
/// materialized public surface は関数本体を持たないため、現在 compile が本当に使ってよい
/// object fragment は loader が作った edge envelope と照合する。`source_capability_policy_hash`
/// がない edge では raw-memory / private-effect policy を比較できないため、store lookup は
/// fail-closed に通常 source fallback を残す。
pub struct NeplObjDirectCallFragmentLookupProbe<'a> {
    pub module_path: &'a str,
    pub target_source_key_hash: u64,
    pub source_capability_policy_hash: Option<u64>,
    pub dependency_public_surface_hash: u64,
    pub surface: &'a TypedPublicSurfaceTable,
}

/// direct-call `.neplobj` fragment の in-memory store。
///
/// store key は `NeplObjDirectCallKey::stable_hash` であり、materialized symbol は lookup を
/// 絞るための secondary index に過ぎない。symbol だけで availability を判断すると、同名
/// signature の body-only edit や policy 変更を stale object で隠してしまうため、lookup では
/// target/profile/source/policy/generic 境界をすべて再確認する。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NeplObjDirectCallFragmentStore {
    fragments: BTreeMap<u64, NeplObjDirectCallFragmentArtifact>,
    materialized_index: BTreeMap<String, Vec<u64>>,
    stats: NeplObjDirectCallFragmentStoreStats,
}

impl NeplObjDirectCallFragmentStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> NeplObjDirectCallFragmentStoreStats {
        self.stats
    }

    pub fn len(&self) -> usize {
        self.fragments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }

    pub fn clear(&mut self) {
        self.fragments.clear();
        self.materialized_index.clear();
        self.stats = NeplObjDirectCallFragmentStoreStats::default();
    }

    pub fn store(
        &mut self,
        fragment: NeplObjDirectCallFragmentArtifact,
    ) -> Result<(), NeplObjDirectCallFragmentStoreReject> {
        let key_hash = fragment.key().stable_hash();
        if let Some(existing) = self.fragments.get(&key_hash) {
            if existing.stable_hash() == fragment.stable_hash() {
                self.stats.duplicate_stores += 1;
                return Ok(());
            }
            self.stats.store_rejects += 1;
            return Err(
                NeplObjDirectCallFragmentStoreReject::BackendPayloadConflict {
                    key_stable_hash: key_hash,
                },
            );
        }
        self.stats.stores += 1;
        self.materialized_index
            .entry(String::from(fragment.materialized_symbol()))
            .or_default()
            .push(key_hash);
        self.fragments.insert(key_hash, fragment);
        Ok(())
    }

    /// body-missing negative cache が materialized edge probe を省略してよいかの保守的判定。
    ///
    /// `.neplmeta` materialization を省くと、その edge から使える direct-call fragment も
    /// compile pipeline へ渡せなくなる。ここでは public surface をまだ投影していないため、
    /// callable 単位までは判定せず、同じ source / dependency / policy context の fragment が
    /// 1 つでもある場合は `true` を返して source fallback の再試行を許す。
    pub fn may_have_fragment_for_probe_context(
        &self,
        target: CompileTarget,
        profile: BuildProfile,
        stdlib_content_hash: Option<u64>,
        target_source_key_hash: u64,
        source_capability_policy_hash: Option<u64>,
        dependency_public_surface_hash: u64,
    ) -> bool {
        let Some(source_capability_policy_hash) = source_capability_policy_hash else {
            return false;
        };
        self.fragments.values().any(|fragment| {
            let key = fragment.key();
            key.schema_version == NEPL_OBJ_DIRECT_CALL_SCHEMA_VERSION
                && key.compiler_identity_hash == nepl_meta_compiler_identity_hash()
                && key.target_hash == nepl_meta_target_hash(target)
                && key.profile_hash == nepl_meta_profile_hash(profile)
                && key.stdlib_content_hash == stdlib_content_hash
                && key.backend_feature_set_hash == nepl_obj_direct_call_backend_feature_set_hash()
                && key.target_source_key_hash == target_source_key_hash
                && key.dependency_public_surface_hash == dependency_public_surface_hash
                && key.source_capability_policy_hash == source_capability_policy_hash
                && key.private_effect_policy_hash
                    == crate::compiler::resource_summary_private_effect_policy_hash()
        })
    }

    /// 現在 materialize された import edge に対して安全に使える fragment だけを返す。
    ///
    /// この lookup は generic 具体化済み fragment producer がまだない MVP なので、空の generic
    /// instantiation hash を持つ direct call だけを返す。generic call、function value、
    /// `memo_call` / PrivateCache lowering は別 proof と backend representation が必要なため、
    /// ここでは object availability として扱わない。
    pub fn matching_direct_call_fragments_for_probes(
        &mut self,
        target: CompileTarget,
        profile: BuildProfile,
        stdlib_content_hash: Option<u64>,
        probes: &[NeplObjDirectCallFragmentLookupProbe<'_>],
    ) -> Vec<NeplObjDirectCallFragmentArtifact> {
        self.stats.lookup_attempts += probes.len();
        if self.fragments.is_empty() || probes.is_empty() {
            self.stats.lookup_misses += probes.len();
            return Vec::new();
        }
        let mut out = Vec::new();
        for probe in probes {
            let Some(source_capability_policy_hash) = probe.source_capability_policy_hash else {
                self.stats.lookup_missing_source_policy += 1;
                continue;
            };
            let symbols = neplobj_direct_call_link_symbols_from_surface(probe.surface);
            if symbols.is_empty() {
                self.stats.lookup_misses += 1;
                continue;
            }
            let before = out.len();
            for symbol in symbols {
                let materialized_symbol = materialized_callable_symbol_for_link_symbol(&symbol);
                let Some(candidate_key_hashes) =
                    self.materialized_index.get(materialized_symbol.as_str())
                else {
                    continue;
                };
                for key_hash in candidate_key_hashes {
                    let Some(fragment) = self.fragments.get(key_hash) else {
                        self.stats.lookup_context_rejects += 1;
                        continue;
                    };
                    if neplobj_direct_call_fragment_matches_lookup(
                        fragment,
                        target,
                        profile,
                        stdlib_content_hash,
                        &symbol,
                        probe.target_source_key_hash,
                        probe.dependency_public_surface_hash,
                        source_capability_policy_hash,
                    ) {
                        out.push(fragment.clone());
                    } else {
                        self.stats.lookup_context_rejects += 1;
                    }
                }
            }
            if out.len() == before {
                self.stats.lookup_misses += 1;
            } else {
                self.stats.lookup_hits += 1;
                self.stats.lookup_fragments_returned += out.len() - before;
            }
        }
        out.sort_by_key(|fragment| fragment.stable_hash());
        out.dedup_by_key(|fragment| fragment.stable_hash());
        out
    }
}

fn neplobj_direct_call_link_symbols_from_surface(
    surface: &TypedPublicSurfaceTable,
) -> Vec<PublicCallableLinkSymbol> {
    let mut symbols = Vec::new();
    for entry in &surface.entries {
        let PublicSurfaceShape::Callable(callable) = &entry.surface else {
            continue;
        };
        if callable.type_param_bounds.is_empty()
            && neplobj_direct_call_callable_surface_is_monomorphic(callable)
        {
            if let Some(symbol) = &callable.link_symbol {
                symbols.push(symbol.clone());
            }
        }
    }
    symbols.sort();
    symbols.dedup();
    symbols
}

fn neplobj_direct_call_callable_surface_is_monomorphic(
    callable: &crate::typecheck::PublicCallableSurface,
) -> bool {
    match &callable.ty {
        crate::typecheck::PublicTypeTerm::Function { type_params, .. } => type_params.is_empty(),
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn neplobj_direct_call_fragment_matches_lookup(
    fragment: &NeplObjDirectCallFragmentArtifact,
    target: CompileTarget,
    profile: BuildProfile,
    stdlib_content_hash: Option<u64>,
    link_symbol: &PublicCallableLinkSymbol,
    target_source_key_hash: u64,
    dependency_public_surface_hash: u64,
    source_capability_policy_hash: u64,
) -> bool {
    let key = fragment.key();
    key.schema_version == NEPL_OBJ_DIRECT_CALL_SCHEMA_VERSION
        && key.compiler_identity_hash == nepl_meta_compiler_identity_hash()
        && key.target_hash == nepl_meta_target_hash(target)
        && key.profile_hash == nepl_meta_profile_hash(profile)
        && key.stdlib_content_hash == stdlib_content_hash
        && key.backend_feature_set_hash == nepl_obj_direct_call_backend_feature_set_hash()
        && key.link_symbol == *link_symbol
        && key.materialized_symbol == materialized_callable_symbol_for_link_symbol(link_symbol)
        && key.target_source_key_hash == target_source_key_hash
        && key.generic_instantiation_hash == nepl_obj_empty_generic_instantiation_hash()
        && key.dependency_public_surface_hash == dependency_public_surface_hash
        && key.source_capability_policy_hash == source_capability_policy_hash
        && key.private_effect_policy_hash
            == crate::compiler::resource_summary_private_effect_policy_hash()
}

/// `.neplobj` direct-call payload の backend 別表現。
///
/// Wasm backend は最終 module の function index / table / data layout に依存するため、
/// 完成済み code section bytes だけでは安全な再利用単位にならない。Wasm payload は
/// body bytes と、後で linker が解決する direct-call relocation を分けて保持する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeplObjDirectCallBackendFragment {
    Wasm(NeplObjWasmDirectCallFragment),
}

/// Wasm direct-call fragment の relocatable payload。
///
/// `function_body_bytes` は wasm-encoder の `Function` 相当の body bytes を保持するための
/// field である。direct call 先の function index は final module assembly で決まるため、
/// `direct_call_relocations` に symbol と byte offset を残し、payload 作成時点では固定しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeplObjWasmDirectCallFragment {
    params: Vec<NeplObjWasmValType>,
    results: Vec<NeplObjWasmValType>,
    function_body_bytes: Vec<u8>,
    direct_call_relocations: Vec<NeplObjWasmDirectCallRelocation>,
}

impl NeplObjWasmDirectCallFragment {
    pub fn new(
        params: Vec<NeplObjWasmValType>,
        results: Vec<NeplObjWasmValType>,
        function_body_bytes: Vec<u8>,
        mut direct_call_relocations: Vec<NeplObjWasmDirectCallRelocation>,
    ) -> Result<Self, NeplObjDirectCallFragmentReject> {
        let body_len = function_body_bytes.len();
        direct_call_relocations.sort();
        let mut previous_offset = None;
        for relocation in &direct_call_relocations {
            if relocation.byte_offset as usize >= body_len {
                return Err(NeplObjDirectCallFragmentReject::RelocationOffsetOutOfRange {
                    byte_offset: relocation.byte_offset,
                    body_len: usize_to_u32_saturating(body_len),
                });
            }
            if previous_offset == Some(relocation.byte_offset) {
                return Err(NeplObjDirectCallFragmentReject::DuplicateRelocationOffset {
                    byte_offset: relocation.byte_offset,
                });
            }
            previous_offset = Some(relocation.byte_offset);
        }
        Ok(Self {
            params,
            results,
            function_body_bytes,
            direct_call_relocations,
        })
    }

    pub fn params(&self) -> &[NeplObjWasmValType] {
        &self.params
    }

    pub fn results(&self) -> &[NeplObjWasmValType] {
        &self.results
    }

    pub fn function_body_bytes(&self) -> &[u8] {
        &self.function_body_bytes
    }

    pub fn direct_call_relocations(&self) -> &[NeplObjWasmDirectCallRelocation] {
        &self.direct_call_relocations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NeplObjWasmValType {
    I32,
    I64,
    F32,
    F64,
}

/// Wasm body 内の direct-call index relocation。
///
/// `byte_offset` は encoded body 中で call index immediate が始まる位置を指す。payload
/// consumer はこの offset を盲目的に信用せず、backend feature set と body encoding version が
/// 一致する場合だけ再配置する。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NeplObjWasmDirectCallRelocation {
    pub byte_offset: u32,
    pub target: PublicCallableLinkSymbol,
}

/// `.neplobj` direct-call fragment を artifact boundary で拒否する理由。
///
/// backend/linker に渡す前に明らかに不正な relocation を落とすことで、cache hit が
/// diagnostic 抑制や unsafe な再配置に進まないようにする。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeplObjDirectCallFragmentReject {
    DuplicateRelocationOffset { byte_offset: u32 },
    RelocationOffsetOutOfRange { byte_offset: u32, body_len: u32 },
}

impl NeplObjDirectCallKey {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        target: CompileTarget,
        profile: BuildProfile,
        stdlib_content_hash: Option<u64>,
        link_symbol: PublicCallableLinkSymbol,
        target_source_key_hash: u64,
        selected_callable_body_hash: u64,
        generic_instantiation_hash: u64,
        dependency_public_surface_hash: u64,
        source_capability_policy_hash: u64,
    ) -> Self {
        let compiler_identity_hash = nepl_meta_compiler_identity_hash();
        let target_hash = nepl_meta_target_hash(target);
        let profile_hash = nepl_meta_profile_hash(profile);
        let backend_feature_set_hash = nepl_obj_direct_call_backend_feature_set_hash();
        let link_symbol_hash = public_callable_link_symbol_stable_hash(&link_symbol);
        let materialized_symbol = materialized_callable_symbol_for_link_symbol(&link_symbol);
        let private_effect_policy_hash =
            crate::compiler::resource_summary_private_effect_policy_hash();
        let stable_hash = nepl_obj_direct_call_key_hash(
            compiler_identity_hash,
            target_hash,
            profile_hash,
            stdlib_content_hash,
            backend_feature_set_hash,
            link_symbol_hash,
            target_source_key_hash,
            selected_callable_body_hash,
            generic_instantiation_hash,
            dependency_public_surface_hash,
            source_capability_policy_hash,
            private_effect_policy_hash,
        );
        Self {
            schema_version: NEPL_OBJ_DIRECT_CALL_SCHEMA_VERSION,
            compiler_identity_hash,
            target_hash,
            profile_hash,
            stdlib_content_hash,
            backend_feature_set_hash,
            link_symbol,
            link_symbol_hash,
            materialized_symbol,
            target_source_key_hash,
            selected_callable_body_hash,
            generic_instantiation_hash,
            dependency_public_surface_hash,
            source_capability_policy_hash,
            private_effect_policy_hash,
            stable_hash,
        }
    }

    pub fn stable_hash(&self) -> u64 {
        self.stable_hash
    }
}

/// direct call `.neplobj` MVP が仮定する backend feature set。
///
/// この値は「現在の wasm/LLVM backend 全体が同じ」という粗い判定ではなく、materialized
/// direct call の body fragment を再利用する時に必要な backend 契約だけを hash 化する。
/// 将来、table layout、data relocation、memoized function value、PrivateCache lowering を
/// 追加する場合は、この feature set を更新して古い `.neplobj` を fail-closed に捨てる。
pub fn nepl_obj_direct_call_backend_feature_set_hash() -> u64 {
    nepl_obj_hash_tag("backend-feature-set", "direct-call-fragment-mvp-v1")
}

/// generic 引数を持たない direct call 用の generic instantiation hash。
///
/// `.neplobj` key は generic の有無にかかわらず同じ field を持つ。非 generic callable では
/// この値を使うことで、「generic 情報を見落とした」のではなく「空の具体化である」と
/// 明示できる。
pub fn nepl_obj_empty_generic_instantiation_hash() -> u64 {
    nepl_obj_hash_tag("generic-instantiation", "empty")
}

/// `.neplmeta` pre-typecheck envelope を作れない理由。
///
/// source key を作れない状態は「再利用できる」ではなく、body skip の前提を証明できない
/// 状態である。caller は通常 load / typecheck fallback に戻す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeplMetaArtifactPreTypecheckEnvelopeReject {
    MissingSourceKey,
}

impl NeplMetaArtifactHeader {
    pub fn new(
        compiler_identity_hash: u64,
        target_hash: u64,
        profile_hash: u64,
        stdlib_content_hash: Option<u64>,
        source_key_hash: Option<u64>,
        dependency_public_surface_hash: Option<u64>,
        typed_public_signature_hash: u64,
        public_entry_count: u32,
        module_surface_hash: Option<u64>,
        module_dependency_edge_count: Option<u32>,
        export_surface_hash: Option<u64>,
        local_export_count: Option<u32>,
        reexport_projection_count: Option<u32>,
        structured_public_surface_hash: u64,
        structured_public_surface_entry_count: u32,
        source_capability_policy_set_hash: Option<u64>,
        private_effect_policy_hash: Option<u64>,
    ) -> Self {
        Self {
            schema_version: NEPL_META_ARTIFACT_SCHEMA_VERSION,
            compiler_identity_hash,
            target_hash,
            profile_hash,
            stdlib_content_hash,
            source_key_hash,
            dependency_public_surface_hash,
            typed_public_signature_hash,
            public_entry_count,
            module_surface_hash,
            module_dependency_edge_count,
            export_surface_hash,
            local_export_count,
            reexport_projection_count,
            structured_public_surface_hash,
            structured_public_surface_entry_count,
            source_capability_policy_set_hash,
            private_effect_policy_hash,
        }
    }

    pub fn compatibility_reject(
        &self,
        expected: NeplMetaArtifactHeader,
    ) -> Option<NeplMetaArtifactCompatibilityReject> {
        if self.schema_version != expected.schema_version {
            return Some(NeplMetaArtifactCompatibilityReject::SchemaVersion);
        }
        if self.compiler_identity_hash != expected.compiler_identity_hash {
            return Some(NeplMetaArtifactCompatibilityReject::CompilerIdentity);
        }
        if self.target_hash != expected.target_hash {
            return Some(NeplMetaArtifactCompatibilityReject::Target);
        }
        if self.profile_hash != expected.profile_hash {
            return Some(NeplMetaArtifactCompatibilityReject::Profile);
        }
        if self.stdlib_content_hash != expected.stdlib_content_hash {
            return Some(NeplMetaArtifactCompatibilityReject::StdlibContent);
        }
        if self.source_key_hash != expected.source_key_hash {
            return Some(NeplMetaArtifactCompatibilityReject::SourceKey);
        }
        if self.dependency_public_surface_hash != expected.dependency_public_surface_hash {
            return Some(NeplMetaArtifactCompatibilityReject::DependencyPublicSurface);
        }
        if self.typed_public_signature_hash != expected.typed_public_signature_hash {
            return Some(NeplMetaArtifactCompatibilityReject::TypedPublicSignature);
        }
        if self.public_entry_count != expected.public_entry_count {
            return Some(NeplMetaArtifactCompatibilityReject::PublicEntryCount);
        }
        if self.module_surface_hash != expected.module_surface_hash {
            return Some(NeplMetaArtifactCompatibilityReject::ModuleSurface);
        }
        if self.module_dependency_edge_count != expected.module_dependency_edge_count {
            return Some(NeplMetaArtifactCompatibilityReject::ModuleDependencyEdgeCount);
        }
        if self.export_surface_hash != expected.export_surface_hash {
            return Some(NeplMetaArtifactCompatibilityReject::ExportSurface);
        }
        if self.local_export_count != expected.local_export_count {
            return Some(NeplMetaArtifactCompatibilityReject::LocalExportCount);
        }
        if self.reexport_projection_count != expected.reexport_projection_count {
            return Some(NeplMetaArtifactCompatibilityReject::ReexportProjectionCount);
        }
        if self.structured_public_surface_hash != expected.structured_public_surface_hash {
            return Some(NeplMetaArtifactCompatibilityReject::StructuredPublicSurface);
        }
        if self.structured_public_surface_entry_count
            != expected.structured_public_surface_entry_count
        {
            return Some(NeplMetaArtifactCompatibilityReject::StructuredPublicSurfaceEntryCount);
        }
        if self.source_capability_policy_set_hash != expected.source_capability_policy_set_hash {
            return Some(NeplMetaArtifactCompatibilityReject::SourceCapabilityPolicySet);
        }
        if self.private_effect_policy_hash != expected.private_effect_policy_hash {
            return Some(NeplMetaArtifactCompatibilityReject::PrivateEffectPolicy);
        }
        None
    }

    pub fn pre_typecheck_compatibility_reject(
        &self,
        expected: NeplMetaArtifactPreTypecheckEnvelope,
    ) -> Option<NeplMetaArtifactCompatibilityReject> {
        if self.schema_version != expected.schema_version {
            return Some(NeplMetaArtifactCompatibilityReject::SchemaVersion);
        }
        if self.compiler_identity_hash != expected.compiler_identity_hash {
            return Some(NeplMetaArtifactCompatibilityReject::CompilerIdentity);
        }
        if self.target_hash != expected.target_hash {
            return Some(NeplMetaArtifactCompatibilityReject::Target);
        }
        if self.profile_hash != expected.profile_hash {
            return Some(NeplMetaArtifactCompatibilityReject::Profile);
        }
        if self.stdlib_content_hash != expected.stdlib_content_hash {
            return Some(NeplMetaArtifactCompatibilityReject::StdlibContent);
        }
        if self.source_key_hash != Some(expected.source_key_hash) {
            return Some(NeplMetaArtifactCompatibilityReject::SourceKey);
        }
        if self.dependency_public_surface_hash != expected.dependency_public_surface_hash {
            return Some(NeplMetaArtifactCompatibilityReject::DependencyPublicSurface);
        }
        if self.module_surface_hash != expected.module_surface_hash {
            return Some(NeplMetaArtifactCompatibilityReject::ModuleSurface);
        }
        if self.module_dependency_edge_count != expected.module_dependency_edge_count {
            return Some(NeplMetaArtifactCompatibilityReject::ModuleDependencyEdgeCount);
        }
        if self.source_capability_policy_set_hash != expected.source_capability_policy_set_hash {
            return Some(NeplMetaArtifactCompatibilityReject::SourceCapabilityPolicySet);
        }
        if self.private_effect_policy_hash != expected.private_effect_policy_hash {
            return Some(NeplMetaArtifactCompatibilityReject::PrivateEffectPolicy);
        }
        None
    }
}

/// `.neplmeta` artifact header が現在 compile と一致しない理由。
///
/// reject reason を enum として分けておくことで、将来 disk / IndexedDB codec を追加しても
/// mismatch を文字列比較で処理せず、静的に網羅できる診断・統計へ接続できる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeplMetaArtifactCompatibilityReject {
    SchemaVersion,
    CompilerIdentity,
    Target,
    Profile,
    StdlibContent,
    SourceKey,
    DependencyPublicSurface,
    TypedPublicSignature,
    PublicEntryCount,
    ModuleSurface,
    ModuleDependencyEdgeCount,
    ExportSurface,
    LocalExportCount,
    ReexportProjectionCount,
    StructuredPublicSurface,
    StructuredPublicSurfaceEntryCount,
    SourceCapabilityPolicySet,
    PrivateEffectPolicy,
}

impl NeplMetaArtifactCompatibilityReject {
    pub fn code(self) -> u32 {
        match self {
            Self::SchemaVersion => 1,
            Self::CompilerIdentity => 2,
            Self::Target => 3,
            Self::Profile => 4,
            Self::StdlibContent => 5,
            Self::SourceKey => 6,
            Self::DependencyPublicSurface => 7,
            Self::TypedPublicSignature => 8,
            Self::PublicEntryCount => 9,
            Self::ModuleSurface => 10,
            Self::ModuleDependencyEdgeCount => 11,
            Self::ExportSurface => 12,
            Self::LocalExportCount => 13,
            Self::ReexportProjectionCount => 14,
            Self::StructuredPublicSurface => 15,
            Self::StructuredPublicSurfaceEntryCount => 16,
            Self::SourceCapabilityPolicySet => 17,
            Self::PrivateEffectPolicy => 18,
        }
    }
}

/// `.neplmeta` artifact の in-memory 表現。
///
/// payload は typed public signature table に限定する。依存 module の body、typed HIR、
/// Resource IR、diagnostic span はここに含めない。依存側の typecheck が必要とする
/// public callable / type / trait / impl surface を先に artifact 化し、body や codegen fragment
/// の cache は `.neplhir` / `.neplproof` / `.neplobj` へ分ける。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeplMetaArtifact {
    header: NeplMetaArtifactHeader,
    public_signatures: TypedPublicSignatureTable,
    module_surface: Option<NeplMetaModuleSurface>,
    export_surface: Option<NeplMetaExportSurface>,
    public_surface: TypedPublicSurfaceTable,
}

impl NeplMetaArtifact {
    pub fn new(
        header: NeplMetaArtifactHeader,
        public_signatures: TypedPublicSignatureTable,
        module_surface: Option<NeplMetaModuleSurface>,
        export_surface: Option<NeplMetaExportSurface>,
        public_surface: TypedPublicSurfaceTable,
    ) -> Self {
        Self {
            header,
            public_signatures,
            module_surface,
            export_surface,
            public_surface,
        }
    }

    pub fn from_public_surface(
        target: CompileTarget,
        profile: BuildProfile,
        stdlib_content_hash: Option<u64>,
        dependency_public_surface_hash: Option<u64>,
        source_map: Option<&SourceMap>,
        public_signatures: TypedPublicSignatureTable,
        public_surface: TypedPublicSurfaceTable,
    ) -> Self {
        Self::from_public_surface_and_module_surface(
            target,
            profile,
            stdlib_content_hash,
            dependency_public_surface_hash,
            source_map,
            public_signatures,
            None,
            public_surface,
        )
    }

    pub fn from_public_surface_and_module_surface(
        target: CompileTarget,
        profile: BuildProfile,
        stdlib_content_hash: Option<u64>,
        dependency_public_surface_hash: Option<u64>,
        source_map: Option<&SourceMap>,
        public_signatures: TypedPublicSignatureTable,
        module_surface: Option<NeplMetaModuleSurface>,
        public_surface: TypedPublicSurfaceTable,
    ) -> Self {
        let export_surface = module_surface.as_ref().map(|surface| {
            NeplMetaExportSurface::from_module_and_public_surface(surface, &public_surface)
        });
        let header = nepl_meta_artifact_header_for_public_surface(
            target,
            profile,
            stdlib_content_hash,
            dependency_public_surface_hash,
            source_map,
            &public_signatures,
            module_surface.as_ref(),
            export_surface.as_ref(),
            &public_surface,
        );
        Self::new(
            header,
            public_signatures,
            module_surface,
            export_surface,
            public_surface,
        )
    }

    /// loader が module 単位で検証した source identity から `.neplmeta` を作る。
    ///
    /// dependency artifact producer は、target module を typecheck するために依存先も
    /// `SourceMap` へ読み込む。しかし pre-typecheck edge probe の互換性境界は、呼び出し元の
    /// root や同時に読み込まれた依存 module ではなく、target module 自身の source key と
    /// capability policy に閉じなければならない。そのため、この constructor は `SourceMap`
    /// 全体から source identity を再計算せず、loader が target source から作った安定値を
    /// header へ固定する。
    pub fn from_public_surface_and_module_surface_with_source_identity(
        target: CompileTarget,
        profile: BuildProfile,
        stdlib_content_hash: Option<u64>,
        dependency_public_surface_hash: Option<u64>,
        source_key_hash: u64,
        source_capability_policy_set_hash: Option<u64>,
        public_signatures: TypedPublicSignatureTable,
        module_surface: Option<NeplMetaModuleSurface>,
        public_surface: TypedPublicSurfaceTable,
    ) -> Self {
        let export_surface = module_surface.as_ref().map(|surface| {
            NeplMetaExportSurface::from_module_and_public_surface(surface, &public_surface)
        });
        let header = NeplMetaArtifactHeader::new(
            nepl_meta_compiler_identity_hash(),
            nepl_meta_target_hash(target),
            nepl_meta_profile_hash(profile),
            stdlib_content_hash,
            Some(source_key_hash),
            dependency_public_surface_hash,
            public_signatures.stable_hash,
            usize_to_u32_saturating(public_signatures.entries.len()),
            module_surface.as_ref().map(|surface| surface.stable_hash),
            module_surface
                .as_ref()
                .map(|surface| usize_to_u32_saturating(surface.dependency_edges.len())),
            export_surface.as_ref().map(|surface| surface.stable_hash),
            export_surface
                .as_ref()
                .map(|surface| usize_to_u32_saturating(surface.local_exports.len())),
            export_surface
                .as_ref()
                .map(|surface| usize_to_u32_saturating(surface.reexport_projections.len())),
            public_surface.stable_hash,
            usize_to_u32_saturating(public_surface.entries.len()),
            source_capability_policy_set_hash,
            Some(crate::compiler::resource_summary_private_effect_policy_hash()),
        );
        Self::new(
            header,
            public_signatures,
            module_surface,
            export_surface,
            public_surface,
        )
    }

    pub fn header(&self) -> NeplMetaArtifactHeader {
        self.header
    }

    pub fn public_signatures(&self) -> &TypedPublicSignatureTable {
        &self.public_signatures
    }

    pub fn module_surface(&self) -> Option<&NeplMetaModuleSurface> {
        self.module_surface.as_ref()
    }

    pub fn export_surface(&self) -> Option<&NeplMetaExportSurface> {
        self.export_surface.as_ref()
    }

    pub fn public_surface(&self) -> &TypedPublicSurfaceTable {
        &self.public_surface
    }

    /// local export を `.neplmeta` typecheck materializer へ渡す public surface に絞る。
    ///
    /// この関数は source body skip を実行しない。export surface が示す公開名と structured
    /// public surface の entry を照合し、import clause から見える local export を
    /// `TypedPublicSurfaceTable` として返す。projection は artifact の安定表現を
    /// typed public surface へ戻す責務だけを持ち、`TypeCtx` / `Env` への登録可否は
    /// 後段の materializer が改めて fail-closed に判定する。
    pub fn materializer_local_export_public_surface_mvp(
        &self,
    ) -> Result<TypedPublicSurfaceTable, NeplMetaMaterializerProjectionReject> {
        self.project_materializer_public_surface_mvp(None)
    }

    /// import clause で見える target artifact の public surface を materializer 入力へ絞る。
    ///
    /// `Open` / clause なしは local export 全体、alias なし selective import は指定名だけを
    /// kind を保ったまま受け入れる。alias、glob、merge、default alias、
    /// re-export projection は target artifact 以外の authority や衝突判定が必要なので、
    /// ここでは推測せず通常 source load へ戻す。
    pub fn materializer_import_public_surface_mvp(
        &self,
        import_clause: Option<&NeplMetaImportClause>,
    ) -> Result<TypedPublicSurfaceTable, NeplMetaMaterializerProjectionReject> {
        self.project_materializer_public_surface_mvp(import_clause)
    }

    /// `.neplmeta` を stdlib public surface materializer MVP へ渡せるかを判定する。
    ///
    /// この判定は body skip そのものではなく、artifact payload が current compile の
    /// `TypeCtx` / `Env` へ安全に投影できる最小条件を満たすかを確認する gate である。
    /// 未対応の import clause や impl lookup は推測で補わず、通常の source load /
    /// typecheck へ戻すために enum reason として返す。
    pub fn materializer_mvp_reject(&self) -> Option<NeplMetaMaterializerMvpReject> {
        if let Some(reject) = self.payload_consistency_reject() {
            return Some(NeplMetaMaterializerMvpReject::PayloadConsistency(reject));
        }
        if self.header.source_key_hash.is_none() {
            return Some(NeplMetaMaterializerMvpReject::MissingSourceKey);
        }
        let module_surface = match self.module_surface.as_ref() {
            Some(surface) => surface,
            None => return Some(NeplMetaMaterializerMvpReject::MissingModuleSurface),
        };
        if module_surface.canonical_module_path.is_empty() {
            return Some(NeplMetaMaterializerMvpReject::MissingModuleIdentity);
        }
        let export_surface = match self.export_surface.as_ref() {
            Some(surface) => surface,
            None => return Some(NeplMetaMaterializerMvpReject::MissingExportSurface),
        };
        if let Some(blocker) = self.public_surface.materializer_blockers().into_iter().next() {
            return Some(NeplMetaMaterializerMvpReject::PublicSurfaceBlocker(blocker));
        }
        for edge in &module_surface.dependency_edges {
            if let Some(reject) =
                materializer_mvp_reject_for_edge(edge.kind, edge.import_clause.as_ref())
            {
                return Some(reject);
            }
        }
        for projection in &export_surface.reexport_projections {
            if projection.target_module_path.is_empty() {
                return Some(NeplMetaMaterializerMvpReject::MissingReexportTarget);
            }
            if let Some(reject) =
                materializer_mvp_reject_for_edge(projection.kind, projection.import_clause.as_ref())
            {
                return Some(reject);
            }
        }
        None
    }

    pub fn compatibility_reject(
        &self,
        expected_header: NeplMetaArtifactHeader,
    ) -> Option<NeplMetaArtifactCompatibilityReject> {
        self.header.compatibility_reject(expected_header)
    }

    pub fn payload_consistency_reject(&self) -> Option<NeplMetaArtifactPayloadReject> {
        if self.header.typed_public_signature_hash != self.public_signatures.stable_hash {
            return Some(NeplMetaArtifactPayloadReject::TypedPublicSignatureHash);
        }
        if self.header.public_entry_count
            != usize_to_u32_saturating(self.public_signatures.entries.len())
        {
            return Some(NeplMetaArtifactPayloadReject::PublicEntryCount);
        }
        let actual_module_surface_hash = self
            .module_surface
            .as_ref()
            .map(|surface| surface.stable_hash);
        if self.header.module_surface_hash != actual_module_surface_hash {
            return Some(NeplMetaArtifactPayloadReject::ModuleSurfaceHash);
        }
        let actual_module_edge_count = self
            .module_surface
            .as_ref()
            .map(|surface| usize_to_u32_saturating(surface.dependency_edges.len()));
        if self.header.module_dependency_edge_count != actual_module_edge_count {
            return Some(NeplMetaArtifactPayloadReject::ModuleDependencyEdgeCount);
        }
        let actual_export_surface_hash =
            self.export_surface.as_ref().map(|surface| surface.stable_hash);
        if self.header.export_surface_hash != actual_export_surface_hash {
            return Some(NeplMetaArtifactPayloadReject::ExportSurfaceHash);
        }
        let actual_local_export_count = self
            .export_surface
            .as_ref()
            .map(|surface| usize_to_u32_saturating(surface.local_exports.len()));
        if self.header.local_export_count != actual_local_export_count {
            return Some(NeplMetaArtifactPayloadReject::LocalExportCount);
        }
        let actual_reexport_projection_count = self
            .export_surface
            .as_ref()
            .map(|surface| usize_to_u32_saturating(surface.reexport_projections.len()));
        if self.header.reexport_projection_count != actual_reexport_projection_count {
            return Some(NeplMetaArtifactPayloadReject::ReexportProjectionCount);
        }
        if self.header.structured_public_surface_hash != self.public_surface.stable_hash {
            return Some(NeplMetaArtifactPayloadReject::StructuredPublicSurfaceHash);
        }
        if self.header.structured_public_surface_entry_count
            != usize_to_u32_saturating(self.public_surface.entries.len())
        {
            return Some(NeplMetaArtifactPayloadReject::StructuredPublicSurfaceEntryCount);
        }
        None
    }

    pub fn entry_count(&self) -> usize {
        self.public_signatures.entries.len()
    }

    fn project_materializer_public_surface_mvp(
        &self,
        import_clause: Option<&NeplMetaImportClause>,
    ) -> Result<TypedPublicSurfaceTable, NeplMetaMaterializerProjectionReject> {
        if let Some(reject) = self.payload_consistency_reject() {
            return Err(NeplMetaMaterializerProjectionReject::PayloadConsistency(
                reject,
            ));
        }
        if self.header.source_key_hash.is_none() {
            return Err(NeplMetaMaterializerProjectionReject::MissingSourceKey);
        }
        let module_surface = self
            .module_surface
            .as_ref()
            .ok_or(NeplMetaMaterializerProjectionReject::MissingModuleSurface)?;
        if module_surface.canonical_module_path.is_empty() {
            return Err(NeplMetaMaterializerProjectionReject::MissingModuleIdentity);
        }
        let export_surface = self
            .export_surface
            .as_ref()
            .ok_or(NeplMetaMaterializerProjectionReject::MissingExportSurface)?;
        if !export_surface.reexport_projections.is_empty() {
            return Err(NeplMetaMaterializerProjectionReject::UnsupportedReexportProjection);
        }
        if let Some(blocker) = self.public_surface.materializer_blockers().into_iter().next() {
            return Err(NeplMetaMaterializerProjectionReject::PublicSurfaceBlocker(
                blocker,
            ));
        }
        let requested_names = requested_materializer_export_names(import_clause)?;
        let export_entries = match requested_names {
            MaterializerExportNames::All => export_surface.local_exports.clone(),
            MaterializerExportNames::Named(names) => {
                let mut out = Vec::new();
                for requested in names {
                    let Some(entry) = export_surface
                        .local_exports
                        .iter()
                        .find(|entry| entry.exported_name == requested)
                    else {
                        return Err(NeplMetaMaterializerProjectionReject::ExportedNameMissing {
                            name: requested,
                        });
                    };
                    out.push(entry.clone());
                }
                out
            }
        };
        let mut projected = Vec::new();
        for export in export_entries {
            if export.origin_module_path != module_surface.canonical_module_path {
                return Err(
                    NeplMetaMaterializerProjectionReject::ExportOriginModuleMismatch {
                        exported_name: export.exported_name,
                        origin_module_path: export.origin_module_path,
                    },
                );
            }
            let expected_entry_kind = export.kind.typed_public_signature_kind();
            let Some(entry) = self
                .public_surface
                .entries
                .iter()
                .find(|entry| {
                    entry.exported
                        && entry.kind == expected_entry_kind
                        && entry.name == export.origin_name
                })
            else {
                return Err(NeplMetaMaterializerProjectionReject::ExportedSurfaceMissing {
                    exported_name: export.exported_name,
                    origin_name: export.origin_name,
                });
            };
            if export.exported_name != entry.name {
                return Err(NeplMetaMaterializerProjectionReject::ExportAliasUnsupported {
                    exported_name: export.exported_name,
                    origin_name: entry.name.clone(),
                });
            }
            projected.push(entry.clone());
        }
        for entry in self.public_surface.entries.iter().filter(|entry| !entry.exported) {
            projected.push(entry.clone());
        }
        Ok(TypedPublicSurfaceTable::new(projected))
    }
}

/// `.neplmeta` artifact store の累積統計。
///
/// store は compile session 内で artifact を再利用できるかどうかを判定する staging
/// authority であり、typecheck body skip そのものではない。統計は「hit したが
/// compatibility / payload / projection で拒否された」場合を区別し、性能調査で
/// cache が効いていない理由を文字列解析なしに確認できるようにする。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NeplMetaArtifactStoreStats {
    pub stores: usize,
    pub store_rejects: usize,
    pub hits: usize,
    pub misses: usize,
    pub payload_rejects: usize,
    pub compatibility_rejects: usize,
    pub projection_rejects: usize,
    pub pre_typecheck_probe_attempts: usize,
    pub pre_typecheck_probe_projected: usize,
    pub pre_typecheck_probe_missing_artifacts: usize,
    pub pre_typecheck_probe_payload_rejects: usize,
    pub pre_typecheck_probe_compatibility_rejects: usize,
    pub pre_typecheck_probe_projection_rejects: usize,
    pub pre_typecheck_probe_projected_entries: usize,
    pub last_pre_typecheck_probe_reject_kind: NeplMetaArtifactProbeRejectKind,
    pub last_pre_typecheck_probe_reject_code: u32,
    pub last_pre_typecheck_probe_projection_blocker_reason_code: u32,
    pub last_pre_typecheck_probe_projection_blocker_entry_kind_code: u32,
    pub last_pre_typecheck_probe_projected_entries: usize,
    pub pre_typecheck_edge_probe_attempts: usize,
    pub pre_typecheck_edge_probe_projected: usize,
    pub pre_typecheck_edge_probe_missing_artifacts: usize,
    pub pre_typecheck_edge_probe_payload_rejects: usize,
    pub pre_typecheck_edge_probe_compatibility_rejects: usize,
    pub pre_typecheck_edge_probe_projection_rejects: usize,
    pub pre_typecheck_edge_probe_projected_entries: usize,
    pub last_pre_typecheck_edge_probe_reject_kind: NeplMetaArtifactProbeRejectKind,
    pub last_pre_typecheck_edge_probe_reject_code: u32,
    pub last_pre_typecheck_edge_probe_projection_blocker_reason_code: u32,
    pub last_pre_typecheck_edge_probe_projection_blocker_entry_kind_code: u32,
    pub last_pre_typecheck_edge_probe_projected_entries: usize,
}

/// pre-typecheck probe が通常 source fallback へ戻った理由の大分類。
///
/// 個別理由は `last_pre_typecheck_probe_reject_code` に保存する。kind と code を分けると、
/// compatibility mismatch と projection unsupported を同じ数値空間へ押し込まずに済み、
/// Web playground 側の JSON も安定した小さい値だけを公開できる。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NeplMetaArtifactProbeRejectKind {
    #[default]
    None,
    MissingArtifact,
    PayloadConsistency,
    Compatibility,
    Projection,
}

impl NeplMetaArtifactProbeRejectKind {
    pub fn code(self) -> u32 {
        match self {
            Self::None => 0,
            Self::MissingArtifact => 1,
            Self::PayloadConsistency => 2,
            Self::Compatibility => 3,
            Self::Projection => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NeplMetaArtifactPreTypecheckProbeScope {
    Root,
    Edge,
}

impl NeplMetaArtifactStoreStats {
    fn record_pre_typecheck_probe_attempt(&mut self, scope: NeplMetaArtifactPreTypecheckProbeScope) {
        match scope {
            NeplMetaArtifactPreTypecheckProbeScope::Root => {
                self.pre_typecheck_probe_attempts += 1;
            }
            NeplMetaArtifactPreTypecheckProbeScope::Edge => {
                self.pre_typecheck_edge_probe_attempts += 1;
            }
        }
    }

    fn record_pre_typecheck_probe_projected(
        &mut self,
        scope: NeplMetaArtifactPreTypecheckProbeScope,
        entry_count: usize,
    ) {
        match scope {
            NeplMetaArtifactPreTypecheckProbeScope::Root => {
                self.pre_typecheck_probe_projected += 1;
                self.pre_typecheck_probe_projected_entries += entry_count;
                self.last_pre_typecheck_probe_reject_kind = NeplMetaArtifactProbeRejectKind::None;
                self.last_pre_typecheck_probe_reject_code = 0;
                self.last_pre_typecheck_probe_projection_blocker_reason_code = 0;
                self.last_pre_typecheck_probe_projection_blocker_entry_kind_code = 0;
                self.last_pre_typecheck_probe_projected_entries = entry_count;
            }
            NeplMetaArtifactPreTypecheckProbeScope::Edge => {
                self.pre_typecheck_edge_probe_projected += 1;
                self.pre_typecheck_edge_probe_projected_entries += entry_count;
                self.last_pre_typecheck_edge_probe_reject_kind =
                    NeplMetaArtifactProbeRejectKind::None;
                self.last_pre_typecheck_edge_probe_reject_code = 0;
                self.last_pre_typecheck_edge_probe_projection_blocker_reason_code = 0;
                self.last_pre_typecheck_edge_probe_projection_blocker_entry_kind_code = 0;
                self.last_pre_typecheck_edge_probe_projected_entries = entry_count;
            }
        }
    }

    fn record_pre_typecheck_probe_missing_artifact(
        &mut self,
        scope: NeplMetaArtifactPreTypecheckProbeScope,
    ) {
        match scope {
            NeplMetaArtifactPreTypecheckProbeScope::Root => {
                self.pre_typecheck_probe_missing_artifacts += 1;
                self.last_pre_typecheck_probe_reject_kind =
                    NeplMetaArtifactProbeRejectKind::MissingArtifact;
                self.last_pre_typecheck_probe_reject_code = 0;
                self.last_pre_typecheck_probe_projection_blocker_reason_code = 0;
                self.last_pre_typecheck_probe_projection_blocker_entry_kind_code = 0;
                self.last_pre_typecheck_probe_projected_entries = 0;
            }
            NeplMetaArtifactPreTypecheckProbeScope::Edge => {
                self.pre_typecheck_edge_probe_missing_artifacts += 1;
                self.last_pre_typecheck_edge_probe_reject_kind =
                    NeplMetaArtifactProbeRejectKind::MissingArtifact;
                self.last_pre_typecheck_edge_probe_reject_code = 0;
                self.last_pre_typecheck_edge_probe_projection_blocker_reason_code = 0;
                self.last_pre_typecheck_edge_probe_projection_blocker_entry_kind_code = 0;
                self.last_pre_typecheck_edge_probe_projected_entries = 0;
            }
        }
    }

    fn record_pre_typecheck_probe_compatibility_reject(
        &mut self,
        scope: NeplMetaArtifactPreTypecheckProbeScope,
        reject: NeplMetaArtifactCompatibilityReject,
    ) {
        match scope {
            NeplMetaArtifactPreTypecheckProbeScope::Root => {
                self.pre_typecheck_probe_compatibility_rejects += 1;
                self.last_pre_typecheck_probe_reject_kind =
                    NeplMetaArtifactProbeRejectKind::Compatibility;
                self.last_pre_typecheck_probe_reject_code = reject.code();
                self.last_pre_typecheck_probe_projection_blocker_reason_code = 0;
                self.last_pre_typecheck_probe_projection_blocker_entry_kind_code = 0;
                self.last_pre_typecheck_probe_projected_entries = 0;
            }
            NeplMetaArtifactPreTypecheckProbeScope::Edge => {
                self.pre_typecheck_edge_probe_compatibility_rejects += 1;
                self.last_pre_typecheck_edge_probe_reject_kind =
                    NeplMetaArtifactProbeRejectKind::Compatibility;
                self.last_pre_typecheck_edge_probe_reject_code = reject.code();
                self.last_pre_typecheck_edge_probe_projection_blocker_reason_code = 0;
                self.last_pre_typecheck_edge_probe_projection_blocker_entry_kind_code = 0;
                self.last_pre_typecheck_edge_probe_projected_entries = 0;
            }
        }
    }

    fn record_pre_typecheck_probe_payload_reject(
        &mut self,
        scope: NeplMetaArtifactPreTypecheckProbeScope,
        reject: NeplMetaArtifactPayloadReject,
    ) {
        match scope {
            NeplMetaArtifactPreTypecheckProbeScope::Root => {
                self.pre_typecheck_probe_payload_rejects += 1;
                self.last_pre_typecheck_probe_reject_kind =
                    NeplMetaArtifactProbeRejectKind::PayloadConsistency;
                self.last_pre_typecheck_probe_reject_code = reject.code();
                self.last_pre_typecheck_probe_projection_blocker_reason_code = 0;
                self.last_pre_typecheck_probe_projection_blocker_entry_kind_code = 0;
                self.last_pre_typecheck_probe_projected_entries = 0;
            }
            NeplMetaArtifactPreTypecheckProbeScope::Edge => {
                self.pre_typecheck_edge_probe_payload_rejects += 1;
                self.last_pre_typecheck_edge_probe_reject_kind =
                    NeplMetaArtifactProbeRejectKind::PayloadConsistency;
                self.last_pre_typecheck_edge_probe_reject_code = reject.code();
                self.last_pre_typecheck_edge_probe_projection_blocker_reason_code = 0;
                self.last_pre_typecheck_edge_probe_projection_blocker_entry_kind_code = 0;
                self.last_pre_typecheck_edge_probe_projected_entries = 0;
            }
        }
    }

    fn record_pre_typecheck_probe_projection_reject(
        &mut self,
        scope: NeplMetaArtifactPreTypecheckProbeScope,
        reject: &NeplMetaMaterializerProjectionReject,
    ) {
        match scope {
            NeplMetaArtifactPreTypecheckProbeScope::Root => {
                self.pre_typecheck_probe_projection_rejects += 1;
                self.last_pre_typecheck_probe_reject_kind =
                    NeplMetaArtifactProbeRejectKind::Projection;
                self.last_pre_typecheck_probe_reject_code = reject.code();
                self.last_pre_typecheck_probe_projection_blocker_reason_code =
                    reject.public_surface_blocker_reason_code();
                self.last_pre_typecheck_probe_projection_blocker_entry_kind_code =
                    reject.public_surface_blocker_entry_kind_code();
                self.last_pre_typecheck_probe_projected_entries = 0;
            }
            NeplMetaArtifactPreTypecheckProbeScope::Edge => {
                self.pre_typecheck_edge_probe_projection_rejects += 1;
                self.last_pre_typecheck_edge_probe_reject_kind =
                    NeplMetaArtifactProbeRejectKind::Projection;
                self.last_pre_typecheck_edge_probe_reject_code = reject.code();
                self.last_pre_typecheck_edge_probe_projection_blocker_reason_code =
                    reject.public_surface_blocker_reason_code();
                self.last_pre_typecheck_edge_probe_projection_blocker_entry_kind_code =
                    reject.public_surface_blocker_entry_kind_code();
                self.last_pre_typecheck_edge_probe_projected_entries = 0;
            }
        }
    }
}

/// module path keyed な `.neplmeta` artifact store。
///
/// この store は永続 codec ではなく、`CompilerSession` などの長寿命 session が
/// artifact を安全に再利用するための in-memory 境界である。artifact は
/// `NeplMetaModuleSurface::canonical_module_path` で保存し、取り出し時に header
/// compatibility と import projection を再確認する。`TypeId`、`Span`、`SourceMap`、
/// typed HIR、Resource IR は store key や payload authority にしない。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NeplMetaArtifactStore {
    artifacts: BTreeMap<String, NeplMetaArtifact>,
    stats: NeplMetaArtifactStoreStats,
}

impl NeplMetaArtifactStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> NeplMetaArtifactStoreStats {
        self.stats
    }

    pub fn len(&self) -> usize {
        self.artifacts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }

    pub fn clear(&mut self) {
        self.artifacts.clear();
        self.stats = NeplMetaArtifactStoreStats::default();
    }

    /// 指定 module の artifact が現在の pre-typecheck envelope と互換かを統計なしで確認する。
    ///
    /// dependency artifact producer は、store が既に同じ source/profile/stdlib 境界の artifact を
    /// 持っている場合に再 typecheck を避けたい。一方で、ここで通常 probe 統計を増やすと
    /// 「実際に materializer が試した回数」と「producer が重複生成を避けた回数」が混ざる。
    /// そのため、この判定は store 内容の鮮度確認だけを行い、hit/miss/reject 統計は更新しない。
    pub fn has_pre_typecheck_compatible_artifact(
        &self,
        module_path: &str,
        expected_envelope: NeplMetaArtifactPreTypecheckEnvelope,
    ) -> bool {
        let Some(artifact) = self.artifacts.get(module_path) else {
            return false;
        };
        if artifact.payload_consistency_reject().is_some() {
            return false;
        }
        artifact
            .header()
            .pre_typecheck_compatibility_reject(expected_envelope)
            .is_none()
    }

    pub fn store(
        &mut self,
        artifact: NeplMetaArtifact,
    ) -> Result<(), NeplMetaArtifactStoreReject> {
        let module_path = match artifact.module_surface() {
            Some(surface) if !surface.canonical_module_path.is_empty() => {
                surface.canonical_module_path.clone()
            }
            Some(_) => {
                self.stats.store_rejects += 1;
                return Err(NeplMetaArtifactStoreReject::MissingModuleIdentity);
            }
            None => {
                self.stats.store_rejects += 1;
                return Err(NeplMetaArtifactStoreReject::MissingModuleSurface);
            }
        };
        if let Some(reject) = artifact.payload_consistency_reject() {
            self.stats.store_rejects += 1;
            self.stats.payload_rejects += 1;
            return Err(NeplMetaArtifactStoreReject::PayloadConsistency(reject));
        }
        if artifact.header().source_key_hash.is_none() {
            self.stats.store_rejects += 1;
            return Err(NeplMetaArtifactStoreReject::MissingSourceKey);
        }
        self.stats.stores += 1;
        self.artifacts.insert(module_path, artifact);
        Ok(())
    }

    /// compatible artifact から import clause 可視の materializer 入力だけを返す。
    ///
    /// ここで返る `TypedPublicSurfaceTable` は、まだ current session の `TypeCtx` / `Env`
    /// に注入された結果ではない。caller はこの後 `typecheck/materializer` に渡し、
    /// `def_id=None` の callable が function identity 必須経路へ流れないよう、通常
    /// source fallback と組み合わせて扱う。
    pub fn materializer_import_public_surface_mvp(
        &mut self,
        module_path: &str,
        expected_header: NeplMetaArtifactHeader,
        import_clause: Option<&NeplMetaImportClause>,
    ) -> Result<TypedPublicSurfaceTable, NeplMetaArtifactStoreReject> {
        let Some(artifact) = self.artifacts.get(module_path) else {
            self.stats.misses += 1;
            return Err(NeplMetaArtifactStoreReject::MissingArtifact {
                module_path: String::from(module_path),
            });
        };
        self.stats.hits += 1;
        if let Some(reject) = artifact.payload_consistency_reject() {
            self.stats.payload_rejects += 1;
            return Err(NeplMetaArtifactStoreReject::PayloadConsistency(reject));
        }
        if artifact.header().source_key_hash.is_none() {
            self.stats.compatibility_rejects += 1;
            return Err(NeplMetaArtifactStoreReject::MissingSourceKey);
        }
        if let Some(reject) = artifact.compatibility_reject(expected_header) {
            self.stats.compatibility_rejects += 1;
            return Err(NeplMetaArtifactStoreReject::Compatibility(reject));
        }
        match artifact.materializer_import_public_surface_mvp(import_clause) {
            Ok(table) => Ok(table),
            Err(reject) => {
                self.stats.projection_rejects += 1;
                Err(NeplMetaArtifactStoreReject::Projection(reject))
            }
        }
    }

    /// pre-typecheck envelope で照合したうえで materializer 入力を返す。
    ///
    /// dependency body skip の入口では、target module の typed public signature や
    /// structured public surface をまだ作れない。ここでは loader/source-map 由来の
    /// envelope だけを先に照合し、payload が self-consistent で projection も MVP 範囲に
    /// 入る場合だけ `TypedPublicSurfaceTable` を返す。caller はこの結果を fresh
    /// `TypeCtx` / `Env` へ materialize する前に、通常 source fallback を保持する。
    pub fn materializer_import_public_surface_pre_typecheck_mvp(
        &mut self,
        module_path: &str,
        expected_envelope: NeplMetaArtifactPreTypecheckEnvelope,
        import_clause: Option<&NeplMetaImportClause>,
    ) -> Result<TypedPublicSurfaceTable, NeplMetaArtifactStoreReject> {
        self.materializer_import_public_surface_pre_typecheck_mvp_with_scope(
            module_path,
            expected_envelope,
            import_clause,
            NeplMetaArtifactPreTypecheckProbeScope::Root,
        )
    }

    /// import / prelude edge 用の pre-typecheck probe。
    ///
    /// root artifact probe と同じ compatibility / projection 判定を使うが、統計は別 field
    /// へ記録する。edge probe は dependency artifact store をまだ使えない理由を観測する
    /// ための入口であり、失敗しても通常 load / typecheck fallback を変えてはならない。
    pub fn materializer_import_public_surface_pre_typecheck_edge_probe_mvp(
        &mut self,
        module_path: &str,
        expected_envelope: NeplMetaArtifactPreTypecheckEnvelope,
        import_clause: Option<&NeplMetaImportClause>,
    ) -> Result<TypedPublicSurfaceTable, NeplMetaArtifactStoreReject> {
        self.materializer_import_public_surface_pre_typecheck_mvp_with_scope(
            module_path,
            expected_envelope,
            import_clause,
            NeplMetaArtifactPreTypecheckProbeScope::Edge,
        )
    }

    fn materializer_import_public_surface_pre_typecheck_mvp_with_scope(
        &mut self,
        module_path: &str,
        expected_envelope: NeplMetaArtifactPreTypecheckEnvelope,
        import_clause: Option<&NeplMetaImportClause>,
        scope: NeplMetaArtifactPreTypecheckProbeScope,
    ) -> Result<TypedPublicSurfaceTable, NeplMetaArtifactStoreReject> {
        self.stats.record_pre_typecheck_probe_attempt(scope);
        let Some(artifact) = self.artifacts.get(module_path) else {
            self.stats.misses += 1;
            self.stats.record_pre_typecheck_probe_missing_artifact(scope);
            return Err(NeplMetaArtifactStoreReject::MissingArtifact {
                module_path: String::from(module_path),
            });
        };
        self.stats.hits += 1;
        if let Some(reject) = artifact.payload_consistency_reject() {
            self.stats.payload_rejects += 1;
            self.stats
                .record_pre_typecheck_probe_payload_reject(scope, reject);
            return Err(NeplMetaArtifactStoreReject::PayloadConsistency(reject));
        }
        if let Some(reject) = artifact
            .header()
            .pre_typecheck_compatibility_reject(expected_envelope)
        {
            self.stats.compatibility_rejects += 1;
            self.stats
                .record_pre_typecheck_probe_compatibility_reject(scope, reject);
            return Err(NeplMetaArtifactStoreReject::Compatibility(reject));
        }
        match artifact.materializer_import_public_surface_mvp(import_clause) {
            Ok(table) => {
                self.stats
                    .record_pre_typecheck_probe_projected(scope, table.entries.len());
                Ok(table)
            }
            Err(reject) => {
                self.stats.projection_rejects += 1;
                self.stats
                    .record_pre_typecheck_probe_projection_reject(scope, &reject);
                Err(NeplMetaArtifactStoreReject::Projection(reject))
            }
        }
    }
}

/// `.neplmeta` artifact store が materializer 入力を返せなかった理由。
///
/// この enum は fallback のための分岐点であり、診断文の一部ではない。body skip
/// 接続時はこれをそのまま統計・デバッグ表示へ流し、推測で artifact を使わない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeplMetaArtifactStoreReject {
    MissingArtifact { module_path: String },
    MissingModuleSurface,
    MissingModuleIdentity,
    MissingSourceKey,
    PayloadConsistency(NeplMetaArtifactPayloadReject),
    Compatibility(NeplMetaArtifactCompatibilityReject),
    Projection(NeplMetaMaterializerProjectionReject),
}

/// `.neplmeta` payload 自体が header と矛盾している理由。
///
/// compatibility check は payload decode 前の fail-closed 判定で使う。payload を読んだ後は、
/// header が主張する public signature hash と entry 数が実 payload と一致することも別に
/// 確認する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeplMetaArtifactPayloadReject {
    TypedPublicSignatureHash,
    PublicEntryCount,
    ModuleSurfaceHash,
    ModuleDependencyEdgeCount,
    ExportSurfaceHash,
    LocalExportCount,
    ReexportProjectionCount,
    StructuredPublicSurfaceHash,
    StructuredPublicSurfaceEntryCount,
}

impl NeplMetaArtifactPayloadReject {
    pub fn code(self) -> u32 {
        match self {
            Self::TypedPublicSignatureHash => 1,
            Self::PublicEntryCount => 2,
            Self::ModuleSurfaceHash => 3,
            Self::ModuleDependencyEdgeCount => 4,
            Self::ExportSurfaceHash => 5,
            Self::LocalExportCount => 6,
            Self::ReexportProjectionCount => 7,
            Self::StructuredPublicSurfaceHash => 8,
            Self::StructuredPublicSurfaceEntryCount => 9,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeplMetaMaterializerMvpReject {
    PayloadConsistency(NeplMetaArtifactPayloadReject),
    MissingSourceKey,
    MissingModuleSurface,
    MissingExportSurface,
    MissingModuleIdentity,
    MissingReexportTarget,
    PublicSurfaceBlocker(PublicSurfaceMaterializerBlocker),
    UnsupportedInclude,
    UnsupportedMerge,
    UnsupportedAlias,
    UnsupportedGlob,
    UnsupportedImplLookup,
}

/// `.neplmeta` export surface を materializer 入力へ投影できなかった理由。
///
/// `NeplMetaMaterializerMvpReject` は artifact 全体の前提条件を判定する。こちらは、
/// target artifact を読めた後に、現在の import clause で見える名前だけを
/// `TypedPublicSurfaceTable` へ絞る段階の拒否理由である。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeplMetaMaterializerProjectionReject {
    PayloadConsistency(NeplMetaArtifactPayloadReject),
    MissingSourceKey,
    MissingModuleSurface,
    MissingExportSurface,
    MissingModuleIdentity,
    PublicSurfaceBlocker(PublicSurfaceMaterializerBlocker),
    UnsupportedReexportProjection,
    UnsupportedAlias,
    UnsupportedMerge,
    UnsupportedGlob,
    UnsupportedExportKind {
        exported_name: String,
        kind: NeplMetaExportKind,
    },
    ExportedNameMissing {
        name: String,
    },
    ExportedSurfaceMissing {
        exported_name: String,
        origin_name: String,
    },
    ExportOriginModuleMismatch {
        exported_name: String,
        origin_module_path: String,
    },
    ExportAliasUnsupported {
        exported_name: String,
        origin_name: String,
    },
}

impl NeplMetaMaterializerProjectionReject {
    pub fn code(&self) -> u32 {
        match self {
            Self::PayloadConsistency(_) => 1,
            Self::MissingSourceKey => 2,
            Self::MissingModuleSurface => 3,
            Self::MissingExportSurface => 4,
            Self::MissingModuleIdentity => 5,
            Self::PublicSurfaceBlocker(_) => 6,
            Self::UnsupportedReexportProjection => 7,
            Self::UnsupportedAlias => 8,
            Self::UnsupportedMerge => 9,
            Self::UnsupportedGlob => 10,
            Self::UnsupportedExportKind { .. } => 11,
            Self::ExportedNameMissing { .. } => 12,
            Self::ExportedSurfaceMissing { .. } => 13,
            Self::ExportOriginModuleMismatch { .. } => 14,
            Self::ExportAliasUnsupported { .. } => 15,
        }
    }

    pub fn public_surface_blocker_reason_code(&self) -> u32 {
        match self {
            Self::PublicSurfaceBlocker(blocker) => blocker.reason.code(),
            _ => 0,
        }
    }

    pub fn public_surface_blocker_entry_kind_code(&self) -> u32 {
        match self {
            Self::PublicSurfaceBlocker(blocker) => blocker.entry_kind.code(),
            _ => 0,
        }
    }
}

impl NeplMetaMaterializerMvpReject {
    pub fn code(&self) -> u32 {
        match self {
            Self::PayloadConsistency(_) => 1,
            Self::MissingModuleSurface => 2,
            Self::MissingExportSurface => 3,
            Self::MissingModuleIdentity => 4,
            Self::MissingReexportTarget => 5,
            Self::PublicSurfaceBlocker(_) => 6,
            Self::UnsupportedInclude => 7,
            Self::UnsupportedMerge => 8,
            Self::UnsupportedAlias => 9,
            Self::UnsupportedGlob => 10,
            Self::UnsupportedImplLookup => 11,
            Self::MissingSourceKey => 12,
        }
    }

    pub fn public_surface_blocker_reason_code(&self) -> u32 {
        match self {
            Self::PublicSurfaceBlocker(blocker) => blocker.reason.code(),
            _ => 0,
        }
    }

    pub fn public_surface_blocker_entry_kind_code(&self) -> u32 {
        match self {
            Self::PublicSurfaceBlocker(blocker) => blocker.entry_kind.code(),
            _ => 0,
        }
    }
}

enum MaterializerExportNames {
    All,
    Named(Vec<String>),
}

fn requested_materializer_export_names(
    import_clause: Option<&NeplMetaImportClause>,
) -> Result<MaterializerExportNames, NeplMetaMaterializerProjectionReject> {
    match import_clause {
        None | Some(NeplMetaImportClause::Open) => Ok(MaterializerExportNames::All),
        Some(NeplMetaImportClause::DefaultAlias | NeplMetaImportClause::Alias(_)) => {
            Err(NeplMetaMaterializerProjectionReject::UnsupportedAlias)
        }
        Some(NeplMetaImportClause::Merge) => {
            Err(NeplMetaMaterializerProjectionReject::UnsupportedMerge)
        }
        Some(NeplMetaImportClause::Selective(items)) => {
            let mut names = Vec::new();
            for item in items {
                if item.glob {
                    return Err(NeplMetaMaterializerProjectionReject::UnsupportedGlob);
                }
                if item.alias.is_some() {
                    return Err(NeplMetaMaterializerProjectionReject::UnsupportedAlias);
                }
                names.push(item.name.clone());
            }
            Ok(MaterializerExportNames::Named(names))
        }
    }
}

pub fn nepl_meta_artifact_header_for_public_surface(
    target: CompileTarget,
    profile: BuildProfile,
    stdlib_content_hash: Option<u64>,
    dependency_public_surface_hash: Option<u64>,
    source_map: Option<&SourceMap>,
    public_signatures: &TypedPublicSignatureTable,
    module_surface: Option<&NeplMetaModuleSurface>,
    export_surface: Option<&NeplMetaExportSurface>,
    public_surface: &TypedPublicSurfaceTable,
) -> NeplMetaArtifactHeader {
    NeplMetaArtifactHeader::new(
        nepl_meta_compiler_identity_hash(),
        nepl_meta_target_hash(target),
        nepl_meta_profile_hash(profile),
        stdlib_content_hash,
        nepl_meta_source_key_hash(source_map, module_surface),
        dependency_public_surface_hash,
        public_signatures.stable_hash,
        usize_to_u32_saturating(public_signatures.entries.len()),
        module_surface.map(|surface| surface.stable_hash),
        module_surface.map(|surface| usize_to_u32_saturating(surface.dependency_edges.len())),
        export_surface.map(|surface| surface.stable_hash),
        export_surface.map(|surface| usize_to_u32_saturating(surface.local_exports.len())),
        export_surface.map(|surface| usize_to_u32_saturating(surface.reexport_projections.len())),
        public_surface.stable_hash,
        usize_to_u32_saturating(public_surface.entries.len()),
        crate::compiler::resource_summary_source_capability_policy_set_hash(source_map),
        Some(crate::compiler::resource_summary_private_effect_policy_hash()),
    )
}

pub fn nepl_meta_artifact_pre_typecheck_envelope_for_module_surface(
    target: CompileTarget,
    profile: BuildProfile,
    stdlib_content_hash: Option<u64>,
    dependency_public_surface_hash: Option<u64>,
    source_map: Option<&SourceMap>,
    module_surface: &NeplMetaModuleSurface,
) -> Result<NeplMetaArtifactPreTypecheckEnvelope, NeplMetaArtifactPreTypecheckEnvelopeReject> {
    let source_key_hash = nepl_meta_source_key_hash(source_map, Some(module_surface))
        .ok_or(NeplMetaArtifactPreTypecheckEnvelopeReject::MissingSourceKey)?;
    Ok(nepl_meta_artifact_pre_typecheck_envelope_for_module_surface_with_source_identity(
        target,
        profile,
        stdlib_content_hash,
        dependency_public_surface_hash,
        source_key_hash,
        crate::compiler::resource_summary_source_capability_policy_set_hash(source_map),
        module_surface,
    ))
}

/// 事前に検証済みの target source identity から pre-typecheck envelope を作る。
///
/// import/prelude edge probe では root compile 全体の `SourceMap` を使うと、target artifact の
/// 互換性が呼び出し元 root や同時に読み込まれた別 module に依存してしまう。そのため loader が
/// target module 単位で計算した source key と capability policy hash を明示的に渡す。
pub fn nepl_meta_artifact_pre_typecheck_envelope_for_module_surface_with_source_identity(
    target: CompileTarget,
    profile: BuildProfile,
    stdlib_content_hash: Option<u64>,
    dependency_public_surface_hash: Option<u64>,
    source_key_hash: u64,
    source_capability_policy_set_hash: Option<u64>,
    module_surface: &NeplMetaModuleSurface,
) -> NeplMetaArtifactPreTypecheckEnvelope {
    NeplMetaArtifactPreTypecheckEnvelope {
        schema_version: NEPL_META_ARTIFACT_SCHEMA_VERSION,
        compiler_identity_hash: nepl_meta_compiler_identity_hash(),
        target_hash: nepl_meta_target_hash(target),
        profile_hash: nepl_meta_profile_hash(profile),
        stdlib_content_hash,
        source_key_hash,
        dependency_public_surface_hash,
        module_surface_hash: Some(module_surface.stable_hash),
        module_dependency_edge_count: Some(usize_to_u32_saturating(
            module_surface.dependency_edges.len(),
        )),
        source_capability_policy_set_hash,
        private_effect_policy_hash: Some(crate::compiler::resource_summary_private_effect_policy_hash()),
    }
}

/// source text だけから `.neplmeta` 用の token-level source key を作る。
///
/// この値は `nepl_meta_source_key_hash` と同じ hash domain を使うが、edge probe のように
/// target source text を既に持っている経路では `SourceMap` 全体を経由しない。
pub fn nepl_meta_source_key_hash_for_source(source: &str) -> u64 {
    nepl_meta_hash_tag(
        "source-key",
        compiled_source_cache_key_part(source).as_str(),
    )
}

/// `.neplmeta` artifact の source-level invalidation key を作る。
///
/// この hash は依存 module の body を typecheck する前に、artifact が同じ source に由来するか
/// を判定するための境界である。値は lexer token stream 由来の `compiled_source_cache_key_part`
/// から作るため、コメントや位置だけの変更では同じ key になり、構文 token や directive の
/// 変更では変わる。`SourceMap` や module identity がない場合は artifact を事前再利用できない
/// ので `None` にし、将来の body skip は fail-closed に通常 load/typecheck へ戻る。
fn nepl_meta_source_key_hash(
    source_map: Option<&SourceMap>,
    module_surface: Option<&NeplMetaModuleSurface>,
) -> Option<u64> {
    let source_map = source_map?;
    let module_path = module_surface?.canonical_module_path.as_str();
    let mut matched_source = None;
    for (file_id, path) in source_map.iter_paths() {
        if path.as_str() == module_path {
            matched_source = source_map.get(file_id);
            break;
        }
    }
    let source = matched_source?;
    Some(nepl_meta_hash_tag(
        "source-key",
        compiled_source_cache_key_part(source).as_str(),
    ))
}

fn nepl_meta_module_surface_hash(
    canonical_module_path: &str,
    default_prelude_path: &str,
    no_prelude: bool,
    implicit_default_prelude: bool,
    dependency_edges: &[NeplMetaModuleDependencyEdge],
) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    hash_str(&mut hash, "module-surface-v1");
    hash_str(&mut hash, canonical_module_path);
    hash_str(&mut hash, default_prelude_path);
    hash_bool(&mut hash, no_prelude);
    hash_bool(&mut hash, implicit_default_prelude);
    hash_u32(&mut hash, usize_to_u32_saturating(dependency_edges.len()));
    for edge in dependency_edges {
        hash_module_dependency_edge(&mut hash, edge);
    }
    hash
}

fn nepl_meta_export_surface_hash(
    local_exports: &[NeplMetaExportEntry],
    reexport_projections: &[NeplMetaReexportProjection],
) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    hash_str(&mut hash, "export-surface-v1");
    hash_u32(&mut hash, usize_to_u32_saturating(local_exports.len()));
    for entry in local_exports {
        hash_export_entry(&mut hash, entry);
    }
    hash_u32(
        &mut hash,
        usize_to_u32_saturating(reexport_projections.len()),
    );
    for projection in reexport_projections {
        hash_reexport_projection(&mut hash, projection);
    }
    hash
}

fn materializer_mvp_reject_for_edge(
    kind: NeplMetaModuleDependencyKind,
    import_clause: Option<&NeplMetaImportClause>,
) -> Option<NeplMetaMaterializerMvpReject> {
    if kind == NeplMetaModuleDependencyKind::Include {
        return Some(NeplMetaMaterializerMvpReject::UnsupportedInclude);
    }
    match import_clause {
        Some(NeplMetaImportClause::DefaultAlias | NeplMetaImportClause::Alias(_)) => {
            Some(NeplMetaMaterializerMvpReject::UnsupportedAlias)
        }
        Some(NeplMetaImportClause::Merge) => {
            Some(NeplMetaMaterializerMvpReject::UnsupportedMerge)
        }
        Some(NeplMetaImportClause::Selective(items)) => {
            for item in items {
                if item.glob {
                    return Some(NeplMetaMaterializerMvpReject::UnsupportedGlob);
                }
                if item.alias.is_some() {
                    return Some(NeplMetaMaterializerMvpReject::UnsupportedAlias);
                }
            }
            None
        }
        Some(NeplMetaImportClause::Open) | None => None,
    }
}

fn hash_export_entry(hash: &mut u64, entry: &NeplMetaExportEntry) {
    hash_str(hash, "local-export");
    hash_str(hash, &entry.exported_name);
    hash_str(hash, &entry.origin_module_path);
    hash_str(hash, &entry.origin_name);
    hash_u8(
        hash,
        match entry.kind {
            NeplMetaExportKind::Callable => 1,
            NeplMetaExportKind::Struct => 2,
            NeplMetaExportKind::Enum => 3,
            NeplMetaExportKind::Trait => 4,
        },
    );
}

fn hash_reexport_projection(hash: &mut u64, projection: &NeplMetaReexportProjection) {
    hash_str(hash, "reexport-projection");
    hash_u32(hash, projection.source_order);
    hash_str(hash, &projection.target_module_path);
    hash_u8(
        hash,
        match projection.kind {
            NeplMetaModuleDependencyKind::Prelude => 1,
            NeplMetaModuleDependencyKind::Import => 2,
            NeplMetaModuleDependencyKind::Include => 3,
        },
    );
    match &projection.import_clause {
        Some(clause) => {
            hash_u8(hash, 1);
            hash_import_clause(hash, clause);
        }
        None => hash_u8(hash, 0),
    }
}

fn hash_module_dependency_edge(hash: &mut u64, edge: &NeplMetaModuleDependencyEdge) {
    hash_str(hash, "edge");
    hash_u8(
        hash,
        match edge.kind {
            NeplMetaModuleDependencyKind::Prelude => 1,
            NeplMetaModuleDependencyKind::Import => 2,
            NeplMetaModuleDependencyKind::Include => 3,
        },
    );
    hash_str(hash, &edge.target_path);
    hash_u8(
        hash,
        match edge.visibility {
            NeplMetaVisibility::Pub => 1,
            NeplMetaVisibility::Private => 2,
        },
    );
    match &edge.import_clause {
        Some(clause) => {
            hash_u8(hash, 1);
            hash_import_clause(hash, clause);
        }
        None => hash_u8(hash, 0),
    }
    hash_bool(hash, edge.public_reexport);
    hash_u32(hash, edge.source_order);
}

fn hash_import_clause(hash: &mut u64, clause: &NeplMetaImportClause) {
    match clause {
        NeplMetaImportClause::DefaultAlias => hash_u8(hash, 1),
        NeplMetaImportClause::Alias(name) => {
            hash_u8(hash, 2);
            hash_str(hash, name);
        }
        NeplMetaImportClause::Open => hash_u8(hash, 3),
        NeplMetaImportClause::Selective(items) => {
            hash_u8(hash, 4);
            hash_u32(hash, usize_to_u32_saturating(items.len()));
            for item in items {
                hash_str(hash, &item.name);
                match &item.alias {
                    Some(alias) => {
                        hash_u8(hash, 1);
                        hash_str(hash, alias);
                    }
                    None => hash_u8(hash, 0),
                }
                hash_bool(hash, item.glob);
            }
        }
        NeplMetaImportClause::Merge => hash_u8(hash, 5),
    }
}

pub fn nepl_meta_compiler_identity_hash() -> u64 {
    nepl_meta_hash_tag("compiler", NEPL_META_COMPILER_IDENTITY_INPUT)
}

pub fn nepl_meta_target_hash(target: CompileTarget) -> u64 {
    nepl_meta_hash_tag("target", target_tag(target))
}

pub fn nepl_meta_profile_hash(profile: BuildProfile) -> u64 {
    nepl_meta_hash_tag("profile", profile_tag(profile))
}

fn target_tag(target: CompileTarget) -> &'static str {
    match target {
        CompileTarget::Wasm => "wasm",
        CompileTarget::Wasi => "wasi",
        CompileTarget::Wasix => "wasix",
        CompileTarget::Llvm => "llvm",
    }
}

fn profile_tag(profile: BuildProfile) -> &'static str {
    match profile {
        BuildProfile::Debug => "debug",
        BuildProfile::Release => "release",
    }
}

fn nepl_meta_hash_tag(domain: &str, value: &str) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    hash_str(&mut hash, NEPL_META_ARTIFACT_HASH_VERSION);
    hash_str(&mut hash, domain);
    hash_str(&mut hash, value);
    hash
}

fn nepl_obj_hash_tag(domain: &str, value: &str) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    hash_str(&mut hash, NEPL_OBJ_DIRECT_CALL_HASH_VERSION);
    hash_str(&mut hash, domain);
    hash_str(&mut hash, value);
    hash
}

#[allow(clippy::too_many_arguments)]
fn nepl_obj_direct_call_key_hash(
    compiler_identity_hash: u64,
    target_hash: u64,
    profile_hash: u64,
    stdlib_content_hash: Option<u64>,
    backend_feature_set_hash: u64,
    link_symbol_hash: u64,
    target_source_key_hash: u64,
    selected_callable_body_hash: u64,
    generic_instantiation_hash: u64,
    dependency_public_surface_hash: u64,
    source_capability_policy_hash: u64,
    private_effect_policy_hash: u64,
) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    hash_str(&mut hash, NEPL_OBJ_DIRECT_CALL_HASH_VERSION);
    hash_u32(&mut hash, NEPL_OBJ_DIRECT_CALL_SCHEMA_VERSION);
    hash_u64(&mut hash, compiler_identity_hash);
    hash_u64(&mut hash, target_hash);
    hash_u64(&mut hash, profile_hash);
    hash_optional_u64(&mut hash, stdlib_content_hash);
    hash_u64(&mut hash, backend_feature_set_hash);
    hash_u64(&mut hash, link_symbol_hash);
    hash_u64(&mut hash, target_source_key_hash);
    hash_u64(&mut hash, selected_callable_body_hash);
    hash_u64(&mut hash, generic_instantiation_hash);
    hash_u64(&mut hash, dependency_public_surface_hash);
    hash_u64(&mut hash, source_capability_policy_hash);
    hash_u64(&mut hash, private_effect_policy_hash);
    hash
}

fn nepl_obj_direct_call_fragment_hash(
    key: &NeplObjDirectCallKey,
    backend: &NeplObjDirectCallBackendFragment,
) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    hash_str(&mut hash, NEPL_OBJ_DIRECT_CALL_HASH_VERSION);
    hash_str(&mut hash, "fragment");
    hash_u64(&mut hash, key.stable_hash);
    match backend {
        NeplObjDirectCallBackendFragment::Wasm(fragment) => {
            hash_str(&mut hash, "wasm");
            hash_wasm_val_type_list(&mut hash, &fragment.params);
            hash_wasm_val_type_list(&mut hash, &fragment.results);
            hash_u32(
                &mut hash,
                usize_to_u32_saturating(fragment.function_body_bytes.len()),
            );
            hash_bytes(&mut hash, &fragment.function_body_bytes);
            hash_u32(
                &mut hash,
                usize_to_u32_saturating(fragment.direct_call_relocations.len()),
            );
            for relocation in &fragment.direct_call_relocations {
                hash_u32(&mut hash, relocation.byte_offset);
                hash_u64(
                    &mut hash,
                    public_callable_link_symbol_stable_hash(&relocation.target),
                );
            }
        }
    }
    hash
}

fn hash_wasm_val_type_list(hash: &mut u64, values: &[NeplObjWasmValType]) {
    hash_u32(hash, usize_to_u32_saturating(values.len()));
    for value in values {
        hash_u8(
            hash,
            match value {
                NeplObjWasmValType::I32 => 1,
                NeplObjWasmValType::I64 => 2,
                NeplObjWasmValType::F32 => 3,
                NeplObjWasmValType::F64 => 4,
            },
        );
    }
}

fn hash_str(hash: &mut u64, value: &str) {
    hash_bytes(hash, value.as_bytes());
    hash_bytes(hash, &[0]);
}

fn hash_bool(hash: &mut u64, value: bool) {
    hash_u8(hash, if value { 1 } else { 0 });
}

fn hash_u8(hash: &mut u64, value: u8) {
    hash_bytes(hash, &[value]);
}

fn hash_u32(hash: &mut u64, value: u32) {
    hash_bytes(hash, &value.to_le_bytes());
}

fn hash_u64(hash: &mut u64, value: u64) {
    hash_bytes(hash, &value.to_le_bytes());
}

fn hash_optional_u64(hash: &mut u64, value: Option<u64>) {
    match value {
        Some(value) => {
            hash_bool(hash, true);
            hash_u64(hash, value);
        }
        None => hash_bool(hash, false),
    }
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(FNV1A64_PRIME);
    }
}

fn usize_to_u32_saturating(value: usize) -> u32 {
    if value > u32::MAX as usize {
        u32::MAX
    } else {
        value as u32
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::{
        nepl_meta_artifact_header_for_public_surface,
        nepl_meta_artifact_pre_typecheck_envelope_for_module_surface, NeplMetaArtifact,
        NeplMetaArtifactCompatibilityReject, NeplMetaArtifactHeader, NeplMetaArtifactPayloadReject,
        NeplMetaArtifactPreTypecheckEnvelopeReject, NeplMetaArtifactProbeRejectKind,
        NeplMetaArtifactStore,
        NeplMetaArtifactStoreReject, NeplMetaExportKind, NeplMetaExportSurface,
        NeplMetaImportClause, NeplMetaImportItem, NeplMetaMaterializerMvpReject,
        NeplMetaMaterializerProjectionReject, NeplMetaModuleDependencyEdge,
        NeplMetaModuleDependencyKind, NeplMetaModuleSurface, NeplMetaVisibility,
        NeplObjDirectCallBackendFragment, NeplObjDirectCallFragmentArtifact,
        NeplObjDirectCallFragmentLookupProbe, NeplObjDirectCallFragmentReject,
        NeplObjDirectCallFragmentStore, NeplObjDirectCallFragmentStoreReject,
        NeplObjDirectCallKey, NeplObjWasmDirectCallFragment, NeplObjWasmDirectCallRelocation,
        NeplObjWasmValType,
    };
    use crate::compiler::{BuildProfile, CompileTarget};
    use crate::source_map::SourceMap;
    use crate::typecheck::{
        PublicCallableLinkSymbol, PublicCallableSurface, PublicEffect, PublicEnumSurface,
        PublicEnumVariantSurface, PublicNominalTypeIdentity, PublicNominalTypeKind,
        PublicStructConstructorPolicy, PublicStructSurface, PublicSurfaceShape,
        PublicTraitCapability, PublicTraitIdentity, PublicTraitSurface, PublicTypeTerm,
        TypedPublicSignatureEntry, TypedPublicSignatureKind, TypedPublicSignatureTable,
        TypedPublicSurfaceEntry, TypedPublicSurfaceTable,
    };

    fn signature_table(name: &str, signature: &str) -> TypedPublicSignatureTable {
        TypedPublicSignatureTable::new(Vec::from([TypedPublicSignatureEntry::new(
            TypedPublicSignatureKind::Callable,
            name.into(),
            signature.into(),
            false,
        )]))
    }

    fn signature_table_for_names(names: &[&str]) -> TypedPublicSignatureTable {
        TypedPublicSignatureTable::new(
            names
                .iter()
                .map(|name| {
                    TypedPublicSignatureEntry::new(
                        TypedPublicSignatureKind::Callable,
                        (*name).into(),
                        "fn unit i32".into(),
                        false,
                    )
                })
                .collect(),
        )
    }

    fn signature_table_for_kinds(
        entries: &[(TypedPublicSignatureKind, &str, &str)],
    ) -> TypedPublicSignatureTable {
        TypedPublicSignatureTable::new(
            entries
                .iter()
                .map(|(kind, name, signature)| {
                    TypedPublicSignatureEntry::new(
                        *kind,
                        (*name).into(),
                        (*signature).into(),
                        false,
                    )
                })
                .collect(),
        )
    }

    fn surface_table(name: &str, result: PublicTypeTerm) -> TypedPublicSurfaceTable {
        TypedPublicSurfaceTable::new(Vec::from([TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Callable,
            name: name.into(),
            exported: true,
            surface: PublicSurfaceShape::Callable(PublicCallableSurface {
                ty: PublicTypeTerm::Function {
                    type_params: Vec::new(),
                    params: Vec::from([PublicTypeTerm::Unit]),
                    result: alloc::boxed::Box::new(result),
                    effect: PublicEffect::Pure,
                },
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: None,
                type_param_bounds: Vec::new(),
            }),
        }]))
    }

    fn materializable_surface_table(name: &str, result: PublicTypeTerm) -> TypedPublicSurfaceTable {
        TypedPublicSurfaceTable::new(Vec::from([TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Callable,
            name: name.into(),
            exported: true,
            surface: PublicSurfaceShape::Callable(PublicCallableSurface {
                ty: PublicTypeTerm::Function {
                    type_params: Vec::new(),
                    params: Vec::from([PublicTypeTerm::Unit]),
                    result: alloc::boxed::Box::new(result),
                    effect: PublicEffect::Pure,
                },
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: Some(PublicCallableLinkSymbol {
                    source_path: "/stdlib/core/math.nepl".into(),
                    name: name.into(),
                    signature_hash: 42,
                }),
                type_param_bounds: Vec::new(),
            }),
        }]))
    }

    fn callable_link_symbol(name: &str, signature_hash: u64) -> PublicCallableLinkSymbol {
        PublicCallableLinkSymbol {
            source_path: "/stdlib/core/math.nepl".into(),
            name: name.into(),
            signature_hash,
        }
    }

    fn neplobj_direct_call_key(
        link_symbol: PublicCallableLinkSymbol,
        target_source_key_hash: u64,
        selected_callable_body_hash: u64,
        generic_instantiation_hash: u64,
    ) -> NeplObjDirectCallKey {
        NeplObjDirectCallKey::new(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(0x5a),
            link_symbol,
            target_source_key_hash,
            selected_callable_body_hash,
            generic_instantiation_hash,
            0x70,
            0x80,
        )
    }

    fn neplobj_direct_call_fragment(
        key: NeplObjDirectCallKey,
        results: Vec<NeplObjWasmValType>,
        function_body_bytes: Vec<u8>,
        direct_call_relocations: Vec<NeplObjWasmDirectCallRelocation>,
    ) -> NeplObjDirectCallFragmentArtifact {
        NeplObjDirectCallFragmentArtifact::new(
            key,
            NeplObjDirectCallBackendFragment::Wasm(
                NeplObjWasmDirectCallFragment::new(
                    Vec::new(),
                    results,
                    function_body_bytes,
                    direct_call_relocations,
                )
                .expect("test fixture uses a valid wasm direct-call fragment"),
            ),
        )
    }

    fn materializable_multi_surface_table(names: &[&str]) -> TypedPublicSurfaceTable {
        TypedPublicSurfaceTable::new(
            names
                .iter()
                .map(|name| TypedPublicSurfaceEntry {
                    kind: TypedPublicSignatureKind::Callable,
                    name: (*name).into(),
                    exported: true,
                    surface: PublicSurfaceShape::Callable(PublicCallableSurface {
                        ty: PublicTypeTerm::Function {
                            type_params: Vec::new(),
                            params: Vec::from([PublicTypeTerm::Unit]),
                            result: alloc::boxed::Box::new(PublicTypeTerm::I32),
                            effect: PublicEffect::Pure,
                        },
                        no_shadow: false,
                        arity: 1,
                        effect: PublicEffect::Pure,
                        field_accessor: None,
                        link_symbol: Some(PublicCallableLinkSymbol {
                            source_path: "/stdlib/core/math.nepl".into(),
                            name: (*name).into(),
                            signature_hash: 42,
                        }),
                        type_param_bounds: Vec::new(),
                    }),
                })
                .collect(),
        )
    }

    fn semantic_trait_entry(name: &str) -> TypedPublicSurfaceEntry {
        TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Trait,
            name: name.into(),
            exported: false,
            surface: PublicSurfaceShape::Trait(PublicTraitSurface {
                identity: Some(PublicTraitIdentity {
                    source_path: "/stdlib/core/traits/copy/primitive.nepl".into(),
                    name: name.into(),
                    arity: 0,
                    definition_hash: 7,
                }),
                type_params: Vec::new(),
                capabilities: Vec::from([PublicTraitCapability::Copy]),
                methods: Vec::new(),
            }),
        }
    }

    fn exported_struct_entry(name: &str) -> TypedPublicSurfaceEntry {
        TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Struct,
            name: name.into(),
            exported: true,
            surface: PublicSurfaceShape::Struct(PublicStructSurface {
                identity: Some(PublicNominalTypeIdentity {
                    kind: PublicNominalTypeKind::Struct,
                    source_path: "/stdlib/core/data.nepl".into(),
                    name: name.into(),
                    arity: 0,
                    definition_hash: 11,
                }),
                type_params: Vec::new(),
                fields: Vec::new(),
                constructor_policy: PublicStructConstructorPolicy::Public,
            }),
        }
    }

    fn exported_enum_entry(name: &str) -> TypedPublicSurfaceEntry {
        TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Enum,
            name: name.into(),
            exported: true,
            surface: PublicSurfaceShape::Enum(PublicEnumSurface {
                identity: Some(PublicNominalTypeIdentity {
                    kind: PublicNominalTypeKind::Enum,
                    source_path: "/stdlib/core/data.nepl".into(),
                    name: name.into(),
                    arity: 0,
                    definition_hash: 13,
                }),
                type_params: Vec::new(),
                variants: Vec::from([PublicEnumVariantSurface {
                    name: "Some".into(),
                    payload: Some(PublicTypeTerm::I32),
                }]),
            }),
        }
    }

    fn exported_trait_entry(name: &str) -> TypedPublicSurfaceEntry {
        let mut entry = semantic_trait_entry(name);
        entry.exported = true;
        entry
    }

    fn test_header(
        public_signatures: &TypedPublicSignatureTable,
        module_surface: Option<&NeplMetaModuleSurface>,
        public_surface: &TypedPublicSurfaceTable,
        dependency_hash: Option<u64>,
    ) -> NeplMetaArtifactHeader {
        let export_surface = module_surface.map(|surface| {
            NeplMetaExportSurface::from_module_and_public_surface(surface, public_surface)
        });
        let mut source_map = SourceMap::new();
        if let Some(surface) = module_surface {
            source_map.add(
                surface.canonical_module_path.as_str(),
                "pub fn answer %fn unit i32 \\:\n    1\n".into(),
            );
        }
        nepl_meta_artifact_header_for_public_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            dependency_hash,
            module_surface.map(|_| &source_map),
            public_signatures,
            module_surface,
            export_surface.as_ref(),
            public_surface,
        )
    }

    fn module_surface(path: &str) -> NeplMetaModuleSurface {
        module_surface_with_edges(
            path,
            Vec::from([NeplMetaModuleDependencyEdge {
                kind: NeplMetaModuleDependencyKind::Import,
                target_path: "/stdlib/core/result.nepl".into(),
                visibility: NeplMetaVisibility::Pub,
                import_clause: Some(NeplMetaImportClause::Open),
                public_reexport: true,
                source_order: 0,
            }]),
        )
    }

    fn module_surface_without_edges(path: &str) -> NeplMetaModuleSurface {
        module_surface_with_edges(path, Vec::new())
    }

    fn module_surface_with_edges(
        path: &str,
        edges: Vec<NeplMetaModuleDependencyEdge>,
    ) -> NeplMetaModuleSurface {
        NeplMetaModuleSurface::new(
            path.into(),
            "/stdlib/std/prelude_base.nepl".into(),
            false,
            true,
            edges,
        )
    }

    fn artifact_for_materializer(
        module_surface: NeplMetaModuleSurface,
        public_surface: TypedPublicSurfaceTable,
    ) -> NeplMetaArtifact {
        let public_signatures = signature_table("answer", "fn unit i32");
        let export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&module_surface, &public_surface);
        NeplMetaArtifact::new(
            test_header(
                &public_signatures,
                Some(&module_surface),
                &public_surface,
                None,
            ),
            public_signatures,
            Some(module_surface),
            Some(export_surface),
            public_surface,
        )
    }

    fn artifact_for_materializer_with_names(
        module_surface: NeplMetaModuleSurface,
        public_surface: TypedPublicSurfaceTable,
        signature_names: &[&str],
    ) -> NeplMetaArtifact {
        let public_signatures = signature_table_for_names(signature_names);
        let export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&module_surface, &public_surface);
        NeplMetaArtifact::new(
            test_header(
                &public_signatures,
                Some(&module_surface),
                &public_surface,
                None,
            ),
            public_signatures,
            Some(module_surface),
            Some(export_surface),
            public_surface,
        )
    }

    /// `.neplmeta` header は public signature hash を invalidation boundary にする。
    /// function body や typed HIR を payload に入れず、依存側 typecheck に必要な公開面だけを
    /// stable value として運ぶ前提を固定する。
    #[test]
    fn neplmeta_header_accepts_matching_public_surface() {
        let public_signatures = signature_table("answer", "fn unit i32");
        let public_surface = surface_table("answer", PublicTypeTerm::I32);
        let module_surface = module_surface("/stdlib/core/math.nepl");
        let export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&module_surface, &public_surface);
        let header = test_header(
            &public_signatures,
            Some(&module_surface),
            &public_surface,
            Some(11),
        );
        let artifact = NeplMetaArtifact::new(
            header,
            public_signatures,
            Some(module_surface),
            Some(export_surface),
            public_surface,
        );

        assert_eq!(artifact.compatibility_reject(header), None);
        assert_eq!(artifact.payload_consistency_reject(), None);
        assert_eq!(artifact.entry_count(), 1);
    }

    /// source key hash は `.neplmeta` を import boundary で使う前の stale artifact
    /// 検出に使う。通常コメントだけの編集では同じ source key になり、public signature が
    /// 同じでも式 token が変わる編集では compatibility check が fail-closed に拒否する。
    #[test]
    fn neplmeta_header_tracks_source_key_without_comment_noise() {
        let public_signatures = signature_table("answer", "fn unit i32");
        let public_surface = surface_table("answer", PublicTypeTerm::I32);
        let module_surface = module_surface_without_edges("/stdlib/core/math.nepl");
        let export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&module_surface, &public_surface);
        let mut base_map = SourceMap::new();
        base_map.add("/stdlib/core/math.nepl", "pub fn answer %fn unit i32 \\:\n    1\n".into());
        let mut comment_map = SourceMap::new();
        comment_map.add(
            "/stdlib/core/math.nepl",
            "// ordinary comment\npub fn answer %fn unit i32 \\:\n    1 // trailing\n".into(),
        );
        let mut edited_map = SourceMap::new();
        edited_map.add("/stdlib/core/math.nepl", "pub fn answer %fn unit i32 \\:\n    2\n".into());

        let base = nepl_meta_artifact_header_for_public_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            None,
            Some(&base_map),
            &public_signatures,
            Some(&module_surface),
            Some(&export_surface),
            &public_surface,
        );
        let comment = nepl_meta_artifact_header_for_public_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            None,
            Some(&comment_map),
            &public_signatures,
            Some(&module_surface),
            Some(&export_surface),
            &public_surface,
        );
        let edited = nepl_meta_artifact_header_for_public_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            None,
            Some(&edited_map),
            &public_signatures,
            Some(&module_surface),
            Some(&export_surface),
            &public_surface,
        );

        assert!(base.source_key_hash.is_some());
        assert_eq!(base.source_key_hash, comment.source_key_hash);
        assert_eq!(
            base.compatibility_reject(comment),
            None,
            "comment-only edits must not invalidate source-level interface artifacts"
        );
        assert_ne!(base.source_key_hash, edited.source_key_hash);
        assert_eq!(
            base.compatibility_reject(edited),
            Some(NeplMetaArtifactCompatibilityReject::SourceKey)
        );
    }

    /// pre-typecheck envelope は typed public surface を要求しない。
    /// 依存先 body を typecheck する前に、現在 source の token-level key だけで stale artifact
    /// を拒否できることを固定する。
    #[test]
    fn neplmeta_pre_typecheck_envelope_rejects_body_token_edit() {
        let public_signatures = signature_table("answer", "fn unit i32");
        let public_surface = surface_table("answer", PublicTypeTerm::I32);
        let module_surface = module_surface_without_edges("/stdlib/core/math.nepl");
        let export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&module_surface, &public_surface);
        let mut stored_map = SourceMap::new();
        stored_map.add("/stdlib/core/math.nepl", "pub fn answer %fn unit i32 \\:\n    1\n".into());
        let mut current_map = SourceMap::new();
        current_map.add("/stdlib/core/math.nepl", "pub fn answer %fn unit i32 \\:\n    2\n".into());
        let stored_header = nepl_meta_artifact_header_for_public_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            None,
            Some(&stored_map),
            &public_signatures,
            Some(&module_surface),
            Some(&export_surface),
            &public_surface,
        );
        let current_envelope = nepl_meta_artifact_pre_typecheck_envelope_for_module_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            None,
            Some(&current_map),
            &module_surface,
        )
        .expect("current source should produce pre-typecheck envelope");

        assert_eq!(
            stored_header.pre_typecheck_compatibility_reject(current_envelope),
            Some(NeplMetaArtifactCompatibilityReject::SourceKey)
        );
    }

    /// source key を作れない artifact は body skip の authority にしない。
    /// `None == None` を compatible として扱うと、SourceMap や module path が欠落した artifact が
    /// import materializer へ流れるため、pre-typecheck envelope 作成時点で fail-closed にする。
    #[test]
    fn neplmeta_pre_typecheck_envelope_requires_source_key() {
        let module_surface = module_surface_without_edges("/stdlib/core/math.nepl");
        let mut wrong_map = SourceMap::new();
        wrong_map.add("/stdlib/core/other.nepl", "pub fn answer %fn unit i32 \\:\n    1\n".into());

        assert_eq!(
            nepl_meta_artifact_pre_typecheck_envelope_for_module_surface(
                CompileTarget::Wasm,
                BuildProfile::Debug,
                Some(7),
                None,
                Some(&wrong_map),
                &module_surface,
            ),
            Err(NeplMetaArtifactPreTypecheckEnvelopeReject::MissingSourceKey)
        );
    }

    /// export surface は local public entry と pub re-export projection を別々に保存する。
    /// glob や open import をこの段階で展開すると target artifact 欠落時に誤ったauthorityを
    /// 作るため、`.neplmeta` には projection として保持して materializer が fail-closed に扱う。
    #[test]
    fn neplmeta_export_surface_preserves_local_exports_and_reexport_projection() {
        let public_surface = surface_table("answer", PublicTypeTerm::I32);
        let module_surface = module_surface("/stdlib/core/math.nepl");
        let export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&module_surface, &public_surface);

        assert_eq!(export_surface.local_exports.len(), 1);
        assert_eq!(export_surface.local_exports[0].exported_name, "answer");
        assert_eq!(
            export_surface.local_exports[0].origin_module_path,
            "/stdlib/core/math.nepl"
        );
        assert_eq!(export_surface.local_exports[0].origin_name, "answer");
        assert_eq!(export_surface.local_exports[0].kind, NeplMetaExportKind::Callable);
        assert_eq!(export_surface.reexport_projections.len(), 1);
        assert_eq!(
            export_surface.reexport_projections[0].target_module_path,
            "/stdlib/core/result.nepl"
        );
        assert_eq!(
            export_surface.reexport_projections[0].kind,
            NeplMetaModuleDependencyKind::Import
        );
        assert_eq!(
            export_surface.reexport_projections[0].import_clause,
            Some(NeplMetaImportClause::Open)
        );
    }

    /// semantic-only trait は dependency typecheck の復元材料であり、import 先の visible
    /// namespace へ export してはいけない。export surface は `exported=true` の entry だけを
    /// local export として扱い、private capability trait を public API に昇格しない。
    #[test]
    fn neplmeta_export_surface_excludes_semantic_only_trait_entries() {
        let module_surface = module_surface_without_edges("/stdlib/core/traits/copy/primitive.nepl");
        let mut entries = materializable_surface_table("clone_i32", PublicTypeTerm::I32).entries;
        entries.push(semantic_trait_entry("Clone"));
        let public_surface = TypedPublicSurfaceTable::new(entries);
        let export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&module_surface, &public_surface);

        assert_eq!(export_surface.local_exports.len(), 1);
        assert_eq!(export_surface.local_exports[0].exported_name, "clone_i32");
        assert!(export_surface
            .local_exports
            .iter()
            .all(|entry| entry.exported_name != "Clone"));
    }

    /// target artifact が読めた後でも、materializer へ渡すのは import clause から見える
    /// local export だけである。Open import は local callable export 全体を投影する。
    #[test]
    fn neplmeta_projection_open_import_keeps_local_callable_exports() {
        let artifact = artifact_for_materializer_with_names(
            module_surface_without_edges("/stdlib/core/math.nepl"),
            materializable_multi_surface_table(&["answer", "double"]),
            &["answer", "double"],
        );

        let projected = artifact
            .materializer_import_public_surface_mvp(Some(&NeplMetaImportClause::Open))
            .expect("open import projection");

        assert_eq!(projected.entries.len(), 2);
        assert!(projected.entries.iter().any(|entry| entry.name == "answer"));
        assert!(projected.entries.iter().any(|entry| entry.name == "double"));
    }

    /// projection は callable に限らず、artifact が保持する local export surface を kind
    /// 付きで復元する。nominal type や trait をここで落とすと、dependency source を
    /// 読まない経路では型名や trait bound の候補を再構成できなくなる。
    #[test]
    fn neplmeta_projection_open_import_keeps_local_nominal_and_trait_exports() {
        let module_surface = module_surface_without_edges("/stdlib/core/data.nepl");
        let public_surface = TypedPublicSurfaceTable::new(Vec::from([
            exported_struct_entry("Box"),
            exported_enum_entry("Option"),
            exported_trait_entry("Show"),
        ]));
        let public_signatures = signature_table_for_kinds(&[
            (TypedPublicSignatureKind::Struct, "Box", ";fields=;constructor=public"),
            (TypedPublicSignatureKind::Enum, "Option", ";variants=Some:i32"),
            (TypedPublicSignatureKind::Trait, "Show", ";capabilities=;methods="),
        ]);
        let export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&module_surface, &public_surface);
        let artifact = NeplMetaArtifact::new(
            test_header(
                &public_signatures,
                Some(&module_surface),
                &public_surface,
                None,
            ),
            public_signatures,
            Some(module_surface),
            Some(export_surface),
            public_surface,
        );

        let projected = artifact
            .materializer_import_public_surface_mvp(Some(&NeplMetaImportClause::Open))
            .expect("open import projection with nominal and trait exports");

        assert_eq!(projected.entries.len(), 3);
        assert!(projected
            .entries
            .iter()
            .any(|entry| entry.name == "Box" && entry.kind == TypedPublicSignatureKind::Struct));
        assert!(projected
            .entries
            .iter()
            .any(|entry| entry.name == "Option" && entry.kind == TypedPublicSignatureKind::Enum));
        assert!(projected
            .entries
            .iter()
            .any(|entry| entry.name == "Show" && entry.kind == TypedPublicSignatureKind::Trait));
    }

    /// import projection では visible export に加えて semantic-only support entry を保持する。
    /// private capability trait は利用者の名前空間には出ないが、後続の trait/impl
    /// materializer が impl header を復元するために必要になる。
    #[test]
    fn neplmeta_projection_keeps_semantic_support_entries_outside_export_set() {
        let module_surface = module_surface_without_edges("/stdlib/core/traits/copy/primitive.nepl");
        let mut public_surface = materializable_surface_table("clone_i32", PublicTypeTerm::I32);
        public_surface.entries.push(semantic_trait_entry("Clone"));
        public_surface = TypedPublicSurfaceTable::new(public_surface.entries);
        let artifact =
            artifact_for_materializer_with_names(module_surface, public_surface, &["clone_i32"]);

        let projected = artifact
            .materializer_import_public_surface_mvp(Some(&NeplMetaImportClause::Open))
            .expect("open import projection with semantic support");

        assert_eq!(projected.entries.len(), 2);
        assert!(projected
            .entries
            .iter()
            .any(|entry| entry.name == "clone_i32" && entry.exported));
        assert!(projected
            .entries
            .iter()
            .any(|entry| entry.name == "Clone" && !entry.exported));
    }

    /// selective import は alias や glob を使わない名前だけを受け入れ、該当する callable
    /// surface だけを materializer へ渡す。これにより不要な overload 候補を依存側 `Env` へ
    /// 注入しない。
    #[test]
    fn neplmeta_projection_selective_import_keeps_requested_callable_only() {
        let artifact = artifact_for_materializer_with_names(
            module_surface_without_edges("/stdlib/core/math.nepl"),
            materializable_multi_surface_table(&["answer", "double"]),
            &["answer", "double"],
        );

        let projected = artifact
            .materializer_import_public_surface_mvp(Some(&NeplMetaImportClause::Selective(
                Vec::from([NeplMetaImportItem {
                    name: "double".into(),
                    alias: None,
                    glob: false,
                }]),
            )))
            .expect("selective import projection");

        assert_eq!(projected.entries.len(), 1);
        assert_eq!(projected.entries[0].name, "double");
    }

    /// selective import でも要求された名前の export kind を保つ。callable だけに絞ると、
    /// `#import "core/option" { Option }` のような型だけを読む差分経路が source fallback
    /// へ戻り続ける。
    #[test]
    fn neplmeta_projection_selective_import_keeps_requested_nominal_export() {
        let module_surface = module_surface_without_edges("/stdlib/core/data.nepl");
        let public_surface = TypedPublicSurfaceTable::new(Vec::from([
            exported_struct_entry("Box"),
            exported_enum_entry("Option"),
        ]));
        let public_signatures = signature_table_for_kinds(&[
            (TypedPublicSignatureKind::Struct, "Box", ";fields=;constructor=public"),
            (TypedPublicSignatureKind::Enum, "Option", ";variants=Some:i32"),
        ]);
        let export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&module_surface, &public_surface);
        let artifact = NeplMetaArtifact::new(
            test_header(
                &public_signatures,
                Some(&module_surface),
                &public_surface,
                None,
            ),
            public_signatures,
            Some(module_surface),
            Some(export_surface),
            public_surface,
        );

        let projected = artifact
            .materializer_import_public_surface_mvp(Some(&NeplMetaImportClause::Selective(
                Vec::from([NeplMetaImportItem {
                    name: "Option".into(),
                    alias: None,
                    glob: false,
                }]),
            )))
            .expect("selective import projection with nominal export");

        assert_eq!(projected.entries.len(), 1);
        assert_eq!(projected.entries[0].name, "Option");
        assert_eq!(projected.entries[0].kind, TypedPublicSignatureKind::Enum);
    }

    /// selective import が要求する名前が local export にない場合、re-export か欠落かを
    /// target artifact 単体では推測しない。missing name は専用 reason で fail-closed にする。
    #[test]
    fn neplmeta_projection_rejects_missing_selective_name() {
        let artifact = artifact_for_materializer_with_names(
            module_surface_without_edges("/stdlib/core/math.nepl"),
            materializable_multi_surface_table(&["answer"]),
            &["answer"],
        );

        let reject = artifact
            .materializer_import_public_surface_mvp(Some(&NeplMetaImportClause::Selective(
                Vec::from([NeplMetaImportItem {
                    name: "missing".into(),
                    alias: None,
                    glob: false,
                }]),
            )))
            .unwrap_err();

        assert_eq!(
            reject,
            super::NeplMetaMaterializerProjectionReject::ExportedNameMissing {
                name: "missing".into()
            }
        );
    }

    /// re-export projection はさらに target artifact を読む必要がある。Open import でもこの
    /// checkpoint では展開せず、通常 source load / typecheck fallback へ戻す。
    #[test]
    fn neplmeta_projection_rejects_reexport_projection_until_target_artifact_exists() {
        let artifact = artifact_for_materializer(
            module_surface("/stdlib/core/math.nepl"),
            materializable_surface_table("answer", PublicTypeTerm::I32),
        );

        assert_eq!(
            artifact.materializer_import_public_surface_mvp(Some(&NeplMetaImportClause::Open)),
            Err(
                super::NeplMetaMaterializerProjectionReject::UnsupportedReexportProjection
            )
        );
    }

    /// alias / glob / merge は依存側の visible name と衝突判定が必要である。MVP projection は
    /// surface を書き換えず、unsupported reason で fail-closed にする。
    #[test]
    fn neplmeta_projection_rejects_alias_glob_and_merge() {
        let artifact = artifact_for_materializer_with_names(
            module_surface_without_edges("/stdlib/core/math.nepl"),
            materializable_multi_surface_table(&["answer"]),
            &["answer"],
        );

        assert_eq!(
            artifact.materializer_import_public_surface_mvp(Some(&NeplMetaImportClause::Alias(
                "Math".into()
            ))),
            Err(super::NeplMetaMaterializerProjectionReject::UnsupportedAlias)
        );
        assert_eq!(
            artifact.materializer_import_public_surface_mvp(Some(&NeplMetaImportClause::Selective(
                Vec::from([NeplMetaImportItem {
                    name: "answer".into(),
                    alias: None,
                    glob: true,
                }]),
            ))),
            Err(super::NeplMetaMaterializerProjectionReject::UnsupportedGlob)
        );
        assert_eq!(
            artifact.materializer_import_public_surface_mvp(Some(&NeplMetaImportClause::Merge)),
            Err(super::NeplMetaMaterializerProjectionReject::UnsupportedMerge)
        );
    }

    #[test]
    fn neplmeta_store_projects_compatible_open_import_surface() {
        let artifact = artifact_for_materializer_with_names(
            module_surface_without_edges("/stdlib/core/math.nepl"),
            materializable_multi_surface_table(&["answer", "zero"]),
            &["answer", "zero"],
        );
        let expected_header = artifact.header();
        let mut store = NeplMetaArtifactStore::new();

        store.store(artifact).unwrap();
        let projected = store
            .materializer_import_public_surface_mvp(
                "/stdlib/core/math.nepl",
                expected_header,
                Some(&NeplMetaImportClause::Open),
            )
            .unwrap();

        let names = projected
            .entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, Vec::from(["answer", "zero"]));
        assert_eq!(store.stats().stores, 1);
        assert_eq!(store.stats().hits, 1);
        assert_eq!(store.stats().misses, 0);
    }

    /// body skip の入口では typed public signature をまだ再計算できない。
    /// store は pre-typecheck envelope だけで source identity と compile context を照合し、
    /// その後に payload consistency と projection を確認してから materializer 入力を返す。
    #[test]
    fn neplmeta_store_projects_with_pre_typecheck_envelope() {
        let module_surface = module_surface_without_edges("/stdlib/core/math.nepl");
        let artifact = artifact_for_materializer_with_names(
            module_surface.clone(),
            materializable_multi_surface_table(&["answer", "zero"]),
            &["answer", "zero"],
        );
        let mut source_map = SourceMap::new();
        source_map.add(
            "/stdlib/core/math.nepl",
            "pub fn answer %fn unit i32 \\:\n    1\n".into(),
        );
        let envelope = nepl_meta_artifact_pre_typecheck_envelope_for_module_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            None,
            Some(&source_map),
            &module_surface,
        )
        .expect("matching source should produce pre-typecheck envelope");
        let mut store = NeplMetaArtifactStore::new();

        store.store(artifact).unwrap();
        let projected = store
            .materializer_import_public_surface_pre_typecheck_mvp(
                "/stdlib/core/math.nepl",
                envelope,
                Some(&NeplMetaImportClause::Open),
            )
            .unwrap();

        let names = projected
            .entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, Vec::from(["answer", "zero"]));
        assert_eq!(store.stats().hits, 1);
        assert_eq!(store.stats().compatibility_rejects, 0);
        assert_eq!(store.stats().pre_typecheck_probe_attempts, 1);
        assert_eq!(store.stats().pre_typecheck_probe_projected, 1);
        assert_eq!(store.stats().pre_typecheck_probe_projected_entries, 2);
        assert_eq!(
            store.stats().last_pre_typecheck_probe_reject_kind,
            NeplMetaArtifactProbeRejectKind::None
        );
        assert_eq!(store.stats().last_pre_typecheck_probe_reject_code, 0);
        assert_eq!(store.stats().last_pre_typecheck_probe_projected_entries, 2);
    }

    /// pre-typecheck envelope は source key を含むため、public signature を再計算する前でも
    /// body token が違う stale artifact を拒否できる。
    #[test]
    fn neplmeta_store_pre_typecheck_envelope_rejects_stale_source_key() {
        let module_surface = module_surface_without_edges("/stdlib/core/math.nepl");
        let artifact = artifact_for_materializer(
            module_surface.clone(),
            materializable_surface_table("answer", PublicTypeTerm::I32),
        );
        let mut edited_source_map = SourceMap::new();
        edited_source_map.add(
            "/stdlib/core/math.nepl",
            "pub fn answer %fn unit i32 \\:\n    2\n".into(),
        );
        let edited_envelope = nepl_meta_artifact_pre_typecheck_envelope_for_module_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            None,
            Some(&edited_source_map),
            &module_surface,
        )
        .expect("edited source should still produce a pre-typecheck envelope");
        let mut store = NeplMetaArtifactStore::new();

        store.store(artifact).unwrap();
        assert_eq!(
            store.materializer_import_public_surface_pre_typecheck_mvp(
                "/stdlib/core/math.nepl",
                edited_envelope,
                Some(&NeplMetaImportClause::Open),
            ),
            Err(NeplMetaArtifactStoreReject::Compatibility(
                NeplMetaArtifactCompatibilityReject::SourceKey
            ))
        );
        assert_eq!(store.stats().hits, 1);
        assert_eq!(store.stats().compatibility_rejects, 1);
        assert_eq!(store.stats().pre_typecheck_probe_attempts, 1);
        assert_eq!(store.stats().pre_typecheck_probe_compatibility_rejects, 1);
        assert_eq!(
            store.stats().last_pre_typecheck_probe_reject_kind,
            NeplMetaArtifactProbeRejectKind::Compatibility
        );
        assert_eq!(
            store.stats().last_pre_typecheck_probe_reject_code,
            NeplMetaArtifactCompatibilityReject::SourceKey.code()
        );
    }

    /// pre-typecheck envelope は source key だけでなく dependency surface と module edge
    /// surface も見る。source が同じでも、依存候補や import/prelude edge が違えば
    /// materializer 入力の authority は別物として拒否する。
    #[test]
    fn neplmeta_store_pre_typecheck_envelope_rejects_dependency_and_module_surface_mismatch() {
        let stored_module_surface = module_surface_without_edges("/stdlib/core/math.nepl");
        let artifact = artifact_for_materializer(
            stored_module_surface.clone(),
            materializable_surface_table("answer", PublicTypeTerm::I32),
        );
        let mut source_map = SourceMap::new();
        source_map.add(
            "/stdlib/core/math.nepl",
            "pub fn answer %fn unit i32 \\:\n    1\n".into(),
        );
        let dependency_mismatch = nepl_meta_artifact_pre_typecheck_envelope_for_module_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            Some(99),
            Some(&source_map),
            &stored_module_surface,
        )
        .expect("matching source should produce envelope");
        let mut store = NeplMetaArtifactStore::new();
        store.store(artifact).unwrap();

        assert_eq!(
            store.materializer_import_public_surface_pre_typecheck_mvp(
                "/stdlib/core/math.nepl",
                dependency_mismatch,
                Some(&NeplMetaImportClause::Open),
            ),
            Err(NeplMetaArtifactStoreReject::Compatibility(
                NeplMetaArtifactCompatibilityReject::DependencyPublicSurface
            ))
        );

        let changed_module_surface = module_surface("/stdlib/core/math.nepl");
        let module_surface_mismatch =
            nepl_meta_artifact_pre_typecheck_envelope_for_module_surface(
                CompileTarget::Wasm,
                BuildProfile::Debug,
                Some(7),
                None,
                Some(&source_map),
                &changed_module_surface,
            )
            .expect("same source with different module surface should produce envelope");
        let artifact = artifact_for_materializer(
            stored_module_surface,
            materializable_surface_table("answer", PublicTypeTerm::I32),
        );
        let mut store = NeplMetaArtifactStore::new();
        store.store(artifact).unwrap();

        assert_eq!(
            store.materializer_import_public_surface_pre_typecheck_mvp(
                "/stdlib/core/math.nepl",
                module_surface_mismatch,
                Some(&NeplMetaImportClause::Open),
            ),
            Err(NeplMetaArtifactStoreReject::Compatibility(
                NeplMetaArtifactCompatibilityReject::ModuleSurface
            ))
        );
    }

    /// pre-typecheck probe の miss は通常 source fallback の理由であり、store 全体の miss と
    /// probe 専用の miss を両方記録する。
    #[test]
    fn neplmeta_store_pre_typecheck_probe_records_missing_artifact() {
        let module_surface = module_surface_without_edges("/stdlib/core/math.nepl");
        let mut source_map = SourceMap::new();
        source_map.add(
            "/stdlib/core/math.nepl",
            "pub fn answer %fn unit i32 \\:\n    1\n".into(),
        );
        let envelope = nepl_meta_artifact_pre_typecheck_envelope_for_module_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            None,
            Some(&source_map),
            &module_surface,
        )
        .expect("matching source should produce pre-typecheck envelope");
        let mut store = NeplMetaArtifactStore::new();

        assert_eq!(
            store.materializer_import_public_surface_pre_typecheck_mvp(
                "/stdlib/core/missing.nepl",
                envelope,
                Some(&NeplMetaImportClause::Open),
            ),
            Err(NeplMetaArtifactStoreReject::MissingArtifact {
                module_path: "/stdlib/core/missing.nepl".into(),
            })
        );
        assert_eq!(store.stats().misses, 1);
        assert_eq!(store.stats().pre_typecheck_probe_attempts, 1);
        assert_eq!(store.stats().pre_typecheck_probe_missing_artifacts, 1);
        assert_eq!(
            store.stats().last_pre_typecheck_probe_reject_kind,
            NeplMetaArtifactProbeRejectKind::MissingArtifact
        );
        assert_eq!(store.stats().last_pre_typecheck_probe_reject_code, 0);
    }

    /// pre-typecheck probe は未対応 import clause を body skip へ進めず、projection reject
    /// として reason code を残して通常 source fallback へ戻す。
    #[test]
    fn neplmeta_store_pre_typecheck_probe_records_projection_reject_reason() {
        let module_surface = module_surface_without_edges("/stdlib/core/math.nepl");
        let artifact = artifact_for_materializer(
            module_surface.clone(),
            materializable_surface_table("answer", PublicTypeTerm::I32),
        );
        let mut source_map = SourceMap::new();
        source_map.add(
            "/stdlib/core/math.nepl",
            "pub fn answer %fn unit i32 \\:\n    1\n".into(),
        );
        let envelope = nepl_meta_artifact_pre_typecheck_envelope_for_module_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            None,
            Some(&source_map),
            &module_surface,
        )
        .expect("matching source should produce pre-typecheck envelope");
        let mut store = NeplMetaArtifactStore::new();

        store.store(artifact).unwrap();
        assert_eq!(
            store.materializer_import_public_surface_pre_typecheck_mvp(
                "/stdlib/core/math.nepl",
                envelope,
                Some(&NeplMetaImportClause::Alias("math".into())),
            ),
            Err(NeplMetaArtifactStoreReject::Projection(
                NeplMetaMaterializerProjectionReject::UnsupportedAlias
            ))
        );
        assert_eq!(store.stats().hits, 1);
        assert_eq!(store.stats().projection_rejects, 1);
        assert_eq!(store.stats().pre_typecheck_probe_attempts, 1);
        assert_eq!(store.stats().pre_typecheck_probe_projection_rejects, 1);
        assert_eq!(
            store.stats().last_pre_typecheck_probe_reject_kind,
            NeplMetaArtifactProbeRejectKind::Projection
        );
        assert_eq!(
            store.stats().last_pre_typecheck_probe_reject_code,
            NeplMetaMaterializerProjectionReject::UnsupportedAlias.code()
        );
    }

    #[test]
    fn neplmeta_store_rejects_missing_artifact_and_stale_header() {
        let artifact = artifact_for_materializer(
            module_surface_without_edges("/stdlib/core/math.nepl"),
            materializable_surface_table("answer", PublicTypeTerm::I32),
        );
        let mut stale_header = artifact.header();
        stale_header.profile_hash ^= 1;
        let mut store = NeplMetaArtifactStore::new();

        assert_eq!(
            store.materializer_import_public_surface_mvp(
                "/stdlib/core/missing.nepl",
                artifact.header(),
                Some(&NeplMetaImportClause::Open),
            ),
            Err(NeplMetaArtifactStoreReject::MissingArtifact {
                module_path: "/stdlib/core/missing.nepl".into(),
            })
        );

        store.store(artifact).unwrap();
        assert_eq!(
            store.materializer_import_public_surface_mvp(
                "/stdlib/core/math.nepl",
                stale_header,
                Some(&NeplMetaImportClause::Open),
            ),
            Err(NeplMetaArtifactStoreReject::Compatibility(
                NeplMetaArtifactCompatibilityReject::Profile
            ))
        );
        assert_eq!(store.stats().misses, 1);
        assert_eq!(store.stats().compatibility_rejects, 1);
    }

    /// SourceMap や canonical path の不足で source key を作れない artifact は、
    /// typed public surface が整っていても materializer/store へ流さない。
    #[test]
    fn neplmeta_materializer_and_store_reject_missing_source_key() {
        let public_signatures = signature_table("answer", "fn unit i32");
        let public_surface = materializable_surface_table("answer", PublicTypeTerm::I32);
        let module_surface = module_surface_without_edges("/stdlib/core/math.nepl");
        let export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&module_surface, &public_surface);
        let header = nepl_meta_artifact_header_for_public_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            None,
            None,
            &public_signatures,
            Some(&module_surface),
            Some(&export_surface),
            &public_surface,
        );
        let artifact = NeplMetaArtifact::new(
            header,
            public_signatures,
            Some(module_surface),
            Some(export_surface),
            public_surface,
        );
        let mut store = NeplMetaArtifactStore::new();

        assert_eq!(
            artifact.materializer_mvp_reject(),
            Some(NeplMetaMaterializerMvpReject::MissingSourceKey)
        );
        assert_eq!(
            artifact.materializer_import_public_surface_mvp(Some(&NeplMetaImportClause::Open)),
            Err(NeplMetaMaterializerProjectionReject::MissingSourceKey)
        );
        assert_eq!(
            store.store(artifact),
            Err(NeplMetaArtifactStoreReject::MissingSourceKey)
        );
        assert_eq!(store.stats().store_rejects, 1);
    }

    #[test]
    fn neplmeta_store_rejects_payload_and_projection_without_mutating_surface() {
        let public_signatures = signature_table("answer", "fn unit i32");
        let public_surface = materializable_surface_table("answer", PublicTypeTerm::I32);
        let module_surface = module_surface_without_edges("/stdlib/core/math.nepl");
        let export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&module_surface, &public_surface);
        let mut stale_header = test_header(
            &public_signatures,
            Some(&module_surface),
            &public_surface,
            None,
        );
        stale_header.structured_public_surface_entry_count += 1;
        let stale_artifact = NeplMetaArtifact::new(
            stale_header,
            public_signatures,
            Some(module_surface),
            Some(export_surface),
            public_surface,
        );
        let mut store = NeplMetaArtifactStore::new();

        assert_eq!(
            store.store(stale_artifact),
            Err(NeplMetaArtifactStoreReject::PayloadConsistency(
                NeplMetaArtifactPayloadReject::StructuredPublicSurfaceEntryCount
            ))
        );
        assert_eq!(store.len(), 0);
        assert_eq!(store.stats().store_rejects, 1);
        assert_eq!(store.stats().payload_rejects, 1);

        let artifact = artifact_for_materializer(
            module_surface_without_edges("/stdlib/core/math.nepl"),
            materializable_surface_table("answer", PublicTypeTerm::I32),
        );
        let expected_header = artifact.header();
        store.store(artifact).unwrap();
        assert_eq!(
            store.materializer_import_public_surface_mvp(
                "/stdlib/core/math.nepl",
                expected_header,
                Some(&NeplMetaImportClause::Alias("math".into())),
            ),
            Err(NeplMetaArtifactStoreReject::Projection(
                NeplMetaMaterializerProjectionReject::UnsupportedAlias
            ))
        );
        assert_eq!(store.stats().projection_rejects, 1);
    }

    /// dependency aggregate public surface が違う場合、同じ module の public signature でも
    /// interface artifact は別 compile context として扱う。import 先の overload や trait impl が
    /// 変わると依存側の call resolution が変わり得るためである。
    #[test]
    fn neplmeta_header_rejects_dependency_surface_mismatch() {
        let public_signatures = signature_table("answer", "fn unit i32");
        let public_surface = surface_table("answer", PublicTypeTerm::I32);
        let module_surface = module_surface("/stdlib/core/math.nepl");
        let export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&module_surface, &public_surface);
        let artifact = NeplMetaArtifact::new(
            test_header(
                &public_signatures,
                Some(&module_surface),
                &public_surface,
                Some(11),
            ),
            public_signatures.clone(),
            Some(module_surface.clone()),
            Some(export_surface),
            public_surface.clone(),
        );
        let expected = test_header(
            &public_signatures,
            Some(&module_surface),
            &public_surface,
            Some(12),
        );

        assert_eq!(
            artifact.compatibility_reject(expected),
            Some(NeplMetaArtifactCompatibilityReject::DependencyPublicSurface)
        );
    }

    /// payload decode 後は header と payload の整合性も別に確認する。disk codec 実装時に
    /// stale header だけを信頼して、異なる public signature table を環境へ注入しないためである。
    #[test]
    fn neplmeta_payload_consistency_rejects_mismatched_signature_hash() {
        let header_signatures = signature_table("answer", "fn unit i32");
        let payload_signatures = signature_table("answer", "fn unit unit");
        let public_surface = surface_table("answer", PublicTypeTerm::I32);
        let artifact = NeplMetaArtifact::new(
            test_header(&header_signatures, None, &public_surface, None),
            payload_signatures,
            None,
            None,
            public_surface,
        );

        assert_eq!(
            artifact.payload_consistency_reject(),
            Some(NeplMetaArtifactPayloadReject::TypedPublicSignatureHash)
        );
    }

    #[test]
    fn neplmeta_payload_consistency_rejects_mismatched_structured_surface() {
        let public_signatures = signature_table("answer", "fn unit i32");
        let header_surface = surface_table("answer", PublicTypeTerm::I32);
        let payload_surface = surface_table("answer", PublicTypeTerm::Unit);
        let artifact = NeplMetaArtifact::new(
            test_header(&public_signatures, None, &header_surface, None),
            public_signatures,
            None,
            None,
            payload_surface,
        );

        assert_eq!(
            artifact.payload_consistency_reject(),
            Some(NeplMetaArtifactPayloadReject::StructuredPublicSurfaceHash)
        );
    }

    #[test]
    fn neplmeta_payload_consistency_rejects_mismatched_module_surface() {
        let public_signatures = signature_table("answer", "fn unit i32");
        let public_surface = surface_table("answer", PublicTypeTerm::I32);
        let header_surface = module_surface("/stdlib/core/math.nepl");
        let payload_surface = module_surface("/stdlib/core/other.nepl");
        let payload_export_surface =
            NeplMetaExportSurface::from_module_and_public_surface(&payload_surface, &public_surface);
        let artifact = NeplMetaArtifact::new(
            test_header(
                &public_signatures,
                Some(&header_surface),
                &public_surface,
                None,
            ),
            public_signatures,
            Some(payload_surface),
            Some(payload_export_surface),
            public_surface,
        );

        assert_eq!(
            artifact.payload_consistency_reject(),
            Some(NeplMetaArtifactPayloadReject::ModuleSurfaceHash)
        );
    }

    /// header と payload の export surface は structured public surface とは別に検証する。
    /// local export や re-export projection が stale になると、materializer が依存側環境へ
    /// 存在しない名前を注入し得るため、hash mismatch は専用理由で fail-closed にする。
    #[test]
    fn neplmeta_payload_consistency_rejects_mismatched_export_surface() {
        let public_signatures = signature_table("answer", "fn unit i32");
        let public_surface = surface_table("answer", PublicTypeTerm::I32);
        let module_surface = module_surface("/stdlib/core/math.nepl");
        let payload_export_surface = NeplMetaExportSurface::from_module_and_public_surface(
            &module_surface,
            &surface_table("other", PublicTypeTerm::I32),
        );
        let artifact = NeplMetaArtifact::new(
            test_header(
                &public_signatures,
                Some(&module_surface),
                &public_surface,
                None,
            ),
            public_signatures,
            Some(module_surface),
            Some(payload_export_surface),
            public_surface,
        );

        assert_eq!(
            artifact.payload_consistency_reject(),
            Some(NeplMetaArtifactPayloadReject::ExportSurfaceHash)
        );
    }

    /// materializer MVP gate は artifact 全体の fail-closed 条件を一か所で判定する。
    /// Open re-export と stable link symbol を持つ primitive callable だけなら、body skip
    /// の次段階へ進める候補として扱える。
    #[test]
    fn neplmeta_materializer_mvp_accepts_open_reexport_and_local_callable() {
        let artifact = artifact_for_materializer(
            module_surface("/stdlib/core/math.nepl"),
            materializable_surface_table("answer", PublicTypeTerm::I32),
        );

        assert_eq!(artifact.materializer_mvp_reject(), None);
    }

    /// stable link symbol のない callable は、fresh session の ABI symbol へ安全に
    /// 再投影できない。MVP gate は public surface preflight の blocker をそのまま
    /// reject reason として返す。
    #[test]
    fn neplmeta_materializer_mvp_rejects_public_surface_blocker() {
        let artifact = artifact_for_materializer(
            module_surface("/stdlib/core/math.nepl"),
            surface_table("answer", PublicTypeTerm::I32),
        );

        assert!(matches!(
            artifact.materializer_mvp_reject(),
            Some(NeplMetaMaterializerMvpReject::PublicSurfaceBlocker(_))
        ));
    }

    /// include は現行 loader では AST inline 境界であり、import / re-export と同じ
    /// materializer authority として扱えない。MVP では推測せず通常 load へ戻す。
    #[test]
    fn neplmeta_materializer_mvp_rejects_include_edge() {
        let artifact = artifact_for_materializer(
            module_surface_with_edges(
                "/stdlib/core/math.nepl",
                Vec::from([NeplMetaModuleDependencyEdge {
                    kind: NeplMetaModuleDependencyKind::Include,
                    target_path: "/stdlib/core/result.nepl".into(),
                    visibility: NeplMetaVisibility::Pub,
                    import_clause: None,
                    public_reexport: true,
                    source_order: 0,
                }]),
            ),
            materializable_surface_table("answer", PublicTypeTerm::I32),
        );

        assert_eq!(
            artifact.materializer_mvp_reject(),
            Some(NeplMetaMaterializerMvpReject::UnsupportedInclude)
        );
    }

    /// `.neplobj` direct call key は `.neplmeta` の stable link symbol だけではなく、
    /// selected callable body hash も含める。公開 signature が同じまま本文だけ変わる
    /// edit では `.neplmeta` を再利用できるが、codegen fragment は別物として扱う必要がある。
    #[test]
    fn neplobj_direct_call_key_tracks_selected_body_hash() {
        let link_symbol = callable_link_symbol("answer", 0x10);
        let first = neplobj_direct_call_key(
            link_symbol.clone(),
            0x20,
            0x30,
            super::nepl_obj_empty_generic_instantiation_hash(),
        );
        let body_edit = neplobj_direct_call_key(
            link_symbol,
            0x20,
            0x31,
            super::nepl_obj_empty_generic_instantiation_hash(),
        );

        assert_ne!(first.stable_hash(), body_edit.stable_hash());
        assert_ne!(
            first.selected_callable_body_hash,
            body_edit.selected_callable_body_hash
        );
    }

    /// generic callable の direct call fragment は、同じ関数本文でも具体化型ごとに
    /// backend lowering や Resource proof が変わり得る。generic instantiation hash を
    /// key に含めることで、`Vec i32` と `Vec u8` のような別具体化を同じ object として
    /// 誤再利用しない。
    #[test]
    fn neplobj_direct_call_key_tracks_generic_instantiation_hash() {
        let link_symbol = callable_link_symbol("id", 0x10);
        let i32_instantiation = neplobj_direct_call_key(link_symbol.clone(), 0x20, 0x30, 0x40);
        let u8_instantiation = neplobj_direct_call_key(link_symbol, 0x20, 0x30, 0x41);

        assert_ne!(
            i32_instantiation.stable_hash(),
            u8_instantiation.stable_hash()
        );
        assert_ne!(
            i32_instantiation.generic_instantiation_hash,
            u8_instantiation.generic_instantiation_hash
        );
    }

    /// materialized symbol は direct call の HIR に現れる `neplmeta$...` 名と同じ規則で
    /// 作る。`.neplobj` key はこの symbol を保持するが、symbol 文字列だけを key にせず、
    /// body hash や source capability policy と組み合わせる。
    #[test]
    fn neplobj_direct_call_key_keeps_materialized_symbol() {
        let key = neplobj_direct_call_key(
            callable_link_symbol("read-value", 0x10),
            0x20,
            0x30,
            super::nepl_obj_empty_generic_instantiation_hash(),
        );

        assert!(key.materialized_symbol.starts_with("neplmeta$read_value$"));
        assert_ne!(key.link_symbol_hash, 0);
    }

    /// `.neplobj` fragment は key だけでなく backend payload を hash する。body bytes や
    /// result 型が変わった場合、同じ direct-call key に見えても codegen 再利用単位は別物である。
    #[test]
    fn neplobj_direct_call_fragment_tracks_backend_payload() {
        let key = neplobj_direct_call_key(
            callable_link_symbol("answer", 42),
            0x20,
            0x30,
            super::nepl_obj_empty_generic_instantiation_hash(),
        );
        let first = neplobj_direct_call_fragment(
            key.clone(),
            Vec::from([NeplObjWasmValType::I32]),
            Vec::from([0x00, 0x41, 0x01, 0x0b]),
            Vec::new(),
        );
        let body_edit = neplobj_direct_call_fragment(
            key.clone(),
            Vec::from([NeplObjWasmValType::I32]),
            Vec::from([0x00, 0x41, 0x02, 0x0b]),
            Vec::new(),
        );
        let signature_edit = neplobj_direct_call_fragment(
            key,
            Vec::from([NeplObjWasmValType::I64]),
            Vec::from([0x00, 0x41, 0x01, 0x0b]),
            Vec::new(),
        );

        assert_ne!(first.stable_hash(), body_edit.stable_hash());
        assert_ne!(first.stable_hash(), signature_edit.stable_hash());
    }

    /// direct call relocation は final module assembly で function index を埋めるための
    /// payload である。入力順に依存して cache key が揺れると同じ body を重複保存するため、
    /// fragment 作成時点で安定順へ正規化する。
    #[test]
    fn neplobj_direct_call_fragment_normalizes_relocations() {
        let key = neplobj_direct_call_key(
            callable_link_symbol("answer", 0x10),
            0x20,
            0x30,
            super::nepl_obj_empty_generic_instantiation_hash(),
        );
        let target_a = callable_link_symbol("callee_a", 0x21);
        let target_b = callable_link_symbol("callee_b", 0x22);
        let relocation_a = NeplObjWasmDirectCallRelocation {
            byte_offset: 4,
            target: target_a,
        };
        let relocation_b = NeplObjWasmDirectCallRelocation {
            byte_offset: 2,
            target: target_b,
        };
        let first = neplobj_direct_call_fragment(
            key.clone(),
            Vec::from([NeplObjWasmValType::I32]),
            Vec::from([0x00, 0x10, 0x00, 0x10, 0x01, 0x0b]),
            Vec::from([relocation_a.clone(), relocation_b.clone()]),
        );
        let reordered = neplobj_direct_call_fragment(
            key,
            Vec::from([NeplObjWasmValType::I32]),
            Vec::from([0x00, 0x10, 0x00, 0x10, 0x01, 0x0b]),
            Vec::from([relocation_b, relocation_a]),
        );

        assert_eq!(first.stable_hash(), reordered.stable_hash());
        match first.backend() {
            NeplObjDirectCallBackendFragment::Wasm(fragment) => {
                assert_eq!(fragment.direct_call_relocations()[0].byte_offset, 2);
                assert_eq!(fragment.direct_call_relocations()[1].byte_offset, 4);
            }
        }
    }

    /// relocation offset が wasm body の外を指す artifact は、linker へ届く前に拒否する。
    /// cache 由来の payload は信頼済み source ではなく、明白な不整合を artifact boundary で
    /// fail-closed に落とす必要がある。
    #[test]
    fn neplobj_direct_call_fragment_rejects_out_of_range_relocation() {
        let result = NeplObjWasmDirectCallFragment::new(
            Vec::new(),
            Vec::from([NeplObjWasmValType::I32]),
            Vec::from([0x00, 0x10, 0x00, 0x0b]),
            Vec::from([NeplObjWasmDirectCallRelocation {
                byte_offset: 4,
                target: callable_link_symbol("callee", 0x21),
            }]),
        );

        assert_eq!(
            result,
            Err(NeplObjDirectCallFragmentReject::RelocationOffsetOutOfRange {
                byte_offset: 4,
                body_len: 4,
            })
        );
    }

    /// 同じ byte offset を複数 relocation が所有すると、final module assembly でどちらの
    /// symbol を書くべきか曖昧になる。offset 重複は payload 作成時点で拒否する。
    #[test]
    fn neplobj_direct_call_fragment_rejects_duplicate_relocation_offset() {
        let result = NeplObjWasmDirectCallFragment::new(
            Vec::new(),
            Vec::from([NeplObjWasmValType::I32]),
            Vec::from([0x00, 0x10, 0x00, 0x0b]),
            Vec::from([
                NeplObjWasmDirectCallRelocation {
                    byte_offset: 2,
                    target: callable_link_symbol("callee_a", 0x21),
                },
                NeplObjWasmDirectCallRelocation {
                    byte_offset: 2,
                    target: callable_link_symbol("callee_b", 0x22),
                },
            ]),
        );

        assert_eq!(
            result,
            Err(NeplObjDirectCallFragmentReject::DuplicateRelocationOffset { byte_offset: 2 })
        );
    }

    /// availability 判定は direct-call key の一致を要求する。materialized symbol が同じでも
    /// body hash などが異なる fragment は、別の payload として扱い source fallback を残す。
    #[test]
    fn neplobj_direct_call_fragment_matches_complete_key() {
        let key = neplobj_direct_call_key(
            callable_link_symbol("answer", 0x10),
            0x20,
            0x30,
            super::nepl_obj_empty_generic_instantiation_hash(),
        );
        let body_edit_key = neplobj_direct_call_key(
            callable_link_symbol("answer", 0x10),
            0x20,
            0x31,
            super::nepl_obj_empty_generic_instantiation_hash(),
        );
        let fragment = neplobj_direct_call_fragment(
            key.clone(),
            Vec::from([NeplObjWasmValType::I32]),
            Vec::from([0x00, 0x0b]),
            Vec::new(),
        );

        assert!(fragment.matches_direct_call_key(&key));
        assert!(!fragment.matches_direct_call_key(&body_edit_key));
    }

    /// same-session `.neplobj` store は materialized symbol だけでなく、target/profile、
    /// source key、dependency surface、source capability policy まで一致した fragment だけを
    /// 返す。これにより `.neplmeta` が同じ callable を復元しても、別 source/policy の body を
    /// 誤って body-missing diagnostic の代替にしない。
    #[test]
    fn neplobj_direct_call_fragment_store_matches_lookup_context() {
        let key = neplobj_direct_call_key(
            callable_link_symbol("answer", 42),
            0x20,
            0x30,
            super::nepl_obj_empty_generic_instantiation_hash(),
        );
        let fragment = neplobj_direct_call_fragment(
            key,
            Vec::from([NeplObjWasmValType::I32]),
            Vec::from([0x00, 0x41, 0x07, 0x0b]),
            Vec::new(),
        );
        let surface = materializable_surface_table("answer", PublicTypeTerm::I32);
        let mut store = NeplObjDirectCallFragmentStore::new();
        store.store(fragment.clone()).unwrap();

        let found = store.matching_direct_call_fragments_for_probes(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(0x5a),
            &[NeplObjDirectCallFragmentLookupProbe {
                module_path: "/stdlib/core/math.nepl",
                target_source_key_hash: 0x20,
                source_capability_policy_hash: Some(0x80),
                dependency_public_surface_hash: 0x70,
                surface: &surface,
            }],
        );

        assert_eq!(found, Vec::from([fragment]));
        let stats = store.stats();
        assert_eq!(stats.lookup_hits, 1);
        assert_eq!(stats.lookup_fragments_returned, 1);
    }

    /// body-missing negative cache は、同じ edge context の object fragment が存在する場合には
    /// materialized probe を省略してはいけない。store は public surface を読む前でも使える
    /// context-level 判定を提供し、将来 fragment producer が追加された時に古い negative cache が
    /// object hit を隠さないようにする。
    #[test]
    fn neplobj_direct_call_fragment_store_keeps_negative_skip_open_for_candidate_context() {
        let key = neplobj_direct_call_key(
            callable_link_symbol("answer", 0x10),
            0x20,
            0x30,
            super::nepl_obj_empty_generic_instantiation_hash(),
        );
        let fragment = neplobj_direct_call_fragment(
            key,
            Vec::from([NeplObjWasmValType::I32]),
            Vec::from([0x00, 0x41, 0x07, 0x0b]),
            Vec::new(),
        );
        let mut store = NeplObjDirectCallFragmentStore::new();
        store.store(fragment).unwrap();

        assert!(store.may_have_fragment_for_probe_context(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(0x5a),
            0x20,
            Some(0x80),
            0x70,
        ));
        assert!(!store.may_have_fragment_for_probe_context(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(0x5a),
            0x21,
            Some(0x80),
            0x70,
        ));
        assert!(!store.may_have_fragment_for_probe_context(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(0x5a),
            0x20,
            None,
            0x70,
        ));
    }

    /// 同じ `.neplobj` direct-call key に異なる backend payload が保存される場合は、
    /// 上書きせず reject する。完全 key が一致しているなら backend lowering は決定的に
    /// 同じ payload になるべきであり、ここを許すと session 内で object authority が揺れる。
    #[test]
    fn neplobj_direct_call_fragment_store_rejects_conflicting_payload() {
        let key = neplobj_direct_call_key(
            callable_link_symbol("answer", 0x10),
            0x20,
            0x30,
            super::nepl_obj_empty_generic_instantiation_hash(),
        );
        let first = neplobj_direct_call_fragment(
            key.clone(),
            Vec::from([NeplObjWasmValType::I32]),
            Vec::from([0x00, 0x41, 0x07, 0x0b]),
            Vec::new(),
        );
        let conflict = neplobj_direct_call_fragment(
            key.clone(),
            Vec::from([NeplObjWasmValType::I32]),
            Vec::from([0x00, 0x41, 0x08, 0x0b]),
            Vec::new(),
        );
        let mut store = NeplObjDirectCallFragmentStore::new();

        store.store(first).unwrap();
        assert_eq!(
            store.store(conflict),
            Err(
                NeplObjDirectCallFragmentStoreReject::BackendPayloadConflict {
                    key_stable_hash: key.stable_hash(),
                }
            )
        );
        assert_eq!(store.stats().store_rejects, 1);
    }

    /// merge / alias / glob は target export artifact を読んだうえで衝突や曖昧性を
    /// 判定する必要がある。MVP gate はこれらを個別 reason で fail-closed にする。
    #[test]
    fn neplmeta_materializer_mvp_rejects_unsupported_import_projection() {
        let alias_artifact = artifact_for_materializer(
            module_surface_with_edges(
                "/stdlib/core/math.nepl",
                Vec::from([NeplMetaModuleDependencyEdge {
                    kind: NeplMetaModuleDependencyKind::Import,
                    target_path: "/stdlib/core/result.nepl".into(),
                    visibility: NeplMetaVisibility::Pub,
                    import_clause: Some(NeplMetaImportClause::Alias("ResultAlias".into())),
                    public_reexport: true,
                    source_order: 0,
                }]),
            ),
            materializable_surface_table("answer", PublicTypeTerm::I32),
        );
        let glob_artifact = artifact_for_materializer(
            module_surface_with_edges(
                "/stdlib/core/math.nepl",
                Vec::from([NeplMetaModuleDependencyEdge {
                    kind: NeplMetaModuleDependencyKind::Import,
                    target_path: "/stdlib/core/result.nepl".into(),
                    visibility: NeplMetaVisibility::Pub,
                    import_clause: Some(NeplMetaImportClause::Selective(Vec::from([
                        NeplMetaImportItem {
                            name: "Result".into(),
                            alias: None,
                            glob: true,
                        },
                    ]))),
                    public_reexport: true,
                    source_order: 0,
                }]),
            ),
            materializable_surface_table("answer", PublicTypeTerm::I32),
        );

        assert_eq!(
            alias_artifact.materializer_mvp_reject(),
            Some(NeplMetaMaterializerMvpReject::UnsupportedAlias)
        );
        assert_eq!(
            glob_artifact.materializer_mvp_reject(),
            Some(NeplMetaMaterializerMvpReject::UnsupportedGlob)
        );
    }
}
