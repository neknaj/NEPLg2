extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Effect, Visibility};
use crate::backend_scalar_type::BackendScalarType;
use crate::source_map::SourceMap;
use crate::types::{NominalStableTypeKind, TypeCtx, TypeId, TypeKind};

use super::env::{Binding, BindingKind, Env};
use super::model::{EnumInfo, RestrictedStructConstructor, StructConstructorPolicy, StructInfo};
use super::public_signature::TypedPublicSignatureKind;
use super::signature::signature_type_string;
use super::traits::{BoundEnv, ImplInfo, ImplKind, TraitApplication, TraitCapability, TraitInfo};
use super::FieldAccessorKind;

const FNV1A64_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x100000001b3;

/// Arena-independent semantic surface used by `.neplmeta`.
///
/// This table keeps exported public entries and the dependency-local semantic
/// entries that those exports need for typechecking, such as private capability
/// traits referenced by impl headers. A later materializer can project the table
/// into a fresh `TypeCtx` and `Env`. The table intentionally avoids `TypeId`,
/// `Span`, `SourceMap`, HIR, Resource IR, and diagnostics. Those values belong
/// to one compiler session and must not become persistent artifact authority.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypedPublicSurfaceTable {
    pub entries: Vec<TypedPublicSurfaceEntry>,
    pub stable_hash: u64,
}

impl TypedPublicSurfaceTable {
    pub fn new(mut entries: Vec<TypedPublicSurfaceEntry>) -> Self {
        entries.sort();
        entries.dedup();
        let stable_hash = typed_public_surface_hash(&entries);
        Self {
            entries,
            stable_hash,
        }
    }

    /// `.neplmeta` materializer が安全に使えない entry を列挙する。
    ///
    /// この関数は materializer そのものではない。artifact の structured public surface を
    /// 現在 compile の `TypeCtx` / `Env` へ投影する前に、名前だけの nominal type や
    /// 対応 binder のない generic parameter を検出し、dependency body skip を fail-closed
    /// に止めるための preflight である。
    pub fn materializer_blockers(&self) -> Vec<PublicSurfaceMaterializerBlocker> {
        let mut blockers = Vec::new();
        for entry in &self.entries {
            collect_surface_shape_materializer_blockers(entry, &entry.surface, &mut blockers);
        }
        blockers
    }

    /// structured public surface が materializer preflight を通過できるかを返す。
    ///
    /// `true` は「この table だけで body skip してよい」という意味ではない。module graph、
    /// import visibility、ABI symbol、trait/impl lookup、artifact header の整合性など、
    /// materializer 全体の他の条件は別途検査する。
    pub fn is_materializer_preflight_ready(&self) -> bool {
        self.materializer_blockers().is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicSurfaceMaterializerBlocker {
    pub entry_kind: TypedPublicSignatureKind,
    pub entry_name: String,
    pub reason: PublicSurfaceMaterializerBlockerReason,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicSurfaceMaterializerBlockerReason {
    /// public struct surface 自体に stable nominal identity がない。
    MissingStructIdentity,
    /// public enum surface 自体に stable nominal identity がない。
    MissingEnumIdentity,
    /// named type term が name だけを持ち、module/source identity を持たない。
    MissingNamedTypeIdentity { type_name: String },
    /// public callable が stable link symbol を持たない。
    MissingCallableLinkSymbol { callable_name: String },
    /// generic parameter term を binder depth / index へ対応付けられなかった。
    UnboundGenericParam { param_name: String },
    /// trait bound target を binder depth / index へ対応付けられなかった。
    UnboundTraitBoundTarget { param_name: String },
    /// trait surface または trait reference が stable trait identity を持たない。
    MissingTraitIdentity { trait_name: String },
    /// trait-local `Self` term が trait method surface の外に現れている。
    TraitSelfOutsideTraitMethod,
}

impl PublicSurfaceMaterializerBlockerReason {
    /// Web playground や regression test が文字列解析なしに blocker を分類するための
    /// stable code。
    ///
    /// この code は user diagnostic の表示文ではなく、`.neplmeta` materializer が
    /// fail-closed で source fallback へ戻った理由を性能調査用に集計する境界である。
    pub fn code(&self) -> u32 {
        match self {
            Self::MissingStructIdentity => 1,
            Self::MissingEnumIdentity => 2,
            Self::MissingNamedTypeIdentity { .. } => 3,
            Self::MissingCallableLinkSymbol { .. } => 4,
            Self::UnboundGenericParam { .. } => 5,
            Self::UnboundTraitBoundTarget { .. } => 6,
            Self::MissingTraitIdentity { .. } => 7,
            Self::TraitSelfOutsideTraitMethod => 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypedPublicSurfaceEntry {
    pub kind: TypedPublicSignatureKind,
    pub name: String,
    /// import 先の visible namespace へ公開できる entry かを示す。
    ///
    /// `false` の entry は private capability trait などの semantic support surface である。
    /// dependency typecheck の復元には必要だが、`.neplmeta` export surface へ出してはならない。
    pub exported: bool,
    pub surface: PublicSurfaceShape,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicSurfaceShape {
    Callable(PublicCallableSurface),
    Struct(PublicStructSurface),
    Enum(PublicEnumSurface),
    Trait(PublicTraitSurface),
    Impl(PublicImplSurface),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicCallableSurface {
    pub ty: PublicTypeTerm,
    pub no_shadow: bool,
    pub arity: u32,
    pub effect: PublicEffect,
    pub field_accessor: Option<PublicFieldAccessorKind>,
    pub link_symbol: Option<PublicCallableLinkSymbol>,
    pub type_param_bounds: Vec<PublicTypeParamBounds>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicCallableLinkSymbol {
    pub source_path: String,
    pub name: String,
    pub signature_hash: u64,
}

/// `.neplmeta` から復元した callable が current session の ABI namespace で使う symbol。
///
/// この symbol は user source の関数名ではなく、stable link symbol を current compiler
/// session の HIR / Resource IR / codegen が扱える形へ写したものである。source path と
/// signature hash を含めることで、同名 callable や overload を同じ `neplmeta$...` 名へ
/// 潰さない。関数本体や generic 具体化は含まないため、`.neplobj` の codegen fragment key
/// ではこの symbol に加えて body hash と generic instantiation hash を必ず組み合わせる。
pub fn materialized_callable_symbol_for_link_symbol(symbol: &PublicCallableLinkSymbol) -> String {
    format!(
        "neplmeta${}${:016x}${:016x}",
        stable_symbol_component(&symbol.name),
        fnv1a64(symbol.source_path.as_str()),
        symbol.signature_hash
    )
}

/// public callable link symbol を artifact key 用の安定 hash へ落とす。
///
/// `PublicCallableLinkSymbol` は source path、公開名、signature hash で構成される。
/// ここではそれらを `.neplobj` / `.neplmeta` 間で共有できる小さな hash にまとめる。
/// body hash ではないため、body-only edit の invalidation には別の selected body hash を
/// key に入れる必要がある。
pub fn public_callable_link_symbol_stable_hash(symbol: &PublicCallableLinkSymbol) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    hash_str(&mut hash, "neplg2-public-callable-link-symbol-v1");
    hash_str(&mut hash, symbol.source_path.as_str());
    hash_str(&mut hash, symbol.name.as_str());
    hash_u64(&mut hash, symbol.signature_hash);
    hash
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicTypeParamBounds {
    pub param: PublicTypeParamBoundTarget,
    pub bounds: Vec<PublicTraitRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicStructSurface {
    pub identity: Option<PublicNominalTypeIdentity>,
    pub type_params: Vec<PublicTypeParam>,
    pub fields: Vec<PublicFieldSurface>,
    pub constructor_policy: PublicStructConstructorPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicFieldSurface {
    pub name: String,
    pub ty: PublicTypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicEnumSurface {
    pub identity: Option<PublicNominalTypeIdentity>,
    pub type_params: Vec<PublicTypeParam>,
    pub variants: Vec<PublicEnumVariantSurface>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicEnumVariantSurface {
    pub name: String,
    pub payload: Option<PublicTypeTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicTraitSurface {
    pub identity: Option<PublicTraitIdentity>,
    pub type_params: Vec<PublicTypeParam>,
    pub capabilities: Vec<PublicTraitCapability>,
    pub methods: Vec<PublicTraitMethodSurface>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicTraitMethodSurface {
    pub name: String,
    pub ty: PublicTypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicImplSurface {
    pub type_params: Vec<PublicTypeParam>,
    pub type_param_bounds: Vec<PublicTypeParamBounds>,
    pub kind: PublicImplKind,
    pub target: PublicTypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicImplKind {
    Inherent,
    Trait { application: PublicTraitRef },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicTraitRef {
    pub name: String,
    pub identity: Option<PublicTraitIdentity>,
    pub args: Vec<PublicTypeTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicTraitIdentity {
    pub source_path: String,
    pub name: String,
    pub arity: u32,
    pub definition_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicTypeParam {
    pub name: String,
    pub copy_cap: bool,
    pub clone_cap: bool,
    pub drop_cap: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicTypeParamRef {
    /// 現在の type expression から見た generic binder の深さ。
    ///
    /// `0` は最も内側の binder を指す。generic function type の内側に入った場合、
    /// その function type が type parameter を導入すると、外側の binder は `1` 以降へ
    /// 押し出される。この値は `.neplmeta` materializer が同名 generic parameter を
    /// 名前だけで誤対応させないための authority である。
    pub binder_depth: u32,
    /// `binder_depth` が指す binder 内での parameter index。
    ///
    /// index は `PublicTypeTerm::Function.type_params` や nominal type surface の
    /// `type_params` と対応する。`PublicTypeParam` 全体を term 側へ複製しないことで、
    /// binder metadata と binder reference の責務を分ける。
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicTypeParamBoundTarget {
    /// structured public surface 内で対応する binder を確定できた bound target。
    Ref(PublicTypeParamRef),
    /// 対応する binder を確定できなかった bound target。
    ///
    /// これは互換性のために推測して materialize してよい値ではない。将来の
    /// `.neplmeta` materializer はこの variant を fail-closed に扱い、dependency
    /// body skip へ進まず通常の source load / typecheck に戻す。
    Unbound(PublicTypeParam),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicNominalTypeIdentity {
    pub kind: PublicNominalTypeKind,
    pub source_path: String,
    pub name: String,
    pub arity: u32,
    pub definition_hash: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicNominalTypeKind {
    Enum,
    Struct,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicTypeTerm {
    Unit,
    I32,
    U8,
    F32,
    Bool,
    Char,
    Str,
    Never,
    TraitSelf,
    Named {
        name: String,
        identity: Option<PublicNominalTypeIdentity>,
    },
    GenericParam(PublicTypeParamRef),
    UnboundGenericParam(PublicTypeParam),
    Tuple(Vec<PublicTypeTerm>),
    Function {
        type_params: Vec<PublicTypeParam>,
        params: Vec<PublicTypeTerm>,
        result: Box<PublicTypeTerm>,
        effect: PublicEffect,
    },
    Apply {
        base: Box<PublicTypeTerm>,
        args: Vec<PublicTypeTerm>,
    },
    Boxed(Box<PublicTypeTerm>),
    Reference {
        inner: Box<PublicTypeTerm>,
        mutable: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicEffect {
    Pure,
    Impure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicTraitCapability {
    Copy,
    Clone,
    Drop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicFieldAccessorKind {
    Get,
    GetRef,
    Put,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicStructConstructorPolicy {
    Public,
    RawMemoryOwnerToken,
    RawMemoryPointer,
    OwnerBackedAggregate,
}

fn typed_public_surface_hash(entries: &[TypedPublicSurfaceEntry]) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    hash_str(&mut hash, "neplg2-typed-public-surface-v8");
    for entry in entries {
        hash_str(&mut hash, entry.kind.as_str());
        hash_str(&mut hash, entry.name.as_str());
        hash_bool(&mut hash, entry.exported);
        hash_public_surface_shape(&mut hash, &entry.surface);
    }
    hash
}

fn hash_public_surface_shape(hash: &mut u64, shape: &PublicSurfaceShape) {
    match shape {
        PublicSurfaceShape::Callable(surface) => {
            hash_str(hash, "callable");
            hash_public_type_term(hash, &surface.ty);
            hash_bool(hash, surface.no_shadow);
            hash_u32(hash, surface.arity);
            hash_public_effect(hash, surface.effect);
            hash_optional_public_field_accessor_kind(hash, surface.field_accessor);
            hash_optional_public_callable_link_symbol(hash, surface.link_symbol.as_ref());
            hash_u32(hash, surface.type_param_bounds.len() as u32);
            for bounds in &surface.type_param_bounds {
                hash_public_type_param_bound_target(hash, &bounds.param);
                hash_u32(hash, bounds.bounds.len() as u32);
                for bound in &bounds.bounds {
                    hash_public_trait_ref(hash, bound);
                }
            }
        }
        PublicSurfaceShape::Struct(surface) => {
            hash_str(hash, "struct");
            hash_optional_public_nominal_identity(hash, surface.identity.as_ref());
            hash_public_type_params(hash, &surface.type_params);
            hash_u32(hash, surface.fields.len() as u32);
            for field in &surface.fields {
                hash_str(hash, field.name.as_str());
                hash_public_type_term(hash, &field.ty);
            }
            hash_str(
                hash,
                public_struct_constructor_policy_tag(surface.constructor_policy),
            );
        }
        PublicSurfaceShape::Enum(surface) => {
            hash_str(hash, "enum");
            hash_optional_public_nominal_identity(hash, surface.identity.as_ref());
            hash_public_type_params(hash, &surface.type_params);
            hash_u32(hash, surface.variants.len() as u32);
            for variant in &surface.variants {
                hash_str(hash, variant.name.as_str());
                match &variant.payload {
                    Some(payload) => {
                        hash_bool(hash, true);
                        hash_public_type_term(hash, payload);
                    }
                    None => hash_bool(hash, false),
                }
            }
        }
        PublicSurfaceShape::Trait(surface) => {
            hash_str(hash, "trait");
            hash_optional_public_trait_identity(hash, surface.identity.as_ref());
            hash_public_type_params(hash, &surface.type_params);
            hash_u32(hash, surface.capabilities.len() as u32);
            for capability in &surface.capabilities {
                hash_str(hash, public_trait_capability_tag(*capability));
            }
            hash_u32(hash, surface.methods.len() as u32);
            for method in &surface.methods {
                hash_str(hash, method.name.as_str());
                hash_public_type_term(hash, &method.ty);
            }
        }
        PublicSurfaceShape::Impl(surface) => {
            hash_str(hash, "impl");
            hash_public_type_params(hash, &surface.type_params);
            hash_public_type_param_bounds(hash, &surface.type_param_bounds);
            hash_public_type_term(hash, &surface.target);
            match &surface.kind {
                PublicImplKind::Inherent => hash_str(hash, "inherent"),
                PublicImplKind::Trait { application } => {
                    hash_str(hash, "trait");
                    hash_public_trait_ref(hash, application);
                }
            }
        }
    }
}

fn collect_surface_shape_materializer_blockers(
    entry: &TypedPublicSurfaceEntry,
    shape: &PublicSurfaceShape,
    blockers: &mut Vec<PublicSurfaceMaterializerBlocker>,
) {
    match shape {
        PublicSurfaceShape::Callable(surface) => {
            if surface.link_symbol.is_none() {
                push_materializer_blocker(
                    entry,
                    PublicSurfaceMaterializerBlockerReason::MissingCallableLinkSymbol {
                        callable_name: entry.name.clone(),
                    },
                    blockers,
                );
            }
            collect_type_term_materializer_blockers(entry, &surface.ty, false, blockers);
            for bounds in &surface.type_param_bounds {
                collect_type_param_bound_target_materializer_blockers(
                    entry,
                    &bounds.param,
                    blockers,
                );
                for bound in &bounds.bounds {
                    collect_trait_ref_materializer_blockers(entry, bound, blockers);
                }
            }
        }
        PublicSurfaceShape::Struct(surface) => {
            if surface.identity.is_none() {
                push_materializer_blocker(
                    entry,
                    PublicSurfaceMaterializerBlockerReason::MissingStructIdentity,
                    blockers,
                );
            }
            for field in &surface.fields {
                collect_type_term_materializer_blockers(entry, &field.ty, false, blockers);
            }
        }
        PublicSurfaceShape::Enum(surface) => {
            if surface.identity.is_none() {
                push_materializer_blocker(
                    entry,
                    PublicSurfaceMaterializerBlockerReason::MissingEnumIdentity,
                    blockers,
                );
            }
            for variant in &surface.variants {
                if let Some(payload) = &variant.payload {
                    collect_type_term_materializer_blockers(entry, payload, false, blockers);
                }
            }
        }
        PublicSurfaceShape::Trait(surface) => {
            if surface.identity.is_none() {
                push_materializer_blocker(
                    entry,
                    PublicSurfaceMaterializerBlockerReason::MissingTraitIdentity {
                        trait_name: entry.name.clone(),
                    },
                    blockers,
                );
            }
            for method in &surface.methods {
                collect_type_term_materializer_blockers(entry, &method.ty, true, blockers);
            }
        }
        PublicSurfaceShape::Impl(surface) => {
            for bounds in &surface.type_param_bounds {
                collect_type_param_bound_target_materializer_blockers(
                    entry,
                    &bounds.param,
                    blockers,
                );
                for bound in &bounds.bounds {
                    collect_trait_ref_materializer_blockers(entry, bound, blockers);
                }
            }
            collect_type_term_materializer_blockers(entry, &surface.target, false, blockers);
            match &surface.kind {
                PublicImplKind::Inherent => {}
                PublicImplKind::Trait { application } => {
                    collect_trait_ref_materializer_blockers(entry, application, blockers);
                }
            }
        }
    }
}

fn collect_type_param_bound_target_materializer_blockers(
    entry: &TypedPublicSurfaceEntry,
    target: &PublicTypeParamBoundTarget,
    blockers: &mut Vec<PublicSurfaceMaterializerBlocker>,
) {
    match target {
        PublicTypeParamBoundTarget::Ref(_) => {}
        PublicTypeParamBoundTarget::Unbound(param) => {
            push_materializer_blocker(
                entry,
                PublicSurfaceMaterializerBlockerReason::UnboundTraitBoundTarget {
                    param_name: param.name.clone(),
                },
                blockers,
            );
        }
    }
}

fn collect_trait_ref_materializer_blockers(
    entry: &TypedPublicSurfaceEntry,
    trait_ref: &PublicTraitRef,
    blockers: &mut Vec<PublicSurfaceMaterializerBlocker>,
) {
    if trait_ref.identity.is_none() {
        push_materializer_blocker(
            entry,
            PublicSurfaceMaterializerBlockerReason::MissingTraitIdentity {
                trait_name: trait_ref.name.clone(),
            },
            blockers,
        );
    }
    for arg in &trait_ref.args {
        collect_type_term_materializer_blockers(entry, arg, false, blockers);
    }
}

fn collect_type_term_materializer_blockers(
    entry: &TypedPublicSurfaceEntry,
    term: &PublicTypeTerm,
    allow_trait_self: bool,
    blockers: &mut Vec<PublicSurfaceMaterializerBlocker>,
) {
    match term {
        PublicTypeTerm::Unit
        | PublicTypeTerm::I32
        | PublicTypeTerm::U8
        | PublicTypeTerm::F32
        | PublicTypeTerm::Bool
        | PublicTypeTerm::Char
        | PublicTypeTerm::Str
        | PublicTypeTerm::Never
        | PublicTypeTerm::GenericParam(_) => {}
        PublicTypeTerm::TraitSelf => {
            if !allow_trait_self {
                push_materializer_blocker(
                    entry,
                    PublicSurfaceMaterializerBlockerReason::TraitSelfOutsideTraitMethod,
                    blockers,
                );
            }
        }
        PublicTypeTerm::Named { name, identity } => {
            if identity.is_none() {
                if BackendScalarType::from_name(name.as_str()).is_none() {
                    push_materializer_blocker(
                        entry,
                        PublicSurfaceMaterializerBlockerReason::MissingNamedTypeIdentity {
                            type_name: name.clone(),
                        },
                        blockers,
                    );
                }
            }
        }
        PublicTypeTerm::UnboundGenericParam(param) => {
            push_materializer_blocker(
                entry,
                PublicSurfaceMaterializerBlockerReason::UnboundGenericParam {
                    param_name: param.name.clone(),
                },
                blockers,
            );
        }
        PublicTypeTerm::Tuple(items) => {
            for item in items {
                collect_type_term_materializer_blockers(entry, item, allow_trait_self, blockers);
            }
        }
        PublicTypeTerm::Function { params, result, .. } => {
            for param in params {
                collect_type_term_materializer_blockers(entry, param, allow_trait_self, blockers);
            }
            collect_type_term_materializer_blockers(entry, result, allow_trait_self, blockers);
        }
        PublicTypeTerm::Apply { base, args } => {
            collect_type_term_materializer_blockers(entry, base, allow_trait_self, blockers);
            for arg in args {
                collect_type_term_materializer_blockers(entry, arg, allow_trait_self, blockers);
            }
        }
        PublicTypeTerm::Boxed(inner) | PublicTypeTerm::Reference { inner, .. } => {
            collect_type_term_materializer_blockers(entry, inner, allow_trait_self, blockers);
        }
    }
}

fn push_materializer_blocker(
    entry: &TypedPublicSurfaceEntry,
    reason: PublicSurfaceMaterializerBlockerReason,
    blockers: &mut Vec<PublicSurfaceMaterializerBlocker>,
) {
    blockers.push(PublicSurfaceMaterializerBlocker {
        entry_kind: entry.kind,
        entry_name: entry.name.clone(),
        reason,
    });
}

fn hash_public_type_params(hash: &mut u64, params: &[PublicTypeParam]) {
    hash_u32(hash, params.len() as u32);
    for param in params {
        hash_public_type_param(hash, param);
    }
}

fn hash_public_type_param(hash: &mut u64, param: &PublicTypeParam) {
    hash_str(hash, param.name.as_str());
    hash_bool(hash, param.copy_cap);
    hash_bool(hash, param.clone_cap);
    hash_bool(hash, param.drop_cap);
}

fn hash_public_type_param_bounds(hash: &mut u64, bounds: &[PublicTypeParamBounds]) {
    hash_u32(hash, bounds.len() as u32);
    for bound_set in bounds {
        hash_public_type_param_bound_target(hash, &bound_set.param);
        hash_u32(hash, bound_set.bounds.len() as u32);
        for bound in &bound_set.bounds {
            hash_public_trait_ref(hash, bound);
        }
    }
}

fn hash_public_trait_ref(hash: &mut u64, trait_ref: &PublicTraitRef) {
    hash_str(hash, trait_ref.name.as_str());
    hash_optional_public_trait_identity(hash, trait_ref.identity.as_ref());
    hash_u32(hash, trait_ref.args.len() as u32);
    for arg in &trait_ref.args {
        hash_public_type_term(hash, arg);
    }
}

fn hash_optional_public_field_accessor_kind(hash: &mut u64, kind: Option<PublicFieldAccessorKind>) {
    match kind {
        Some(kind) => {
            hash_bool(hash, true);
            hash_str(hash, public_field_accessor_kind_tag(kind));
        }
        None => hash_bool(hash, false),
    }
}

fn hash_optional_public_callable_link_symbol(
    hash: &mut u64,
    symbol: Option<&PublicCallableLinkSymbol>,
) {
    match symbol {
        Some(symbol) => {
            hash_bool(hash, true);
            hash_str(hash, symbol.source_path.as_str());
            hash_str(hash, symbol.name.as_str());
            hash_u64(hash, symbol.signature_hash);
        }
        None => hash_bool(hash, false),
    }
}

fn hash_public_type_term(hash: &mut u64, term: &PublicTypeTerm) {
    match term {
        PublicTypeTerm::Unit => hash_str(hash, "unit"),
        PublicTypeTerm::I32 => hash_str(hash, "i32"),
        PublicTypeTerm::U8 => hash_str(hash, "u8"),
        PublicTypeTerm::F32 => hash_str(hash, "f32"),
        PublicTypeTerm::Bool => hash_str(hash, "bool"),
        PublicTypeTerm::Char => hash_str(hash, "char"),
        PublicTypeTerm::Str => hash_str(hash, "str"),
        PublicTypeTerm::Never => hash_str(hash, "never"),
        PublicTypeTerm::TraitSelf => hash_str(hash, "trait-self"),
        PublicTypeTerm::Named { name, identity } => {
            hash_str(hash, "named");
            hash_str(hash, name.as_str());
            hash_optional_public_nominal_identity(hash, identity.as_ref());
        }
        PublicTypeTerm::GenericParam(param_ref) => {
            hash_str(hash, "generic-ref");
            hash_public_type_param_ref(hash, param_ref);
        }
        PublicTypeTerm::UnboundGenericParam(param) => {
            hash_str(hash, "unbound-generic");
            hash_public_type_param(hash, param);
        }
        PublicTypeTerm::Tuple(items) => {
            hash_str(hash, "tuple");
            hash_u32(hash, items.len() as u32);
            for item in items {
                hash_public_type_term(hash, item);
            }
        }
        PublicTypeTerm::Function {
            type_params,
            params,
            result,
            effect,
        } => {
            hash_str(hash, "function");
            hash_public_type_params(hash, type_params);
            hash_u32(hash, params.len() as u32);
            for param in params {
                hash_public_type_term(hash, param);
            }
            hash_public_type_term(hash, result);
            hash_public_effect(hash, *effect);
        }
        PublicTypeTerm::Apply { base, args } => {
            hash_str(hash, "apply");
            hash_public_type_term(hash, base);
            hash_u32(hash, args.len() as u32);
            for arg in args {
                hash_public_type_term(hash, arg);
            }
        }
        PublicTypeTerm::Boxed(inner) => {
            hash_str(hash, "box");
            hash_public_type_term(hash, inner);
        }
        PublicTypeTerm::Reference { inner, mutable } => {
            hash_str(hash, "ref");
            hash_bool(hash, *mutable);
            hash_public_type_term(hash, inner);
        }
    }
}

fn hash_public_type_param_ref(hash: &mut u64, param_ref: &PublicTypeParamRef) {
    hash_u32(hash, param_ref.binder_depth);
    hash_u32(hash, param_ref.index);
}

fn hash_public_type_param_bound_target(hash: &mut u64, target: &PublicTypeParamBoundTarget) {
    match target {
        PublicTypeParamBoundTarget::Ref(param_ref) => {
            hash_str(hash, "ref");
            hash_public_type_param_ref(hash, param_ref);
        }
        PublicTypeParamBoundTarget::Unbound(param) => {
            hash_str(hash, "unbound");
            hash_public_type_param(hash, param);
        }
    }
}

fn hash_public_effect(hash: &mut u64, effect: PublicEffect) {
    hash_str(hash, public_effect_tag(effect));
}

fn hash_optional_public_nominal_identity(
    hash: &mut u64,
    identity: Option<&PublicNominalTypeIdentity>,
) {
    match identity {
        Some(identity) => {
            hash_bool(hash, true);
            hash_str(hash, public_nominal_type_kind_tag(identity.kind));
            hash_str(hash, identity.source_path.as_str());
            hash_str(hash, identity.name.as_str());
            hash_u32(hash, identity.arity);
            hash_u64(hash, identity.definition_hash);
        }
        None => hash_bool(hash, false),
    }
}

fn hash_optional_public_trait_identity(hash: &mut u64, identity: Option<&PublicTraitIdentity>) {
    match identity {
        Some(identity) => {
            hash_bool(hash, true);
            hash_str(hash, identity.source_path.as_str());
            hash_str(hash, identity.name.as_str());
            hash_u32(hash, identity.arity);
            hash_u64(hash, identity.definition_hash);
        }
        None => hash_bool(hash, false),
    }
}

fn public_nominal_type_kind_tag(kind: PublicNominalTypeKind) -> &'static str {
    match kind {
        PublicNominalTypeKind::Enum => "enum",
        PublicNominalTypeKind::Struct => "struct",
    }
}

fn stable_symbol_component(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        String::from("_")
    } else {
        out
    }
}

fn fnv1a64(text: &str) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV1A64_PRIME);
    }
    hash
}

fn hash_str(hash: &mut u64, value: &str) {
    hash_bytes(hash, value.as_bytes());
    hash_bytes(hash, &[0]);
}

fn hash_bool(hash: &mut u64, value: bool) {
    hash_bytes(hash, &[u8::from(value), 0xff]);
}

fn hash_u32(hash: &mut u64, value: u32) {
    hash_bytes(hash, &value.to_le_bytes());
}

fn hash_u64(hash: &mut u64, value: u64) {
    hash_bytes(hash, &value.to_le_bytes());
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(FNV1A64_PRIME);
    }
}

struct PublicNominalDefinitionHasher {
    state: u64,
}

impl PublicNominalDefinitionHasher {
    fn new(namespace: &str) -> Self {
        let mut hasher = Self {
            state: FNV1A64_OFFSET,
        };
        hasher.write_str(namespace);
        hasher
    }

    fn write_str(&mut self, value: &str) {
        self.write_usize(value.len());
        for byte in value.as_bytes() {
            self.write_u8(*byte);
        }
    }

    fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }

    fn write_u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.write_u8(byte);
        }
    }

    fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    fn write_u8(&mut self, value: u8) {
        self.state ^= u64::from(value);
        self.state = self.state.wrapping_mul(FNV1A64_PRIME);
    }

    fn finish(self) -> u64 {
        self.state
    }
}

fn hash_public_nominal_type_params(
    hash: &mut PublicNominalDefinitionHasher,
    params: &[PublicTypeParam],
) -> Option<()> {
    hash.write_usize(params.len());
    for param in params {
        hash_public_nominal_type_param(hash, param);
    }
    Some(())
}

fn hash_public_nominal_type_param(
    hash: &mut PublicNominalDefinitionHasher,
    param: &PublicTypeParam,
) {
    hash.write_str("var");
    hash.write_str(param.name.as_str());
    hash.write_bool(param.copy_cap);
    hash.write_bool(param.clone_cap);
    hash.write_bool(param.drop_cap);
}

fn hash_public_nominal_type_list<'a>(
    hash: &mut PublicNominalDefinitionHasher,
    items: &'a [PublicTypeTerm],
    binders: &mut Vec<&'a [PublicTypeParam]>,
) -> Option<()> {
    hash.write_usize(items.len());
    for item in items {
        hash_public_nominal_type_surface(hash, item, binders)?;
    }
    Some(())
}

fn hash_public_nominal_type_surface<'a>(
    hash: &mut PublicNominalDefinitionHasher,
    term: &'a PublicTypeTerm,
    binders: &mut Vec<&'a [PublicTypeParam]>,
) -> Option<()> {
    match term {
        PublicTypeTerm::Unit => hash.write_str("unit"),
        PublicTypeTerm::I32 => hash.write_str("i32"),
        PublicTypeTerm::U8 => hash.write_str("u8"),
        PublicTypeTerm::F32 => hash.write_str("f32"),
        PublicTypeTerm::Bool => hash.write_str("bool"),
        PublicTypeTerm::Char => hash.write_str("char"),
        PublicTypeTerm::Str => hash.write_str("str"),
        PublicTypeTerm::Never => hash.write_str("never"),
        PublicTypeTerm::TraitSelf | PublicTypeTerm::UnboundGenericParam(_) => return None,
        PublicTypeTerm::Named { name, identity } => match identity {
            Some(identity) => {
                hash.write_str(public_nominal_identity_stable_key_component(identity).as_str());
            }
            None => {
                let scalar = BackendScalarType::from_name(name.as_str())?;
                hash.write_str("backend-scalar");
                hash.write_str(scalar.source_name());
            }
        },
        PublicTypeTerm::GenericParam(param_ref) => {
            let param = public_nominal_type_param_ref(binders, param_ref)?;
            hash_public_nominal_type_param(hash, param);
        }
        PublicTypeTerm::Tuple(items) => {
            hash.write_str("tuple");
            hash_public_nominal_type_list(hash, items, binders)?;
        }
        PublicTypeTerm::Function {
            type_params,
            params,
            result,
            effect,
        } => {
            hash.write_str("fn");
            hash.write_str(match effect {
                PublicEffect::Pure => "pure",
                PublicEffect::Impure => "impure",
            });
            hash_public_nominal_type_params(hash, type_params)?;
            binders.push(type_params);
            hash_public_nominal_type_list(hash, params, binders)?;
            hash_public_nominal_type_surface(hash, result, binders)?;
            binders.pop();
        }
        PublicTypeTerm::Apply { base, args } => {
            hash.write_str("apply");
            hash_public_nominal_type_surface(hash, base, binders)?;
            hash_public_nominal_type_list(hash, args, binders)?;
        }
        PublicTypeTerm::Boxed(inner) => {
            hash.write_str("box");
            hash_public_nominal_type_surface(hash, inner, binders)?;
        }
        PublicTypeTerm::Reference { inner, mutable } => {
            hash.write_str("ref");
            hash.write_bool(*mutable);
            hash_public_nominal_type_surface(hash, inner, binders)?;
        }
    }
    Some(())
}

fn public_nominal_type_param_ref<'a>(
    binders: &'a [&[PublicTypeParam]],
    param_ref: &PublicTypeParamRef,
) -> Option<&'a PublicTypeParam> {
    let binder_depth = param_ref.binder_depth as usize;
    let index = param_ref.index as usize;
    let binder_index = binders.len().checked_sub(1 + binder_depth)?;
    binders.get(binder_index)?.get(index)
}

fn public_nominal_identity_stable_key_component(identity: &PublicNominalTypeIdentity) -> String {
    format!(
        "nominal(kind={},path={},name={},arity={},hash={:016x})",
        public_nominal_type_kind_tag(identity.kind),
        public_stable_text_component(identity.source_path.as_str()),
        public_stable_text_component(identity.name.as_str()),
        identity.arity,
        identity.definition_hash
    )
}

fn public_stable_text_component(text: &str) -> String {
    format!("{}:{text}", text.len())
}

fn public_effect_from_ast(effect: Effect) -> PublicEffect {
    match effect {
        Effect::Pure => PublicEffect::Pure,
        Effect::Impure => PublicEffect::Impure,
    }
}

fn public_effect_tag(effect: PublicEffect) -> &'static str {
    match effect {
        PublicEffect::Pure => "pure",
        PublicEffect::Impure => "impure",
    }
}

fn public_trait_capability_from_model(capability: TraitCapability) -> PublicTraitCapability {
    match capability {
        TraitCapability::Copy => PublicTraitCapability::Copy,
        TraitCapability::Clone => PublicTraitCapability::Clone,
        TraitCapability::Drop => PublicTraitCapability::Drop,
    }
}

fn public_trait_capability_tag(capability: PublicTraitCapability) -> &'static str {
    match capability {
        PublicTraitCapability::Copy => "Copy",
        PublicTraitCapability::Clone => "Clone",
        PublicTraitCapability::Drop => "Drop",
    }
}

fn public_field_accessor_kind_from_model(kind: FieldAccessorKind) -> PublicFieldAccessorKind {
    match kind {
        FieldAccessorKind::Get => PublicFieldAccessorKind::Get,
        FieldAccessorKind::GetRef => PublicFieldAccessorKind::GetRef,
        FieldAccessorKind::Put => PublicFieldAccessorKind::Put,
    }
}

fn public_field_accessor_kind_tag(kind: PublicFieldAccessorKind) -> &'static str {
    match kind {
        PublicFieldAccessorKind::Get => "get",
        PublicFieldAccessorKind::GetRef => "get_ref",
        PublicFieldAccessorKind::Put => "put",
    }
}

fn public_struct_constructor_policy_from_model(
    policy: StructConstructorPolicy,
) -> PublicStructConstructorPolicy {
    match policy {
        StructConstructorPolicy::Public => PublicStructConstructorPolicy::Public,
        StructConstructorPolicy::RawMemoryBoundaryOnly(RestrictedStructConstructor::OwnerToken) => {
            PublicStructConstructorPolicy::RawMemoryOwnerToken
        }
        StructConstructorPolicy::RawMemoryBoundaryOnly(RestrictedStructConstructor::RawPointer) => {
            PublicStructConstructorPolicy::RawMemoryPointer
        }
        StructConstructorPolicy::OwnerBackedAggregateBoundaryOnly => {
            PublicStructConstructorPolicy::OwnerBackedAggregate
        }
    }
}

fn public_struct_constructor_policy_tag(policy: PublicStructConstructorPolicy) -> &'static str {
    match policy {
        PublicStructConstructorPolicy::Public => "public",
        PublicStructConstructorPolicy::RawMemoryOwnerToken => "raw_memory_owner_token",
        PublicStructConstructorPolicy::RawMemoryPointer => "raw_memory_pointer",
        PublicStructConstructorPolicy::OwnerBackedAggregate => "owner_backed_aggregate",
    }
}

pub(super) fn build_typed_public_surface_table(
    ctx: &TypeCtx,
    source_map: Option<&SourceMap>,
    env: &Env,
    structs: &BTreeMap<String, StructInfo>,
    enums: &BTreeMap<String, EnumInfo>,
    traits: &BTreeMap<String, TraitInfo>,
    impls: &[ImplInfo],
) -> TypedPublicSurfaceTable {
    build_typed_public_surface_table_excluding_files(
        ctx,
        source_map,
        env,
        structs,
        enums,
        traits,
        impls,
        &BTreeSet::new(),
    )
}

pub(super) fn build_typed_public_surface_table_excluding_files(
    ctx: &TypeCtx,
    source_map: Option<&SourceMap>,
    env: &Env,
    structs: &BTreeMap<String, StructInfo>,
    enums: &BTreeMap<String, EnumInfo>,
    traits: &BTreeMap<String, TraitInfo>,
    impls: &[ImplInfo],
    excluded_files: &BTreeSet<u32>,
) -> TypedPublicSurfaceTable {
    let mut entries = Vec::new();
    let mut semantic_trait_names = BTreeMap::new();
    if let Some(global_scope) = env.scopes.first() {
        for binding in global_scope.callables.iter().filter(|binding| {
            binding.defined
                && binding.visibility == Visibility::Pub
                && !excluded_files.contains(&binding.span.file_id.0)
        }) {
            if let BindingKind::Func {
                effect,
                arity,
                field_accessor,
                type_param_bounds,
                ..
            } = &binding.kind
            {
                let ty = public_type_term(ctx, binding.ty, &BTreeMap::new());
                collect_bound_env_trait_names(type_param_bounds, &mut semantic_trait_names);
                entries.push(TypedPublicSurfaceEntry {
                    kind: TypedPublicSignatureKind::Callable,
                    name: binding.name.clone(),
                    exported: true,
                    surface: PublicSurfaceShape::Callable(PublicCallableSurface {
                        ty: ty.clone(),
                        no_shadow: binding.no_shadow,
                        arity: usize_to_u32_saturating(*arity),
                        effect: public_effect_from_ast(*effect),
                        field_accessor: field_accessor.map(public_field_accessor_kind_from_model),
                        link_symbol: public_callable_link_symbol(source_map, binding, &ty),
                        type_param_bounds: public_type_param_bounds(
                            ctx,
                            source_map,
                            traits,
                            type_param_bounds,
                            &public_function_root_generics(ctx, binding.ty),
                        ),
                    }),
                });
            }
        }
    }
    for (name, info) in structs.iter().filter(|(_, info)| {
        info.visibility == Visibility::Pub && !excluded_files.contains(&info.span.file_id.0)
    }) {
        entries.push(TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Struct,
            name: name.clone(),
            exported: true,
            surface: PublicSurfaceShape::Struct(public_struct_surface(ctx, info)),
        });
    }
    for (name, info) in enums.iter().filter(|(_, info)| {
        info.visibility == Visibility::Pub && !excluded_files.contains(&info.span.file_id.0)
    }) {
        entries.push(TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Enum,
            name: name.clone(),
            exported: true,
            surface: PublicSurfaceShape::Enum(public_enum_surface(ctx, info)),
        });
    }
    collect_impl_trait_names_excluding_files(impls, excluded_files, &mut semantic_trait_names);
    for (name, info) in traits.iter().filter(|(name, info)| {
        !excluded_files.contains(&info.span.file_id.0)
            && (info.visibility == Visibility::Pub || semantic_trait_names.contains_key(*name))
    }) {
        entries.push(TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Trait,
            name: name.clone(),
            exported: info.visibility == Visibility::Pub,
            surface: PublicSurfaceShape::Trait(public_trait_surface(ctx, source_map, name, info)),
        });
    }
    for impl_info in impls
        .iter()
        .filter(|impl_info| !excluded_files.contains(&impl_info.span.file_id.0))
    {
        entries.push(TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Impl,
            name: impl_public_name(ctx, impl_info),
            exported: false,
            surface: PublicSurfaceShape::Impl(public_impl_surface(
                ctx, source_map, traits, impl_info,
            )),
        });
    }
    TypedPublicSurfaceTable::new(entries)
}

fn collect_impl_trait_names_excluding_files(
    impls: &[ImplInfo],
    excluded_files: &BTreeSet<u32>,
    out: &mut BTreeMap<String, ()>,
) {
    for info in impls {
        if excluded_files.contains(&info.span.file_id.0) {
            continue;
        }
        collect_bound_env_trait_names(&info.type_param_bounds, out);
        if let ImplKind::Trait { application, .. } = &info.kind {
            out.insert(String::from(application.trait_id.as_str()), ());
        }
    }
}

fn collect_bound_env_trait_names(bounds: &BoundEnv, out: &mut BTreeMap<String, ()>) {
    for (_type_param, trait_bounds) in bounds.iter() {
        for bound in trait_bounds {
            out.insert(String::from(bound.application.trait_id.as_str()), ());
        }
    }
}

fn public_callable_link_symbol(
    source_map: Option<&SourceMap>,
    binding: &Binding,
    ty: &PublicTypeTerm,
) -> Option<PublicCallableLinkSymbol> {
    let source_path = source_map?
        .path(binding.span.file_id)
        .map(|path| String::from(path.as_str()))?;
    Some(PublicCallableLinkSymbol {
        source_path,
        name: binding.name.clone(),
        signature_hash: public_type_term_stable_hash(ty),
    })
}

pub(super) fn public_type_term_stable_hash(term: &PublicTypeTerm) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    hash_str(&mut hash, "neplg2-public-type-term-v1");
    hash_public_type_term(&mut hash, term);
    hash
}

pub(super) fn public_struct_definition_hash<'a>(
    type_params: &'a [PublicTypeParam],
    fields: &'a [PublicFieldSurface],
) -> Option<u64> {
    let mut hash = PublicNominalDefinitionHasher::new("neplg2-nominal-definition-surface-v1");
    hash.write_str("struct");
    hash_public_nominal_type_params(&mut hash, type_params)?;
    hash.write_usize(fields.len());
    let mut binders = Vec::from([type_params]);
    for field in fields {
        hash.write_str(field.name.as_str());
        hash_public_nominal_type_surface(&mut hash, &field.ty, &mut binders)?;
    }
    Some(hash.finish())
}

pub(super) fn public_enum_definition_hash<'a>(
    type_params: &'a [PublicTypeParam],
    variants: &'a [PublicEnumVariantSurface],
) -> Option<u64> {
    let mut hash = PublicNominalDefinitionHasher::new("neplg2-nominal-definition-surface-v1");
    hash.write_str("enum");
    hash_public_nominal_type_params(&mut hash, type_params)?;
    hash.write_usize(variants.len());
    let mut binders = Vec::from([type_params]);
    for variant in variants {
        hash.write_str(variant.name.as_str());
        match &variant.payload {
            Some(payload) => {
                hash.write_bool(true);
                hash_public_nominal_type_surface(&mut hash, payload, &mut binders)?;
            }
            None => hash.write_bool(false),
        }
    }
    Some(hash.finish())
}

fn public_struct_surface(ctx: &TypeCtx, info: &StructInfo) -> PublicStructSurface {
    let (type_params, generics) = public_type_params(ctx, &info.type_params);
    PublicStructSurface {
        identity: public_nominal_type_identity(ctx, info.ty),
        type_params,
        fields: info
            .field_names
            .iter()
            .zip(info.fields.iter())
            .map(|(name, field)| PublicFieldSurface {
                name: name.clone(),
                ty: public_type_term(ctx, *field, &generics),
            })
            .collect(),
        constructor_policy: public_struct_constructor_policy_from_model(info.constructor_policy),
    }
}

fn public_enum_surface(ctx: &TypeCtx, info: &EnumInfo) -> PublicEnumSurface {
    let (type_params, generics) = public_type_params(ctx, &info.type_params);
    PublicEnumSurface {
        identity: public_nominal_type_identity(ctx, info.ty),
        type_params,
        variants: info
            .variants
            .iter()
            .map(|variant| PublicEnumVariantSurface {
                name: variant.name.clone(),
                payload: variant
                    .payload
                    .map(|payload| public_type_term(ctx, payload, &generics)),
            })
            .collect(),
    }
}

fn public_trait_surface(
    ctx: &TypeCtx,
    source_map: Option<&SourceMap>,
    name: &str,
    info: &TraitInfo,
) -> PublicTraitSurface {
    let (type_params, generics) = public_type_params(ctx, &info.type_params);
    let mut capabilities = info
        .capabilities
        .iter()
        .copied()
        .map(public_trait_capability_from_model)
        .collect::<Vec<_>>();
    capabilities.sort();
    let mut methods = info
        .methods
        .iter()
        .map(|(name, method)| PublicTraitMethodSurface {
            name: name.clone(),
            ty: public_type_term_with_trait_self(ctx, *method, &generics, Some(info.self_ty)),
        })
        .collect::<Vec<_>>();
    methods.sort();
    let definition_hash = public_trait_definition_hash(&type_params, &capabilities, &methods);
    PublicTraitSurface {
        identity: info
            .stable_identity
            .as_ref()
            .map(|identity| PublicTraitIdentity {
                source_path: identity.source_path.clone(),
                name: identity.name.clone(),
                arity: identity.arity,
                definition_hash: identity.definition_hash,
            })
            .or_else(|| {
                public_trait_identity(
                    source_map,
                    info.span,
                    name,
                    type_params.len(),
                    definition_hash,
                )
            }),
        type_params,
        capabilities,
        methods,
    }
}

fn public_impl_surface(
    ctx: &TypeCtx,
    source_map: Option<&SourceMap>,
    traits: &BTreeMap<String, TraitInfo>,
    info: &ImplInfo,
) -> PublicImplSurface {
    let (type_params, generics) = public_type_params(ctx, &info.type_params);
    let type_param_bounds =
        public_type_param_bounds(ctx, source_map, traits, &info.type_param_bounds, &generics);
    PublicImplSurface {
        type_params,
        type_param_bounds,
        kind: match &info.kind {
            ImplKind::Inherent => PublicImplKind::Inherent,
            ImplKind::Trait { application, .. } => PublicImplKind::Trait {
                application: public_trait_ref_from_application(
                    ctx,
                    source_map,
                    traits,
                    application,
                    &generics,
                ),
            },
        },
        target: public_type_term(ctx, info.target_ty, &generics),
    }
}

fn public_type_param_bounds(
    ctx: &TypeCtx,
    source_map: Option<&SourceMap>,
    traits: &BTreeMap<String, TraitInfo>,
    bounds: &BoundEnv,
    generics: &BTreeMap<TypeId, PublicTypeParamRef>,
) -> Vec<PublicTypeParamBounds> {
    let mut out = bounds
        .iter()
        .map(|(type_param, trait_bounds)| PublicTypeParamBounds {
            param: public_type_param_bound_target(ctx, type_param.type_id(), generics),
            bounds: trait_bounds
                .iter()
                .map(|bound| {
                    public_trait_ref_from_application(
                        ctx,
                        source_map,
                        traits,
                        &bound.application,
                        generics,
                    )
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    out.sort();
    out
}

fn public_type_param_bound_target(
    ctx: &TypeCtx,
    type_param: TypeId,
    generics: &BTreeMap<TypeId, PublicTypeParamRef>,
) -> PublicTypeParamBoundTarget {
    match generics.get(&ctx.resolve_id(type_param)) {
        Some(param_ref) => PublicTypeParamBoundTarget::Ref(param_ref.clone()),
        None => PublicTypeParamBoundTarget::Unbound(public_type_param(ctx, type_param, 0)),
    }
}

fn public_trait_ref_from_application(
    ctx: &TypeCtx,
    source_map: Option<&SourceMap>,
    traits: &BTreeMap<String, TraitInfo>,
    application: &TraitApplication,
    generics: &BTreeMap<TypeId, PublicTypeParamRef>,
) -> PublicTraitRef {
    let trait_name = application.trait_id.as_str();
    PublicTraitRef {
        name: String::from(trait_name),
        identity: traits
            .get(trait_name)
            .and_then(|info| public_trait_identity_for_info(ctx, source_map, trait_name, info)),
        args: application
            .args
            .iter()
            .map(|arg| public_type_term(ctx, *arg, generics))
            .collect(),
    }
}

fn public_type_params(
    ctx: &TypeCtx,
    type_params: &[TypeId],
) -> (Vec<PublicTypeParam>, BTreeMap<TypeId, PublicTypeParamRef>) {
    let mut params = Vec::new();
    let mut generics = BTreeMap::new();
    for (index, type_param) in type_params.iter().enumerate() {
        let param = public_type_param(ctx, *type_param, index);
        generics.insert(
            ctx.resolve_id(*type_param),
            PublicTypeParamRef {
                binder_depth: 0,
                index: usize_to_u32_saturating(index),
            },
        );
        params.push(param);
    }
    (params, generics)
}

fn public_function_root_generics(
    ctx: &TypeCtx,
    ty: TypeId,
) -> BTreeMap<TypeId, PublicTypeParamRef> {
    match ctx.get(ctx.resolve_id(ty)) {
        TypeKind::Function { type_params, .. } => public_type_params(ctx, &type_params).1,
        _ => BTreeMap::new(),
    }
}

fn increment_generic_binder_depths(
    generics: &BTreeMap<TypeId, PublicTypeParamRef>,
) -> BTreeMap<TypeId, PublicTypeParamRef> {
    generics
        .iter()
        .map(|(type_id, param_ref)| {
            let mut shifted = param_ref.clone();
            shifted.binder_depth = shifted.binder_depth.saturating_add(1);
            (*type_id, shifted)
        })
        .collect()
}

fn public_type_param(ctx: &TypeCtx, type_param: TypeId, index: usize) -> PublicTypeParam {
    match ctx.get(ctx.resolve_id(type_param)) {
        TypeKind::Var(var) => PublicTypeParam {
            name: var.label.unwrap_or_else(|| format!("$T{index}")),
            copy_cap: var.copy_cap,
            clone_cap: var.clone_cap,
            drop_cap: var.drop_cap,
        },
        _ => PublicTypeParam {
            name: format!("$T{index}"),
            copy_cap: false,
            clone_cap: false,
            drop_cap: false,
        },
    }
}

fn public_nominal_type_identity(ctx: &TypeCtx, ty: TypeId) -> Option<PublicNominalTypeIdentity> {
    let identity = ctx.nominal_stable_identity(ty)?;
    Some(PublicNominalTypeIdentity {
        kind: public_nominal_type_kind(identity.kind()),
        source_path: String::from(identity.source_path()),
        name: String::from(identity.name()),
        arity: usize_to_u32_saturating(identity.arity()),
        definition_hash: identity.definition_hash(),
    })
}

fn public_nominal_type_kind(kind: NominalStableTypeKind) -> PublicNominalTypeKind {
    match kind {
        NominalStableTypeKind::Enum => PublicNominalTypeKind::Enum,
        NominalStableTypeKind::Struct => PublicNominalTypeKind::Struct,
    }
}

fn public_trait_identity_for_info(
    ctx: &TypeCtx,
    source_map: Option<&SourceMap>,
    name: &str,
    info: &TraitInfo,
) -> Option<PublicTraitIdentity> {
    let (type_params, generics) = public_type_params(ctx, &info.type_params);
    let mut capabilities = info
        .capabilities
        .iter()
        .copied()
        .map(public_trait_capability_from_model)
        .collect::<Vec<_>>();
    capabilities.sort();
    let mut methods = info
        .methods
        .iter()
        .map(|(method_name, method)| PublicTraitMethodSurface {
            name: method_name.clone(),
            ty: public_type_term_with_trait_self(ctx, *method, &generics, Some(info.self_ty)),
        })
        .collect::<Vec<_>>();
    methods.sort();
    Some(public_trait_identity(
        source_map,
        info.span,
        name,
        type_params.len(),
        public_trait_definition_hash(&type_params, &capabilities, &methods),
    )?)
}

fn public_trait_identity(
    source_map: Option<&SourceMap>,
    span: crate::span::Span,
    name: &str,
    arity: usize,
    definition_hash: u64,
) -> Option<PublicTraitIdentity> {
    let source_path = source_map?
        .path(span.file_id)
        .map(|path| String::from(path.as_str()))?;
    Some(PublicTraitIdentity {
        source_path,
        name: String::from(name),
        arity: usize_to_u32_saturating(arity),
        definition_hash,
    })
}

pub(super) fn public_trait_definition_hash(
    type_params: &[PublicTypeParam],
    capabilities: &[PublicTraitCapability],
    methods: &[PublicTraitMethodSurface],
) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    hash_str(&mut hash, "neplg2-public-trait-definition-v1");
    hash_public_type_params(&mut hash, type_params);
    hash_u32(&mut hash, capabilities.len() as u32);
    for capability in capabilities {
        hash_str(&mut hash, public_trait_capability_tag(*capability));
    }
    hash_u32(&mut hash, methods.len() as u32);
    for method in methods {
        hash_str(&mut hash, method.name.as_str());
        hash_public_type_term(&mut hash, &method.ty);
    }
    hash
}

fn public_type_term(
    ctx: &TypeCtx,
    ty: TypeId,
    generics: &BTreeMap<TypeId, PublicTypeParamRef>,
) -> PublicTypeTerm {
    public_type_term_with_trait_self(ctx, ty, generics, None)
}

fn public_type_term_with_trait_self(
    ctx: &TypeCtx,
    ty: TypeId,
    generics: &BTreeMap<TypeId, PublicTypeParamRef>,
    trait_self: Option<TypeId>,
) -> PublicTypeTerm {
    let resolved = ctx.resolve_id(ty);
    if trait_self.map(|self_ty| ctx.resolve_id(self_ty)) == Some(resolved) {
        return PublicTypeTerm::TraitSelf;
    }
    if let Some(param) = generics.get(&resolved) {
        return PublicTypeTerm::GenericParam(param.clone());
    }
    match ctx.get(resolved) {
        TypeKind::Unit => PublicTypeTerm::Unit,
        TypeKind::I32 => PublicTypeTerm::I32,
        TypeKind::U8 => PublicTypeTerm::U8,
        TypeKind::F32 => PublicTypeTerm::F32,
        TypeKind::Bool => PublicTypeTerm::Bool,
        TypeKind::Char => PublicTypeTerm::Char,
        TypeKind::Str => PublicTypeTerm::Str,
        TypeKind::Never => PublicTypeTerm::Never,
        TypeKind::Named(name) => PublicTypeTerm::Named {
            name,
            identity: None,
        },
        TypeKind::Enum { name, .. } | TypeKind::Struct { name, .. } => PublicTypeTerm::Named {
            name,
            identity: public_nominal_type_identity(ctx, resolved),
        },
        TypeKind::Tuple { items } => PublicTypeTerm::Tuple(
            items
                .iter()
                .map(|item| public_type_term_with_trait_self(ctx, *item, generics, trait_self))
                .collect(),
        ),
        TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        } => {
            let (function_params, function_generics) = public_type_params(ctx, &type_params);
            let mut scoped_generics = if type_params.is_empty() {
                generics.clone()
            } else {
                increment_generic_binder_depths(generics)
            };
            scoped_generics.extend(function_generics);
            PublicTypeTerm::Function {
                type_params: function_params,
                params: params
                    .iter()
                    .map(|param| {
                        public_type_term_with_trait_self(ctx, *param, &scoped_generics, trait_self)
                    })
                    .collect(),
                result: Box::new(public_type_term_with_trait_self(
                    ctx,
                    result,
                    &scoped_generics,
                    trait_self,
                )),
                effect: public_effect_from_ast(effect),
            }
        }
        TypeKind::Var(var) => PublicTypeTerm::UnboundGenericParam(PublicTypeParam {
            name: var.label.unwrap_or_else(|| String::from("$unbound")),
            copy_cap: var.copy_cap,
            clone_cap: var.clone_cap,
            drop_cap: var.drop_cap,
        }),
        TypeKind::Apply { base, args } => PublicTypeTerm::Apply {
            base: Box::new(public_type_term_with_trait_self(
                ctx, base, generics, trait_self,
            )),
            args: args
                .iter()
                .map(|arg| public_type_term_with_trait_self(ctx, *arg, generics, trait_self))
                .collect(),
        },
        TypeKind::Box(inner) => PublicTypeTerm::Boxed(Box::new(public_type_term_with_trait_self(
            ctx, inner, generics, trait_self,
        ))),
        TypeKind::Reference(inner, mutable) => PublicTypeTerm::Reference {
            inner: Box::new(public_type_term_with_trait_self(
                ctx, inner, generics, trait_self,
            )),
            mutable,
        },
    }
}

fn impl_public_name(ctx: &TypeCtx, info: &ImplInfo) -> String {
    let generics = public_impl_signature_generic_names(ctx, &info.type_params);
    match &info.kind {
        ImplKind::Inherent => format!(
            "impl:{}",
            signature_type_string(ctx, info.target_ty, &generics)
        ),
        ImplKind::Trait { application, .. } => {
            public_trait_application_signature_name(ctx, application, &generics)
        }
    }
}

fn public_impl_signature_generic_names(
    ctx: &TypeCtx,
    type_params: &[TypeId],
) -> BTreeMap<TypeId, String> {
    type_params
        .iter()
        .enumerate()
        .map(|(index, type_param)| (ctx.resolve_id(*type_param), format!("$T{index}")))
        .collect()
}

fn public_trait_application_signature_name(
    ctx: &TypeCtx,
    application: &TraitApplication,
    generics: &BTreeMap<TypeId, String>,
) -> String {
    if application.args.is_empty() {
        return String::from(application.trait_id.as_str());
    }
    let mut name = String::from(application.trait_id.as_str());
    name.push('<');
    for (index, arg) in application.args.iter().enumerate() {
        if index > 0 {
            name.push(',');
        }
        name.push_str(&signature_type_string(ctx, *arg, generics));
    }
    name.push('>');
    name
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
    use alloc::boxed::Box;
    use alloc::string::String;
    use alloc::vec::Vec;

    use crate::ast::Effect;
    use crate::compiler::{BuildProfile, CompileTarget};
    use crate::lexer;
    use crate::parser;
    use crate::source_map::SourceMap;
    use crate::span::FileId;
    use crate::typecheck::{typecheck, TypeCheckResult};
    use crate::types::TypeCtx;

    use super::{
        public_type_params, public_type_term, PublicCallableLinkSymbol, PublicCallableSurface,
        PublicEffect, PublicFieldAccessorKind, PublicImplKind, PublicNominalTypeKind,
        PublicSurfaceMaterializerBlockerReason, PublicSurfaceShape, PublicTraitIdentity,
        PublicTraitRef, PublicTypeParam, PublicTypeParamBoundTarget, PublicTypeParamBounds,
        PublicTypeParamRef, PublicTypeTerm, TypedPublicSignatureKind, TypedPublicSurfaceEntry,
        TypedPublicSurfaceTable,
    };

    fn typecheck_source(source: &str) -> TypeCheckResult {
        let file_id = FileId(0);
        let lex = lexer::lex(file_id, source);
        assert!(
            lex.diagnostics.is_empty(),
            "lexer diagnostics: {:?}",
            lex.diagnostics
        );
        let parsed = parser::parse_tokens(file_id, lex);
        assert!(
            parsed.diagnostics.is_empty(),
            "parser diagnostics: {:?}",
            parsed.diagnostics
        );
        let module = parsed.module.expect("parser should produce a module");
        let checked = typecheck(&module, CompileTarget::Wasm, BuildProfile::Debug, None);
        assert!(
            checked.diagnostics.is_empty(),
            "typecheck diagnostics: {:?}",
            checked.diagnostics
        );
        checked
    }

    fn typecheck_source_with_path(path: &str, source: &str) -> TypeCheckResult {
        let mut source_map = SourceMap::new();
        let file_id = source_map.add(path, String::from(source));
        let lex = lexer::lex(file_id, source);
        assert!(
            lex.diagnostics.is_empty(),
            "lexer diagnostics: {:?}",
            lex.diagnostics
        );
        let parsed = parser::parse_tokens(file_id, lex);
        assert!(
            parsed.diagnostics.is_empty(),
            "parser diagnostics: {:?}",
            parsed.diagnostics
        );
        let module = parsed.module.expect("parser should produce a module");
        let checked = typecheck(
            &module,
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(&source_map),
        );
        assert!(
            checked.diagnostics.is_empty(),
            "typecheck diagnostics: {:?}",
            checked.diagnostics
        );
        checked
    }

    fn test_link_symbol(name: &str) -> PublicCallableLinkSymbol {
        PublicCallableLinkSymbol {
            source_path: String::from("project/core/test.nepl"),
            name: String::from(name),
            signature_hash: 1,
        }
    }

    /// `.neplmeta` structured surface は public signature text と別に、materializer が
    /// `TypeCtx` / `Env` を再構築するための形を保持する。body-only edit では変わらず、
    /// callable result type の変更では変わることを固定する。
    #[test]
    fn typed_public_surface_hash_tracks_structured_callable_boundary() {
        let first = typecheck_source("pub fn answer %fn unit i32 \\unit:\n    1\n");
        let body_edit = typecheck_source("pub fn answer %fn unit i32 \\unit:\n    2\n");
        let type_edit = typecheck_source("pub fn answer %fn unit unit \\unit:\n    unit\n");

        assert_eq!(
            first.public_surface.stable_hash,
            body_edit.public_surface.stable_hash
        );
        assert_ne!(
            first.public_surface.stable_hash,
            type_edit.public_surface.stable_hash
        );
        assert!(first.public_surface.entries.iter().any(|entry| {
            matches!(
                &entry.surface,
                PublicSurfaceShape::Callable(callable)
                    if entry.name == "answer"
                        && matches!(
                            &callable.ty,
                            PublicTypeTerm::Function { result, .. }
                                if matches!(result.as_ref(), PublicTypeTerm::I32)
                        )
            )
        }));
    }

    /// `\unit` は値引数ではなく 0 引数 lambda の表層記法であり、
    /// `%fn unit i32` も 0 引数 function type として正規化される。
    /// `.neplmeta` はこの canonical surface を保存し、旧 `()` 表記から
    /// `unit` keyword へ移した後も callable arity と型 boundary を崩さない。
    #[test]
    fn typed_public_surface_keeps_nullary_unit_callable_arity_separate_from_type_shape() {
        let checked = typecheck_source("pub fn answer %fn unit i32 \\unit:\n    1\n");
        let callable = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| {
                entry.kind == TypedPublicSignatureKind::Callable && entry.name == "answer"
            })
            .and_then(|entry| match &entry.surface {
                PublicSurfaceShape::Callable(callable) => Some(callable),
                _ => None,
            })
            .expect("answer callable surface");

        assert_eq!(callable.arity, 0);
        match &callable.ty {
            PublicTypeTerm::Function {
                params,
                result,
                effect,
                ..
            } => {
                assert!(params.is_empty());
                assert!(matches!(result.as_ref(), PublicTypeTerm::I32));
                assert_eq!(*effect, PublicEffect::Pure);
            }
            other => panic!("unexpected callable type surface: {:?}", other),
        }
    }

    /// SourceMap がある public callable surface は、span-derived symbol ではなく
    /// source path と signature hash から作る stable link symbol を持つ。body-only edit では
    /// 変わらず、signature edit では変わるので、materializer は Span を authority にしない。
    #[test]
    fn typed_public_surface_keeps_stable_callable_link_symbol() {
        let first = typecheck_source_with_path(
            "project/core/math.nepl",
            "pub fn answer %fn unit i32 \\unit:\n    1\n",
        );
        let body_edit = typecheck_source_with_path(
            "project/core/math.nepl",
            "pub fn answer %fn unit i32 \\unit:\n    2\n",
        );
        let signature_edit = typecheck_source_with_path(
            "project/core/math.nepl",
            "pub fn answer %fn unit bool \\unit:\n    true\n",
        );

        let first_symbol = first
            .public_surface
            .entries
            .iter()
            .find(|entry| {
                entry.kind == TypedPublicSignatureKind::Callable && entry.name == "answer"
            })
            .and_then(|entry| match &entry.surface {
                PublicSurfaceShape::Callable(callable) => callable.link_symbol.as_ref(),
                _ => None,
            })
            .expect("answer link symbol");
        let body_symbol = body_edit
            .public_surface
            .entries
            .iter()
            .find(|entry| {
                entry.kind == TypedPublicSignatureKind::Callable && entry.name == "answer"
            })
            .and_then(|entry| match &entry.surface {
                PublicSurfaceShape::Callable(callable) => callable.link_symbol.as_ref(),
                _ => None,
            })
            .expect("answer link symbol after body edit");
        let signature_symbol = signature_edit
            .public_surface
            .entries
            .iter()
            .find(|entry| {
                entry.kind == TypedPublicSignatureKind::Callable && entry.name == "answer"
            })
            .and_then(|entry| match &entry.surface {
                PublicSurfaceShape::Callable(callable) => callable.link_symbol.as_ref(),
                _ => None,
            })
            .expect("answer link symbol after signature edit");

        assert_eq!(first_symbol.source_path, "project/core/math.nepl");
        assert_eq!(first_symbol.name, "answer");
        assert_ne!(first_symbol.signature_hash, 0);
        assert_eq!(first_symbol, body_symbol);
        assert_ne!(first_symbol, signature_symbol);
    }

    /// 同名・同signatureの public callable でも source path が異なれば別 link symbol になる。
    /// cross-file overload disambiguation を Span に頼らず、module/source boundary で分けるための
    /// public ABI authority である。
    #[test]
    fn typed_public_surface_callable_link_symbol_distinguishes_source_paths() {
        let source = "pub fn answer %fn unit i32 \\unit:\n    1\n";
        let first = typecheck_source_with_path("project/a/math.nepl", source);
        let second = typecheck_source_with_path("project/b/math.nepl", source);

        let first_symbol = first
            .public_surface
            .entries
            .iter()
            .find(|entry| {
                entry.kind == TypedPublicSignatureKind::Callable && entry.name == "answer"
            })
            .and_then(|entry| match &entry.surface {
                PublicSurfaceShape::Callable(callable) => callable.link_symbol.as_ref(),
                _ => None,
            })
            .expect("first answer link symbol");
        let second_symbol = second
            .public_surface
            .entries
            .iter()
            .find(|entry| {
                entry.kind == TypedPublicSignatureKind::Callable && entry.name == "answer"
            })
            .and_then(|entry| match &entry.surface {
                PublicSurfaceShape::Callable(callable) => callable.link_symbol.as_ref(),
                _ => None,
            })
            .expect("second answer link symbol");

        assert_ne!(first_symbol, second_symbol);
        assert_ne!(
            first.public_surface.stable_hash,
            second.public_surface.stable_hash
        );
    }

    /// field accessor helper は普通の callable と同じ型だけでは materializer 後の Resource /
    /// SourceCapability 境界を復元できない。structured surface は `get_field` 系 helper の
    /// accessor kind を保持し、単なる user function と区別できるようにする。
    #[test]
    fn typed_public_surface_keeps_field_accessor_kind_for_callable() {
        let checked = typecheck_source_with_path(
            "project/core/field.nepl",
            "pub fn get <.T,.I,.R> %fn .T fn .I .R \\obj\\idx:\n    #intrinsic \"get_field\" <> (obj,idx)\n",
        );

        let get = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Callable && entry.name == "get")
            .expect("get public callable surface");
        let PublicSurfaceShape::Callable(callable) = &get.surface else {
            panic!("get must be a callable surface");
        };
        assert_eq!(callable.field_accessor, Some(PublicFieldAccessorKind::Get));
        assert_eq!(callable.arity, 2);
        match &callable.ty {
            PublicTypeTerm::Function { params, .. } => assert_eq!(params.len(), 2),
            other => panic!("unexpected get type surface: {:?}", other),
        }
        assert!(callable.link_symbol.is_some());
    }

    /// `#intrinsic "get_field_ref"` を使っていても、selector を固定した specialized wrapper は
    /// field accessor facade ではない。`.neplmeta` がこれを accessor として復元すると、
    /// `get_ref` の 2 引数 ABI と wrapper の 1 引数 API が衝突するため、metadata は付けない。
    #[test]
    fn typed_public_surface_does_not_mark_specialized_field_ref_wrapper_as_accessor() {
        let checked = typecheck_source_with_path(
            "project/core/mem/types.nepl",
            "pub struct RegionToken<.T>:\n    raw %i32\n    size %i32\npub fn region_token_size_ref <.T> %fn &RegionToken .T &i32 \\token:\n    #intrinsic \"get_field_ref\" <> (token,\"size\")\n",
        );

        let entry = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| {
                entry.kind == TypedPublicSignatureKind::Callable
                    && entry.name == "region_token_size_ref"
            })
            .expect("region_token_size_ref public callable surface");
        let PublicSurfaceShape::Callable(callable) = &entry.surface else {
            panic!("region_token_size_ref must be a callable surface");
        };
        assert_eq!(callable.arity, 1);
        assert_eq!(callable.field_accessor, None);
    }

    /// structured surface は field 名と型を enum/struct として保持する。stable text を
    /// 後から parse するのではなく、この payload を fresh `TypeCtx` へ投影する前提を固定する。
    #[test]
    fn typed_public_surface_keeps_struct_fields_as_terms() {
        let checked = typecheck_source("pub struct Pair:\n    left %i32\n    right %bool\n");

        let pair = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Struct && entry.name == "Pair")
            .expect("Pair public surface");
        let PublicSurfaceShape::Struct(surface) = &pair.surface else {
            panic!("Pair must be a structured struct surface");
        };
        assert_eq!(surface.fields.len(), 2);
        assert_eq!(surface.fields[0].name, "left");
        assert_eq!(surface.fields[0].ty, PublicTypeTerm::I32);
        assert_eq!(surface.fields[1].name, "right");
        assert_eq!(surface.fields[1].ty, PublicTypeTerm::Bool);
    }

    /// SourceMap がある compile では public nominal type に session-local `TypeId`
    /// ではなく source path / name / arity / definition hash から作る stable identity を
    /// 付与する。materializer はこの identity がない public nominal type を安全側で拒否できる。
    #[test]
    fn typed_public_surface_keeps_stable_nominal_identity_for_public_struct() {
        let checked = typecheck_source_with_path(
            "project/core/model.nepl",
            "pub struct Item:\n    value %i32\n",
        );

        let item = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Struct && entry.name == "Item")
            .expect("Item public surface");
        let PublicSurfaceShape::Struct(surface) = &item.surface else {
            panic!("Item must be a structured struct surface");
        };
        let identity = surface.identity.as_ref().expect("stable nominal identity");
        assert_eq!(identity.kind, PublicNominalTypeKind::Struct);
        assert_eq!(identity.source_path, "project/core/model.nepl");
        assert_eq!(identity.name, "Item");
        assert_eq!(identity.arity, 0);
        assert_ne!(identity.definition_hash, 0);
    }

    /// public field type が別の public nominal type を参照する場合も、単なる名前ではなく
    /// stable identity を一緒に保持する。同名型を別 module から materialize する段階で、
    /// `Named(String)` だけを authority にしないための境界である。
    #[test]
    fn typed_public_surface_keeps_nominal_identity_on_named_type_reference() {
        let checked = typecheck_source_with_path(
            "project/core/holder.nepl",
            "pub struct Item:\n    value %i32\npub struct Holder:\n    item %Item\n",
        );

        let holder = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Struct && entry.name == "Holder")
            .expect("Holder public surface");
        let PublicSurfaceShape::Struct(surface) = &holder.surface else {
            panic!("Holder must be a structured struct surface");
        };
        let field = surface.fields.first().expect("Holder.item field");
        let PublicTypeTerm::Named { name, identity } = &field.ty else {
            panic!("Holder.item must be a named type term");
        };
        let identity = identity.as_ref().expect("Item stable nominal identity");
        assert_eq!(name, "Item");
        assert_eq!(identity.kind, PublicNominalTypeKind::Struct);
        assert_eq!(identity.source_path, "project/core/holder.nepl");
        assert_eq!(identity.name, "Item");
        assert_eq!(identity.arity, 0);
    }

    /// generic parameter term は binder 側の `PublicTypeParam` 全体を複製せず、
    /// binder depth と index だけで参照する。これにより、同名 generic parameter を
    /// materializer が名前だけで誤って対応付ける経路を閉じる。
    #[test]
    fn typed_public_surface_uses_binder_indexed_refs_for_struct_generic_fields() {
        let checked = typecheck_source("pub struct Box<.T>:\n    value %.T\n");

        let boxed = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Struct && entry.name == "Box")
            .expect("Box public surface");
        let PublicSurfaceShape::Struct(surface) = &boxed.surface else {
            panic!("Box must be a structured struct surface");
        };
        assert_eq!(surface.type_params.len(), 1);
        assert_eq!(surface.fields.len(), 1);
        assert_eq!(
            surface.fields[0].ty,
            PublicTypeTerm::GenericParam(PublicTypeParamRef {
                binder_depth: 0,
                index: 0,
            })
        );
    }

    /// callable surface の root function generic も binder-indexed ref として保持する。
    /// bounds や materializer は `Function.type_params[0]` と term 内の `.T` を
    /// `binder_depth=0,index=0` で対応付けられる。
    #[test]
    fn typed_public_surface_uses_binder_indexed_refs_for_callable_generics() {
        let checked = typecheck_source("pub fn id <.T> %fn .T .T \\x:\n    x\n");

        let id = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Callable && entry.name == "id")
            .expect("id public surface");
        let PublicSurfaceShape::Callable(callable) = &id.surface else {
            panic!("id must be a callable surface");
        };
        let PublicTypeTerm::Function {
            type_params,
            params,
            result,
            ..
        } = &callable.ty
        else {
            panic!("id must have a function type surface");
        };
        assert_eq!(type_params.len(), 1);
        assert_eq!(
            params[0],
            PublicTypeTerm::GenericParam(PublicTypeParamRef {
                binder_depth: 0,
                index: 0,
            })
        );
        assert_eq!(
            result.as_ref(),
            &PublicTypeTerm::GenericParam(PublicTypeParamRef {
                binder_depth: 0,
                index: 0,
            })
        );
    }

    /// callable の trait bound は function type の外側にある surface field だが、
    /// 対象 type parameter は root function binder の depth/index を参照する。
    /// bounds だけ名前解決へ戻ると materializer が同名 generic を誤束縛するため、
    /// bound target も binder-indexed ref として固定する。
    #[test]
    fn typed_public_surface_uses_binder_indexed_refs_for_callable_bounds() {
        let checked = typecheck_source(
            "trait Show:\n    fn show %fn Self i32 \\x:\n        0\npub fn call_show <.T: Show> %fn .T i32 \\x:\n    0\n",
        );

        let call_show = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| {
                entry.kind == TypedPublicSignatureKind::Callable && entry.name == "call_show"
            })
            .expect("call_show public surface");
        let PublicSurfaceShape::Callable(callable) = &call_show.surface else {
            panic!("call_show must be a callable surface");
        };
        assert_eq!(callable.type_param_bounds.len(), 1);
        assert_eq!(
            callable.type_param_bounds[0].param,
            PublicTypeParamBoundTarget::Ref(PublicTypeParamRef {
                binder_depth: 0,
                index: 0,
            })
        );
        assert_eq!(callable.type_param_bounds[0].bounds.len(), 1);
        assert_eq!(callable.type_param_bounds[0].bounds[0].name, "Show");
        assert!(callable.type_param_bounds[0].bounds[0].identity.is_none());
    }

    /// SourceMap がある compile では trait definition と trait bound reference に
    /// stable identity を付与する。materializer は trait 名だけではなく、この identity を
    /// authority として使うことで、別 module の同名 trait への誤対応を避けられる。
    #[test]
    fn typed_public_surface_keeps_stable_trait_identity_on_trait_refs() {
        let checked = typecheck_source_with_path(
            "project/core/show.nepl",
            "pub trait Show:\n    fn show %fn Self i32 \\x:\n        0\npub fn call_show <.T: Show> %fn .T i32 \\x:\n    0\n",
        );

        let show = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Trait && entry.name == "Show")
            .expect("Show public trait surface");
        let PublicSurfaceShape::Trait(show_surface) = &show.surface else {
            panic!("Show must be a structured trait surface");
        };
        let trait_identity = show_surface
            .identity
            .as_ref()
            .expect("Show stable trait identity");
        assert_eq!(trait_identity.source_path, "project/core/show.nepl");
        assert_eq!(trait_identity.name, "Show");
        assert_eq!(trait_identity.arity, 0);
        assert_ne!(trait_identity.definition_hash, 0);
        let method = show_surface
            .methods
            .iter()
            .find(|method| method.name == "show")
            .expect("Show.show method surface");
        let PublicTypeTerm::Function { params, result, .. } = &method.ty else {
            panic!("Show.show must have a function type surface");
        };
        assert_eq!(params, &[PublicTypeTerm::TraitSelf]);
        assert_eq!(result.as_ref(), &PublicTypeTerm::I32);

        let call_show = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| {
                entry.kind == TypedPublicSignatureKind::Callable && entry.name == "call_show"
            })
            .expect("call_show public surface");
        let PublicSurfaceShape::Callable(callable) = &call_show.surface else {
            panic!("call_show must be a callable surface");
        };
        let bound_identity = callable.type_param_bounds[0].bounds[0]
            .identity
            .as_ref()
            .expect("Show bound stable trait identity");
        assert_eq!(bound_identity, trait_identity);
        assert!(checked
            .public_surface
            .materializer_blockers()
            .iter()
            .all(|blocker| !matches!(
                &blocker.reason,
                PublicSurfaceMaterializerBlockerReason::MissingTraitIdentity { trait_name }
                    if trait_name == "Show"
            )));
        assert!(checked
            .public_surface
            .materializer_blockers()
            .iter()
            .all(|blocker| !matches!(
                &blocker.reason,
                PublicSurfaceMaterializerBlockerReason::UnboundGenericParam { param_name }
                    if param_name == "Self"
            )));
    }

    /// public callable が private trait bound を持つ場合でも、その trait は dependency
    /// 側の型検査に必要な semantic surface である。public export としては公開しないが、
    /// `.neplmeta` には stable identity 付きで保持し、名前だけの trait lookup に戻さない。
    #[test]
    fn typed_public_surface_keeps_private_trait_bound_as_semantic_surface() {
        let checked = typecheck_source_with_path(
            "project/core/private_show.nepl",
            "trait Show:\n    fn show %fn Self i32 \\x:\n        0\npub fn call_show <.T: Show> %fn .T i32 \\x:\n    0\n",
        );

        let private_show = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Trait && entry.name == "Show")
            .expect("private Show semantic trait surface");
        assert!(!private_show.exported);
        let PublicSurfaceShape::Trait(private_show_surface) = &private_show.surface else {
            panic!("private Show must be carried as a trait semantic surface");
        };
        let trait_identity = private_show_surface
            .identity
            .as_ref()
            .expect("private Show stable trait identity");

        let call_show = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| {
                entry.kind == TypedPublicSignatureKind::Callable && entry.name == "call_show"
            })
            .expect("call_show public surface");
        let PublicSurfaceShape::Callable(callable) = &call_show.surface else {
            panic!("call_show must be a callable surface");
        };
        assert_eq!(
            callable.type_param_bounds[0].bounds[0]
                .identity
                .as_ref()
                .expect("private Show bound stable trait identity"),
            trait_identity
        );
        assert!(checked
            .public_surface
            .materializer_blockers()
            .iter()
            .all(|blocker| !matches!(
                &blocker.reason,
                PublicSurfaceMaterializerBlockerReason::MissingTraitIdentity { trait_name }
                    if trait_name == "Show"
            )));
    }

    /// private capability trait の impl は、public API ではなくても型検査 semantics に影響する。
    /// impl surface の trait application は semantic trait surface の identity を参照するため、
    /// `Copy` / `Clone` のような capability impl を artifact 境界で名前だけに落とさない。
    #[test]
    fn typed_public_surface_keeps_private_capability_impl_trait_identity() {
        let checked = typecheck_source_with_path(
            "project/core/copy_like.nepl",
            "trait Clone:\n    #capability clone\n    fn clone %fn &Self Self \\x:\n        *x\ntrait Copy:\n    #capability copy\n    fn copy_mark %fn Self Self \\x:\n        x\nimpl Clone for i32:\n    fn clone %fn &i32 i32 \\x:\n        *x\nimpl Copy for i32:\n    fn copy_mark %fn i32 i32 \\x:\n        x\n",
        );

        let copy_trait = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Trait && entry.name == "Copy")
            .expect("private Copy semantic trait surface");
        assert!(!copy_trait.exported);
        let PublicSurfaceShape::Trait(copy_trait_surface) = &copy_trait.surface else {
            panic!("Copy must be carried as a trait semantic surface");
        };
        let copy_identity = copy_trait_surface
            .identity
            .as_ref()
            .expect("Copy stable trait identity");

        let copy_impl = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Impl && entry.name == "Copy")
            .expect("Copy impl surface");
        assert!(!copy_impl.exported);
        let PublicSurfaceShape::Impl(copy_impl_surface) = &copy_impl.surface else {
            panic!("Copy impl must be a structured impl surface");
        };
        let PublicImplKind::Trait { application } = &copy_impl_surface.kind else {
            panic!("Copy impl must be a trait impl surface");
        };
        assert_eq!(
            application
                .identity
                .as_ref()
                .expect("Copy impl stable trait identity"),
            copy_identity
        );
        assert!(checked
            .public_surface
            .materializer_blockers()
            .iter()
            .all(|blocker| !matches!(
                &blocker.reason,
                PublicSurfaceMaterializerBlockerReason::MissingTraitIdentity { trait_name }
                    if trait_name == "Copy" || trait_name == "Clone"
            )));
    }

    /// impl header の trait application も name だけではなく trait identity を持つ。
    /// callable bound と同じ identity 形状を使うことで、materializer は impl lookup の
    /// authority を trait 名の文字列比較へ戻さずに済む。
    #[test]
    fn typed_public_surface_keeps_stable_trait_identity_on_impl_refs() {
        let checked = typecheck_source_with_path(
            "project/core/show_impl.nepl",
            "pub trait Show:\n    fn show %fn Self i32 \\x:\n        0\nimpl Show for i32:\n    fn show %fn i32 i32 \\x:\n        x\n",
        );

        let show = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Trait && entry.name == "Show")
            .expect("Show public trait surface");
        let PublicSurfaceShape::Trait(show_surface) = &show.surface else {
            panic!("Show must be a structured trait surface");
        };
        let trait_identity = show_surface
            .identity
            .as_ref()
            .expect("Show stable trait identity");

        let impl_entry = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Impl)
            .expect("Show impl surface");
        let PublicSurfaceShape::Impl(impl_surface) = &impl_entry.surface else {
            panic!("impl entry must be a structured impl surface");
        };
        let PublicImplKind::Trait { application, .. } = &impl_surface.kind else {
            panic!("impl entry must be a trait impl surface");
        };
        assert_eq!(
            application
                .identity
                .as_ref()
                .expect("impl trait application identity"),
            trait_identity
        );
    }

    /// generic impl header は impl 自身の binder と bound を持つ。
    /// target type や bound target を名前だけで再構築すると、別 scope の `.T` と誤対応するため、
    /// `.neplmeta` surface は impl binder の depth/index を authority として保持する。
    #[test]
    fn typed_public_surface_uses_binder_indexed_refs_for_generic_impls() {
        let bounded = typecheck_source_with_path(
            "project/core/generic_impl.nepl",
            "pub trait Touch:\n    #capability clone\n    fn touch %fn &Self unit \\x:\n        unit\npub struct Holder<.T>:\n    value %.T\nimpl<.T: Touch> Touch for Holder .T:\n    fn touch %fn &Holder .T unit \\x:\n        unit\n",
        );
        let unbounded = typecheck_source_with_path(
            "project/core/generic_impl.nepl",
            "pub trait Touch:\n    #capability clone\n    fn touch %fn &Self unit \\x:\n        unit\npub struct Holder<.T>:\n    value %.T\nimpl<.T> Touch for Holder .T:\n    fn touch %fn &Holder .T unit \\x:\n        unit\n",
        );

        let impl_entry = bounded
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Impl)
            .expect("generic Touch impl surface");
        let PublicSurfaceShape::Impl(impl_surface) = &impl_entry.surface else {
            panic!("generic Touch impl must be a structured impl surface");
        };

        assert_eq!(impl_surface.type_params.len(), 1);
        assert_eq!(impl_surface.type_param_bounds.len(), 1);
        assert_eq!(
            impl_surface.type_param_bounds[0].param,
            PublicTypeParamBoundTarget::Ref(PublicTypeParamRef {
                binder_depth: 0,
                index: 0,
            })
        );
        assert_eq!(impl_surface.type_param_bounds[0].bounds[0].name, "Touch");
        assert!(impl_surface.type_param_bounds[0].bounds[0]
            .identity
            .is_some());

        let PublicTypeTerm::Apply { base, args } = &impl_surface.target else {
            panic!("generic impl target must be an applied type");
        };
        let PublicTypeTerm::Named { name, identity } = base.as_ref() else {
            panic!("generic impl target base must be a named type");
        };
        assert_eq!(name, "Holder");
        assert!(identity.is_some());
        assert_eq!(
            args.as_slice(),
            &[PublicTypeTerm::GenericParam(PublicTypeParamRef {
                binder_depth: 0,
                index: 0,
            })]
        );

        let impl_only = TypedPublicSurfaceTable::new(Vec::from([impl_entry.clone()]));
        assert!(impl_only.is_materializer_preflight_ready());
        assert_ne!(
            bounded.public_surface.stable_hash,
            unbounded.public_surface.stable_hash
        );
        assert_ne!(
            bounded.public_signatures.stable_hash,
            unbounded.public_signatures.stable_hash
        );
    }

    /// 同じ trait 名でも source path が異なれば別 identity になる。これは dependency
    /// artifact を別 module から materialize するとき、同名 trait を誤って共有しないための
    /// invalidation boundary である。
    #[test]
    fn typed_public_surface_trait_identity_distinguishes_source_paths() {
        let source = "pub trait Show:\n    fn show %fn Self i32 \\x:\n        0\n";
        let first = typecheck_source_with_path("project/a/show.nepl", source);
        let second = typecheck_source_with_path("project/b/show.nepl", source);

        assert_ne!(
            first.public_surface.stable_hash,
            second.public_surface.stable_hash
        );

        let first_identity = first
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Trait && entry.name == "Show")
            .and_then(|entry| match &entry.surface {
                PublicSurfaceShape::Trait(surface) => surface.identity.as_ref(),
                _ => None,
            })
            .expect("first Show identity");
        let second_identity = second
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Trait && entry.name == "Show")
            .and_then(|entry| match &entry.surface {
                PublicSurfaceShape::Trait(surface) => surface.identity.as_ref(),
                _ => None,
            })
            .expect("second Show identity");
        assert_ne!(first_identity, second_identity);
    }

    /// trait definition hash は method body や doc comment ではなく、method signature と
    /// capability などの public contract だけで変わる。body-only edit を `.neplmeta`
    /// invalidation として扱わない一方、signature edit は必ず検出する。
    #[test]
    fn typed_public_surface_trait_identity_tracks_trait_contract_edits_only() {
        let base = typecheck_source_with_path(
            "project/core/show.nepl",
            "/// base docs\npub trait Show:\n    fn show %fn Self i32 \\x:\n        0\n",
        );
        let body_and_doc_edit = typecheck_source_with_path(
            "project/core/show.nepl",
            "/// edited docs\npub trait Show:\n    fn show %fn Self i32 \\x:\n        1\n",
        );
        let signature_edit = typecheck_source_with_path(
            "project/core/show.nepl",
            "/// base docs\npub trait Show:\n    fn show %fn Self bool \\x:\n        true\n",
        );

        assert_eq!(
            base.public_surface.stable_hash,
            body_and_doc_edit.public_surface.stable_hash
        );
        assert_ne!(
            base.public_surface.stable_hash,
            signature_edit.public_surface.stable_hash
        );
    }

    /// trait capability は呼び出し側の所有権・clone/copy/drop 検査に影響する public
    /// contract なので、definition hash に含める。method signature が同じでも capability
    /// が変われば古い `.neplmeta` surface は再利用できない。
    #[test]
    fn typed_public_surface_trait_identity_tracks_capability_edits() {
        let without_capability = typecheck_source_with_path(
            "project/core/show.nepl",
            "pub trait Show:\n    fn show %fn Self i32 \\x:\n        0\n",
        );
        let with_capability = typecheck_source_with_path(
            "project/core/show.nepl",
            "pub trait Show:\n    #capability clone\n    fn show %fn Self i32 \\x:\n        0\n",
        );

        assert_ne!(
            without_capability.public_surface.stable_hash,
            with_capability.public_surface.stable_hash
        );
    }

    /// trait method 内の `Self` は trait type parameter ではなく trait definition 固有の
    /// implicit receiver type である。structured surface はこれを `TraitSelf` として保持し、
    /// 通常の `.T` binder と混同しない。
    #[test]
    fn typed_public_surface_keeps_trait_self_distinct_from_trait_generics() {
        let checked = typecheck_source_with_path(
            "project/core/mapper.nepl",
            "pub trait Mapper<.T>:\n    fn map %fn Self .T \\x:\n        0\n",
        );

        let mapper = checked
            .public_surface
            .entries
            .iter()
            .find(|entry| entry.kind == TypedPublicSignatureKind::Trait && entry.name == "Mapper")
            .expect("Mapper public trait surface");
        let PublicSurfaceShape::Trait(surface) = &mapper.surface else {
            panic!("Mapper must be a structured trait surface");
        };
        assert_eq!(surface.type_params.len(), 1);
        let method = surface
            .methods
            .iter()
            .find(|method| method.name == "map")
            .expect("Mapper.map method surface");
        let PublicTypeTerm::Function { params, result, .. } = &method.ty else {
            panic!("Mapper.map must have a function type surface");
        };
        assert_eq!(params, &[PublicTypeTerm::TraitSelf]);
        assert_eq!(
            result.as_ref(),
            &PublicTypeTerm::GenericParam(PublicTypeParamRef {
                binder_depth: 0,
                index: 0,
            })
        );
        assert!(checked.public_surface.materializer_blockers().is_empty());
    }

    /// nested generic function type に入ると、その function の type parameter list が
    /// depth 0 になり、外側 binder は depth 1 へ押し出される。この規則を固定して、
    /// 同名 `.T` が内外にある場合でも名前ではなく binder 位置で対応付ける。
    #[test]
    fn public_type_term_shifts_outer_generic_refs_inside_nested_generic_function() {
        let mut ctx = TypeCtx::new();
        let outer_t = ctx.fresh_var(Some(String::from(".T")));
        let inner_t = ctx.fresh_var(Some(String::from(".T")));
        let nested_fn = ctx.function(
            Vec::from([inner_t]),
            Vec::from([inner_t, outer_t]),
            outer_t,
            Effect::Pure,
        );
        let (_outer_params, outer_generics) = public_type_params(&ctx, &[outer_t]);
        let term = public_type_term(&ctx, nested_fn, &outer_generics);

        let PublicTypeTerm::Function {
            type_params,
            params,
            result,
            ..
        } = term
        else {
            panic!("nested_fn must surface as function");
        };
        assert_eq!(type_params.len(), 1);
        assert_eq!(
            params[0],
            PublicTypeTerm::GenericParam(PublicTypeParamRef {
                binder_depth: 0,
                index: 0,
            })
        );
        assert_eq!(
            params[1],
            PublicTypeTerm::GenericParam(PublicTypeParamRef {
                binder_depth: 1,
                index: 0,
            })
        );
        assert_eq!(
            result.as_ref(),
            &PublicTypeTerm::GenericParam(PublicTypeParamRef {
                binder_depth: 1,
                index: 0,
            })
        );
    }

    /// primitive だけで構成された callable surface は、materializer preflight では
    /// blocker を持たない。実際の body skip には module graph や artifact header の
    /// 検査がさらに必要だが、型 term 自体は current session へ投影できる。
    #[test]
    fn materializer_preflight_accepts_primitive_callable_surface() {
        let table = TypedPublicSurfaceTable::new(Vec::from([TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Callable,
            name: String::from("answer"),
            exported: true,
            surface: PublicSurfaceShape::Callable(PublicCallableSurface {
                ty: PublicTypeTerm::Function {
                    type_params: Vec::new(),
                    params: Vec::from([PublicTypeTerm::Unit]),
                    result: Box::new(PublicTypeTerm::I32),
                    effect: PublicEffect::Pure,
                },
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: Some(test_link_symbol("answer")),
                type_param_bounds: Vec::new(),
            }),
        }]));

        assert!(table.is_materializer_preflight_ready());
        assert!(table.materializer_blockers().is_empty());
    }

    /// stable link symbol がない callable surface は、fresh session でどの ABI symbol に
    /// 対応させるべきかを安全に判断できない。materializer preflight は型が primitive だけでも
    /// fail-closed に止める。
    #[test]
    fn materializer_preflight_rejects_callable_without_link_symbol() {
        let table = TypedPublicSurfaceTable::new(Vec::from([TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Callable,
            name: String::from("answer"),
            exported: true,
            surface: PublicSurfaceShape::Callable(PublicCallableSurface {
                ty: PublicTypeTerm::Function {
                    type_params: Vec::new(),
                    params: Vec::from([PublicTypeTerm::Unit]),
                    result: Box::new(PublicTypeTerm::I32),
                    effect: PublicEffect::Pure,
                },
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: None,
                type_param_bounds: Vec::new(),
            }),
        }]));

        let blockers = table.materializer_blockers();
        assert!(!table.is_materializer_preflight_ready());
        assert_eq!(
            blockers[0].reason,
            PublicSurfaceMaterializerBlockerReason::MissingCallableLinkSymbol {
                callable_name: String::from("answer"),
            }
        );
    }

    /// stable identity を持つ trait reference は materializer が current session の trait
    /// definition に対応付けるための authority になる。generic bound の対象も binder-indexed
    /// ref で閉じている場合、trait 名だけを理由に body skip を止める必要はない。
    #[test]
    fn materializer_preflight_accepts_stable_trait_ref_bound() {
        let param = PublicTypeParam {
            name: String::from(".T"),
            copy_cap: false,
            clone_cap: false,
            drop_cap: false,
        };
        let table = TypedPublicSurfaceTable::new(Vec::from([TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Callable,
            name: String::from("call_show"),
            exported: true,
            surface: PublicSurfaceShape::Callable(PublicCallableSurface {
                ty: PublicTypeTerm::Function {
                    type_params: Vec::from([param.clone()]),
                    params: Vec::from([PublicTypeTerm::GenericParam(PublicTypeParamRef {
                        binder_depth: 0,
                        index: 0,
                    })]),
                    result: Box::new(PublicTypeTerm::I32),
                    effect: PublicEffect::Pure,
                },
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: Some(test_link_symbol("call_show")),
                type_param_bounds: Vec::from([PublicTypeParamBounds {
                    param: PublicTypeParamBoundTarget::Ref(PublicTypeParamRef {
                        binder_depth: 0,
                        index: 0,
                    }),
                    bounds: Vec::from([PublicTraitRef {
                        name: String::from("Show"),
                        identity: Some(PublicTraitIdentity {
                            source_path: String::from("project/core/show.nepl"),
                            name: String::from("Show"),
                            arity: 0,
                            definition_hash: 1,
                        }),
                        args: Vec::new(),
                    }]),
                }]),
            }),
        }]));

        assert!(table.is_materializer_preflight_ready());
        assert!(table.materializer_blockers().is_empty());
    }

    /// name だけの nominal type は、別 module の同名型と誤対応する危険がある。
    /// materializer preflight はこれを authority として使わず、fail-closed に拒否する。
    #[test]
    fn materializer_preflight_rejects_named_type_without_identity() {
        let table = TypedPublicSurfaceTable::new(Vec::from([TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Callable,
            name: String::from("make_item"),
            exported: true,
            surface: PublicSurfaceShape::Callable(PublicCallableSurface {
                ty: PublicTypeTerm::Function {
                    type_params: Vec::new(),
                    params: Vec::from([PublicTypeTerm::Unit]),
                    result: Box::new(PublicTypeTerm::Named {
                        name: String::from("Item"),
                        identity: None,
                    }),
                    effect: PublicEffect::Pure,
                },
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: Some(test_link_symbol("make_item")),
                type_param_bounds: Vec::new(),
            }),
        }]));

        let blockers = table.materializer_blockers();
        assert!(!table.is_materializer_preflight_ready());
        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].entry_name, "make_item");
        assert_eq!(
            blockers[0].reason,
            PublicSurfaceMaterializerBlockerReason::MissingNamedTypeIdentity {
                type_name: String::from("Item"),
            }
        );
    }

    /// backend scalar は `i64` / `u64` / `f64` / `u32` のように `TypeKind::Named` で
    /// 表現されるが、ユーザー定義の名義型ではない。これらは compiler-defined scalar
    /// domain の安定名を authority として持つため、nominal identity 欠落 blocker にはしない。
    #[test]
    fn materializer_preflight_accepts_backend_scalar_named_terms() {
        let table = TypedPublicSurfaceTable::new(Vec::from([TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Callable,
            name: String::from("wide_id"),
            exported: true,
            surface: PublicSurfaceShape::Callable(PublicCallableSurface {
                ty: PublicTypeTerm::Function {
                    type_params: Vec::new(),
                    params: Vec::from([PublicTypeTerm::Named {
                        name: String::from("i64"),
                        identity: None,
                    }]),
                    result: Box::new(PublicTypeTerm::Named {
                        name: String::from("u64"),
                        identity: None,
                    }),
                    effect: PublicEffect::Pure,
                },
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: Some(test_link_symbol("wide_id")),
                type_param_bounds: Vec::new(),
            }),
        }]));

        assert!(table.is_materializer_preflight_ready());
        assert!(table.materializer_blockers().is_empty());
    }

    /// 対応 binder を確定できない generic parameter や、stable identity を持たない
    /// trait reference は materializer の推測で補ってはいけない。preflight は
    /// それぞれを blocker として列挙し、通常の source typecheck へ戻せるようにする。
    #[test]
    fn materializer_preflight_rejects_unbound_generics_and_trait_refs() {
        let param = PublicTypeParam {
            name: String::from(".T"),
            copy_cap: true,
            clone_cap: false,
            drop_cap: false,
        };
        let table = TypedPublicSurfaceTable::new(Vec::from([TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Callable,
            name: String::from("show_like"),
            exported: true,
            surface: PublicSurfaceShape::Callable(PublicCallableSurface {
                ty: PublicTypeTerm::Function {
                    type_params: Vec::from([param.clone()]),
                    params: Vec::from([PublicTypeTerm::UnboundGenericParam(param.clone())]),
                    result: Box::new(PublicTypeTerm::I32),
                    effect: PublicEffect::Pure,
                },
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: Some(test_link_symbol("show_like")),
                type_param_bounds: Vec::from([PublicTypeParamBounds {
                    param: PublicTypeParamBoundTarget::Unbound(param),
                    bounds: Vec::from([PublicTraitRef {
                        name: String::from("Show"),
                        identity: None,
                        args: Vec::new(),
                    }]),
                }]),
            }),
        }]));

        let blockers = table.materializer_blockers();
        assert!(!table.is_materializer_preflight_ready());
        assert!(blockers.iter().any(|blocker| {
            blocker.reason
                == PublicSurfaceMaterializerBlockerReason::UnboundGenericParam {
                    param_name: String::from(".T"),
                }
        }));
        assert!(blockers.iter().any(|blocker| {
            blocker.reason
                == PublicSurfaceMaterializerBlockerReason::UnboundTraitBoundTarget {
                    param_name: String::from(".T"),
                }
        }));
        assert!(blockers.iter().any(|blocker| {
            blocker.reason
                == PublicSurfaceMaterializerBlockerReason::MissingTraitIdentity {
                    trait_name: String::from("Show"),
                }
        }));
    }
}
