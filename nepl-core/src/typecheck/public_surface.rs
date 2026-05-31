extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Effect, Visibility};
use crate::types::{NominalStableTypeKind, TypeCtx, TypeId, TypeKind};

use super::env::{BindingKind, Env};
use super::model::{EnumInfo, RestrictedStructConstructor, StructConstructorPolicy, StructInfo};
use super::public_signature::TypedPublicSignatureKind;
use super::signature::signature_type_string;
use super::traits::{BoundEnv, ImplInfo, ImplKind, TraitApplication, TraitCapability, TraitInfo};

const FNV1A64_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x100000001b3;

/// Arena-independent public surface used by `.neplmeta`.
///
/// This table keeps structured entries that a later materializer can project
/// into a fresh `TypeCtx` and `Env`. It intentionally avoids `TypeId`, `Span`,
/// `SourceMap`, HIR, Resource IR, and diagnostics. Those values belong to one
/// compiler session and must not become persistent artifact authority.
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
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypedPublicSurfaceEntry {
    pub kind: TypedPublicSignatureKind,
    pub name: String,
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
    pub type_param_bounds: Vec<PublicTypeParamBounds>,
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
    pub kind: PublicImplKind,
    pub target: PublicTypeTerm,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PublicImplKind {
    Inherent,
    Trait {
        application: PublicTraitRef,
        self_ty: PublicTypeTerm,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicTraitRef {
    pub name: String,
    pub args: Vec<PublicTypeTerm>,
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
pub enum PublicStructConstructorPolicy {
    Public,
    RawMemoryOwnerToken,
    RawMemoryPointer,
    OwnerBackedAggregate,
}

fn typed_public_surface_hash(entries: &[TypedPublicSurfaceEntry]) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    hash_str(&mut hash, "neplg2-typed-public-surface-v3");
    for entry in entries {
        hash_str(&mut hash, entry.kind.as_str());
        hash_str(&mut hash, entry.name.as_str());
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
            hash_public_type_term(hash, &surface.target);
            match &surface.kind {
                PublicImplKind::Inherent => hash_str(hash, "inherent"),
                PublicImplKind::Trait {
                    application,
                    self_ty,
                } => {
                    hash_str(hash, "trait");
                    hash_public_trait_ref(hash, application);
                    hash_public_type_term(hash, self_ty);
                }
            }
        }
    }
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

fn hash_public_trait_ref(hash: &mut u64, trait_ref: &PublicTraitRef) {
    hash_str(hash, trait_ref.name.as_str());
    hash_u32(hash, trait_ref.args.len() as u32);
    for arg in &trait_ref.args {
        hash_public_type_term(hash, arg);
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

fn public_nominal_type_kind_tag(kind: PublicNominalTypeKind) -> &'static str {
    match kind {
        PublicNominalTypeKind::Enum => "enum",
        PublicNominalTypeKind::Struct => "struct",
    }
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
    env: &Env,
    structs: &BTreeMap<String, StructInfo>,
    enums: &BTreeMap<String, EnumInfo>,
    traits: &BTreeMap<String, TraitInfo>,
    impls: &[ImplInfo],
) -> TypedPublicSurfaceTable {
    let mut entries = Vec::new();
    if let Some(global_scope) = env.scopes.first() {
        for binding in global_scope
            .callables
            .iter()
            .filter(|binding| binding.defined && binding.visibility == Visibility::Pub)
        {
            if let BindingKind::Func {
                effect,
                arity,
                type_param_bounds,
                ..
            } = &binding.kind
            {
                entries.push(TypedPublicSurfaceEntry {
                    kind: TypedPublicSignatureKind::Callable,
                    name: binding.name.clone(),
                    surface: PublicSurfaceShape::Callable(PublicCallableSurface {
                        ty: public_type_term(ctx, binding.ty, &BTreeMap::new()),
                        no_shadow: binding.no_shadow,
                        arity: usize_to_u32_saturating(*arity),
                        effect: public_effect_from_ast(*effect),
                        type_param_bounds: public_type_param_bounds(
                            ctx,
                            type_param_bounds,
                            &public_function_root_generics(ctx, binding.ty),
                        ),
                    }),
                });
            }
        }
    }
    for (name, info) in structs
        .iter()
        .filter(|(_, info)| info.visibility == Visibility::Pub)
    {
        entries.push(TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Struct,
            name: name.clone(),
            surface: PublicSurfaceShape::Struct(public_struct_surface(ctx, info)),
        });
    }
    for (name, info) in enums
        .iter()
        .filter(|(_, info)| info.visibility == Visibility::Pub)
    {
        entries.push(TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Enum,
            name: name.clone(),
            surface: PublicSurfaceShape::Enum(public_enum_surface(ctx, info)),
        });
    }
    for (name, info) in traits
        .iter()
        .filter(|(_, info)| info.visibility == Visibility::Pub)
    {
        entries.push(TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Trait,
            name: name.clone(),
            surface: PublicSurfaceShape::Trait(public_trait_surface(ctx, info)),
        });
    }
    for impl_info in impls {
        entries.push(TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Impl,
            name: impl_public_name(ctx, impl_info),
            surface: PublicSurfaceShape::Impl(public_impl_surface(ctx, impl_info)),
        });
    }
    TypedPublicSurfaceTable::new(entries)
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

fn public_trait_surface(ctx: &TypeCtx, info: &TraitInfo) -> PublicTraitSurface {
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
            ty: public_type_term(ctx, *method, &generics),
        })
        .collect::<Vec<_>>();
    methods.sort();
    PublicTraitSurface {
        type_params,
        capabilities,
        methods,
    }
}

fn public_impl_surface(ctx: &TypeCtx, info: &ImplInfo) -> PublicImplSurface {
    let generics = BTreeMap::new();
    PublicImplSurface {
        kind: match &info.kind {
            ImplKind::Inherent => PublicImplKind::Inherent,
            ImplKind::Trait {
                application,
                self_ty,
            } => PublicImplKind::Trait {
                application: public_trait_ref_from_application(ctx, application, &generics),
                self_ty: public_type_term(ctx, *self_ty, &generics),
            },
        },
        target: public_type_term(ctx, info.target_ty, &generics),
    }
}

fn public_type_param_bounds(
    ctx: &TypeCtx,
    bounds: &BoundEnv,
    generics: &BTreeMap<TypeId, PublicTypeParamRef>,
) -> Vec<PublicTypeParamBounds> {
    let mut out = bounds
        .iter()
        .map(|(type_param, trait_bounds)| PublicTypeParamBounds {
            param: public_type_param_bound_target(ctx, type_param.type_id(), generics),
            bounds: trait_bounds
                .iter()
                .map(|bound| public_trait_ref_from_application(ctx, &bound.application, generics))
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
    application: &TraitApplication,
    generics: &BTreeMap<TypeId, PublicTypeParamRef>,
) -> PublicTraitRef {
    PublicTraitRef {
        name: String::from(application.trait_id.as_str()),
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

fn public_type_term(
    ctx: &TypeCtx,
    ty: TypeId,
    generics: &BTreeMap<TypeId, PublicTypeParamRef>,
) -> PublicTypeTerm {
    let resolved = ctx.resolve_id(ty);
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
                .map(|item| public_type_term(ctx, *item, generics))
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
                    .map(|param| public_type_term(ctx, *param, &scoped_generics))
                    .collect(),
                result: Box::new(public_type_term(ctx, result, &scoped_generics)),
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
            base: Box::new(public_type_term(ctx, base, generics)),
            args: args
                .iter()
                .map(|arg| public_type_term(ctx, *arg, generics))
                .collect(),
        },
        TypeKind::Box(inner) => {
            PublicTypeTerm::Boxed(Box::new(public_type_term(ctx, inner, generics)))
        }
        TypeKind::Reference(inner, mutable) => PublicTypeTerm::Reference {
            inner: Box::new(public_type_term(ctx, inner, generics)),
            mutable,
        },
    }
}

fn impl_public_name(ctx: &TypeCtx, info: &ImplInfo) -> String {
    match &info.kind {
        ImplKind::Inherent => {
            let generics = BTreeMap::new();
            format!(
                "impl:{}",
                signature_type_string(ctx, info.target_ty, &generics)
            )
        }
        ImplKind::Trait { application, .. } => application.display_name(ctx),
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
        public_type_params, public_type_term, PublicNominalTypeKind, PublicSurfaceShape,
        PublicTypeParamBoundTarget, PublicTypeParamRef, PublicTypeTerm, TypedPublicSignatureKind,
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
}
