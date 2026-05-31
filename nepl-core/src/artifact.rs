extern crate alloc;

use crate::compiler::{BuildProfile, CompileTarget};
use crate::source_map::SourceMap;
use crate::typecheck::{TypedPublicSignatureTable, TypedPublicSurfaceTable};

const FNV1A64_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x100000001b3;
const NEPL_META_ARTIFACT_SCHEMA_VERSION: u32 = 4;
const NEPL_META_ARTIFACT_HASH_VERSION: &str = "neplg2-neplmeta-artifact-v4";
const NEPL_META_COMPILER_IDENTITY_INPUT: &str = concat!(
    "neplg2-compiler:",
    env!("CARGO_PKG_VERSION"),
    ":neplmeta-v4"
);

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
    public_surface: TypedPublicSurfaceTable,
}

impl NeplMetaArtifact {
    pub fn new(
        header: NeplMetaArtifactHeader,
        public_signatures: TypedPublicSignatureTable,
        public_surface: TypedPublicSurfaceTable,
    ) -> Self {
        Self {
            header,
            public_signatures,
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
        let header = nepl_meta_artifact_header_for_public_surface(
            target,
            profile,
            stdlib_content_hash,
            dependency_public_surface_hash,
            source_map,
            &public_signatures,
            &public_surface,
        );
        Self::new(header, public_signatures, public_surface)
    }

    pub fn header(&self) -> NeplMetaArtifactHeader {
        self.header
    }

    pub fn public_signatures(&self) -> &TypedPublicSignatureTable {
        &self.public_signatures
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
        public_surface.stable_hash,
        usize_to_u32_saturating(public_surface.entries.len()),
        crate::compiler::resource_summary_source_capability_policy_set_hash(source_map),
        Some(crate::compiler::resource_summary_private_effect_policy_hash()),
    )
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
                type_param_bounds: Vec::new(),
            }),
        }]))
    }

    fn test_header(
        public_signatures: &TypedPublicSignatureTable,
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
        let header = test_header(&public_signatures, &public_surface, Some(11));
        let artifact = NeplMetaArtifact::new(header, public_signatures, public_surface);

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
        let artifact = NeplMetaArtifact::new(
            test_header(&public_signatures, &public_surface, Some(11)),
            public_signatures.clone(),
            public_surface.clone(),
        );
        let expected = test_header(&public_signatures, &public_surface, Some(12));

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
            test_header(&header_signatures, &public_surface, None),
            payload_signatures,
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
            test_header(&public_signatures, &header_surface, None),
            public_signatures,
            payload_surface,
        );

        assert_eq!(
            artifact.payload_consistency_reject(),
            Some(NeplMetaArtifactPayloadReject::StructuredPublicSurfaceHash)
        );
    }
}
