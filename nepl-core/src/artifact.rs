extern crate alloc;

use crate::compiler::{BuildProfile, CompileTarget};
use crate::source_map::SourceMap;
use crate::typecheck::{
    PublicSurfaceMaterializerBlockerReason, TypedPublicSignatureKind,
    TypedPublicSignatureTable, TypedPublicSurfaceTable,
};
use alloc::string::String;
use alloc::vec::Vec;

const FNV1A64_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x100000001b3;
const NEPL_META_ARTIFACT_SCHEMA_VERSION: u32 = 10;
const NEPL_META_ARTIFACT_HASH_VERSION: &str = "neplg2-neplmeta-artifact-v10";
const NEPL_META_COMPILER_IDENTITY_INPUT: &str = concat!(
    "neplg2-compiler:",
    env!("CARGO_PKG_VERSION"),
    ":neplmeta-v10"
);

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

impl NeplMetaArtifactHeader {
    pub fn new(
        compiler_identity_hash: u64,
        target_hash: u64,
        profile_hash: u64,
        stdlib_content_hash: Option<u64>,
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
            return Some(NeplMetaMaterializerMvpReject::PublicSurfaceBlocker(
                blocker.reason,
            ));
        }
        if self
            .public_surface
            .entries
            .iter()
            .any(|entry| entry.kind == TypedPublicSignatureKind::Impl)
        {
            return Some(NeplMetaMaterializerMvpReject::UnsupportedImplLookup);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeplMetaMaterializerMvpReject {
    PayloadConsistency(NeplMetaArtifactPayloadReject),
    MissingModuleSurface,
    MissingExportSurface,
    MissingModuleIdentity,
    MissingReexportTarget,
    PublicSurfaceBlocker(PublicSurfaceMaterializerBlockerReason),
    UnsupportedInclude,
    UnsupportedMerge,
    UnsupportedAlias,
    UnsupportedGlob,
    UnsupportedImplLookup,
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
        nepl_meta_artifact_header_for_public_surface, NeplMetaArtifact,
        NeplMetaArtifactCompatibilityReject, NeplMetaArtifactHeader, NeplMetaArtifactPayloadReject,
        NeplMetaExportKind, NeplMetaExportSurface, NeplMetaImportClause, NeplMetaImportItem,
        NeplMetaMaterializerMvpReject, NeplMetaModuleDependencyEdge,
        NeplMetaModuleDependencyKind, NeplMetaModuleSurface, NeplMetaVisibility,
    };
    use crate::compiler::{BuildProfile, CompileTarget};
    use crate::typecheck::{
        PublicCallableLinkSymbol, PublicCallableSurface, PublicEffect, PublicSurfaceShape,
        PublicTypeTerm, TypedPublicSignatureEntry, TypedPublicSignatureKind,
        TypedPublicSignatureTable, TypedPublicSurfaceEntry, TypedPublicSurfaceTable,
    };

    fn signature_table(name: &str, signature: &str) -> TypedPublicSignatureTable {
        TypedPublicSignatureTable::new(Vec::from([TypedPublicSignatureEntry::new(
            TypedPublicSignatureKind::Callable,
            name.into(),
            signature.into(),
            false,
        )]))
    }

    fn surface_table(name: &str, result: PublicTypeTerm) -> TypedPublicSurfaceTable {
        TypedPublicSurfaceTable::new(Vec::from([TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Callable,
            name: name.into(),
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

    fn test_header(
        public_signatures: &TypedPublicSignatureTable,
        module_surface: Option<&NeplMetaModuleSurface>,
        public_surface: &TypedPublicSurfaceTable,
        dependency_hash: Option<u64>,
    ) -> NeplMetaArtifactHeader {
        let export_surface = module_surface.map(|surface| {
            NeplMetaExportSurface::from_module_and_public_surface(surface, public_surface)
        });
        nepl_meta_artifact_header_for_public_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            dependency_hash,
            None,
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
