extern crate alloc;

use crate::compiler::{BuildProfile, CompileTarget};
use crate::source_map::SourceMap;
use crate::typecheck::{TypedPublicSignatureTable, TypedPublicSurfaceTable};
use alloc::string::String;
use alloc::vec::Vec;

const FNV1A64_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x100000001b3;
const NEPL_META_ARTIFACT_SCHEMA_VERSION: u32 = 9;
const NEPL_META_ARTIFACT_HASH_VERSION: &str = "neplg2-neplmeta-artifact-v9";
const NEPL_META_COMPILER_IDENTITY_INPUT: &str = concat!(
    "neplg2-compiler:",
    env!("CARGO_PKG_VERSION"),
    ":neplmeta-v9"
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
    public_surface: TypedPublicSurfaceTable,
}

impl NeplMetaArtifact {
    pub fn new(
        header: NeplMetaArtifactHeader,
        public_signatures: TypedPublicSignatureTable,
        module_surface: Option<NeplMetaModuleSurface>,
        public_surface: TypedPublicSurfaceTable,
    ) -> Self {
        Self {
            header,
            public_signatures,
            module_surface,
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
        let header = nepl_meta_artifact_header_for_public_surface(
            target,
            profile,
            stdlib_content_hash,
            dependency_public_surface_hash,
            source_map,
            &public_signatures,
            module_surface.as_ref(),
            &public_surface,
        );
        Self::new(header, public_signatures, module_surface, public_surface)
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

    pub fn public_surface(&self) -> &TypedPublicSurfaceTable {
        &self.public_surface
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
    StructuredPublicSurfaceHash,
    StructuredPublicSurfaceEntryCount,
}

pub fn nepl_meta_artifact_header_for_public_surface(
    target: CompileTarget,
    profile: BuildProfile,
    stdlib_content_hash: Option<u64>,
    dependency_public_surface_hash: Option<u64>,
    source_map: Option<&SourceMap>,
    public_signatures: &TypedPublicSignatureTable,
    module_surface: Option<&NeplMetaModuleSurface>,
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
        NeplMetaImportClause, NeplMetaModuleDependencyEdge, NeplMetaModuleDependencyKind,
        NeplMetaModuleSurface, NeplMetaVisibility,
    };
    use crate::compiler::{BuildProfile, CompileTarget};
    use crate::typecheck::{
        PublicCallableSurface, PublicEffect, PublicSurfaceShape, PublicTypeTerm,
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

    fn test_header(
        public_signatures: &TypedPublicSignatureTable,
        module_surface: Option<&NeplMetaModuleSurface>,
        public_surface: &TypedPublicSurfaceTable,
        dependency_hash: Option<u64>,
    ) -> NeplMetaArtifactHeader {
        nepl_meta_artifact_header_for_public_surface(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(7),
            dependency_hash,
            None,
            public_signatures,
            module_surface,
            public_surface,
        )
    }

    fn module_surface(path: &str) -> NeplMetaModuleSurface {
        NeplMetaModuleSurface::new(
            path.into(),
            "/stdlib/std/prelude_base.nepl".into(),
            false,
            true,
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

    /// `.neplmeta` header は public signature hash を invalidation boundary にする。
    /// function body や typed HIR を payload に入れず、依存側 typecheck に必要な公開面だけを
    /// stable value として運ぶ前提を固定する。
    #[test]
    fn neplmeta_header_accepts_matching_public_surface() {
        let public_signatures = signature_table("answer", "fn unit i32");
        let public_surface = surface_table("answer", PublicTypeTerm::I32);
        let module_surface = module_surface("/stdlib/core/math.nepl");
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
            public_surface,
        );

        assert_eq!(artifact.compatibility_reject(header), None);
        assert_eq!(artifact.payload_consistency_reject(), None);
        assert_eq!(artifact.entry_count(), 1);
    }

    /// dependency aggregate public surface が違う場合、同じ module の public signature でも
    /// interface artifact は別 compile context として扱う。import 先の overload や trait impl が
    /// 変わると依存側の call resolution が変わり得るためである。
    #[test]
    fn neplmeta_header_rejects_dependency_surface_mismatch() {
        let public_signatures = signature_table("answer", "fn unit i32");
        let public_surface = surface_table("answer", PublicTypeTerm::I32);
        let module_surface = module_surface("/stdlib/core/math.nepl");
        let artifact = NeplMetaArtifact::new(
            test_header(
                &public_signatures,
                Some(&module_surface),
                &public_surface,
                Some(11),
            ),
            public_signatures.clone(),
            Some(module_surface.clone()),
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
        let artifact = NeplMetaArtifact::new(
            test_header(
                &public_signatures,
                Some(&header_surface),
                &public_surface,
                None,
            ),
            public_signatures,
            Some(payload_surface),
            public_surface,
        );

        assert_eq!(
            artifact.payload_consistency_reject(),
            Some(NeplMetaArtifactPayloadReject::ModuleSurfaceHash)
        );
    }
}
