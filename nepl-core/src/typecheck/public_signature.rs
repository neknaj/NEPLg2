extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Visibility;
use crate::types::{TypeCtx, TypeId};

use super::env::Env;
use super::model::{EnumInfo, RestrictedStructConstructor, StructConstructorPolicy, StructInfo};
use super::signature::{function_signature_string, signature_type_string};
use super::traits::{ImplInfo, ImplKind, TraitCapability, TraitInfo};

const FNV1A64_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x100000001b3;

/// Typed public surface that can be compared across compiler sessions.
///
/// The table intentionally stores stable text and a deterministic hash instead
/// of `TypeId`, `Span`, `SourceMap`, typed HIR, or Resource IR.  Those values
/// are tied to a single typecheck arena or source-map allocation, while this
/// table is meant to become the invalidation boundary for later semantic
/// caches.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypedPublicSignatureTable {
    pub entries: Vec<TypedPublicSignatureEntry>,
    pub stable_hash: u64,
}

impl TypedPublicSignatureTable {
    pub fn new(mut entries: Vec<TypedPublicSignatureEntry>) -> Self {
        entries.sort();
        entries.dedup();
        let stable_hash = typed_public_signature_hash(&entries);
        Self {
            entries,
            stable_hash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypedPublicSignatureEntry {
    pub kind: TypedPublicSignatureKind,
    pub name: String,
    pub signature: String,
    pub no_shadow: bool,
}

impl TypedPublicSignatureEntry {
    pub fn new(
        kind: TypedPublicSignatureKind,
        name: String,
        signature: String,
        no_shadow: bool,
    ) -> Self {
        Self {
            kind,
            name,
            signature,
            no_shadow,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypedPublicSignatureKind {
    Callable,
    Struct,
    Enum,
    Trait,
    Impl,
}

fn typed_public_signature_hash(entries: &[TypedPublicSignatureEntry]) -> u64 {
    let mut hash = FNV1A64_OFFSET;
    hash_str(&mut hash, "neplg2-typed-public-signature-v1");
    for entry in entries {
        hash_str(&mut hash, entry.kind.as_str());
        hash_str(&mut hash, entry.name.as_str());
        hash_str(&mut hash, entry.signature.as_str());
        hash_bool(&mut hash, entry.no_shadow);
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

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(FNV1A64_PRIME);
    }
}

impl TypedPublicSignatureKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Callable => "callable",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Impl => "impl",
        }
    }
}

pub(super) fn build_typed_public_signature_table(
    ctx: &TypeCtx,
    env: &Env,
    structs: &BTreeMap<String, StructInfo>,
    enums: &BTreeMap<String, EnumInfo>,
    traits: &BTreeMap<String, TraitInfo>,
    impls: &[ImplInfo],
) -> TypedPublicSignatureTable {
    let mut entries = Vec::new();
    if let Some(global_scope) = env.scopes.first() {
        for binding in global_scope
            .callables
            .iter()
            .filter(|binding| binding.defined && binding.visibility == Visibility::Pub)
        {
            entries.push(TypedPublicSignatureEntry::new(
                TypedPublicSignatureKind::Callable,
                binding.name.clone(),
                function_signature_string(ctx, binding.ty),
                binding.no_shadow,
            ));
        }
    }
    for (name, info) in structs
        .iter()
        .filter(|(_, info)| info.visibility == Visibility::Pub)
    {
        entries.push(TypedPublicSignatureEntry::new(
            TypedPublicSignatureKind::Struct,
            name.clone(),
            struct_public_signature(ctx, info),
            false,
        ));
    }
    for (name, info) in enums
        .iter()
        .filter(|(_, info)| info.visibility == Visibility::Pub)
    {
        entries.push(TypedPublicSignatureEntry::new(
            TypedPublicSignatureKind::Enum,
            name.clone(),
            enum_public_signature(ctx, info),
            false,
        ));
    }
    for (name, info) in traits
        .iter()
        .filter(|(_, info)| info.visibility == Visibility::Pub)
    {
        entries.push(TypedPublicSignatureEntry::new(
            TypedPublicSignatureKind::Trait,
            name.clone(),
            trait_public_signature(ctx, info),
            false,
        ));
    }
    for impl_info in impls {
        entries.push(TypedPublicSignatureEntry::new(
            TypedPublicSignatureKind::Impl,
            impl_public_name(ctx, impl_info),
            impl_public_signature(ctx, impl_info),
            false,
        ));
    }
    TypedPublicSignatureTable::new(entries)
}

fn signature_generic_names(ctx: &TypeCtx, type_params: &[TypeId]) -> BTreeMap<TypeId, String> {
    let mut generics = BTreeMap::new();
    for (index, type_param) in type_params.iter().enumerate() {
        generics.insert(ctx.resolve_id(*type_param), format!("$T{index}"));
    }
    generics
}

fn struct_public_signature(ctx: &TypeCtx, info: &StructInfo) -> String {
    let generics = signature_generic_names(ctx, &info.type_params);
    let mut out = String::new();
    push_type_params(&mut out, ctx, &info.type_params, &generics);
    out.push_str(";fields=");
    for (index, (name, field)) in info.field_names.iter().zip(info.fields.iter()).enumerate() {
        if index > 0 {
            out.push('|');
        }
        out.push_str(name);
        out.push(':');
        out.push_str(&signature_type_string(ctx, *field, &generics));
    }
    out.push_str(";constructor=");
    out.push_str(match info.constructor_policy {
        StructConstructorPolicy::Public => "public",
        StructConstructorPolicy::RawMemoryBoundaryOnly(RestrictedStructConstructor::OwnerToken) => {
            "raw_memory_owner_token"
        }
        StructConstructorPolicy::RawMemoryBoundaryOnly(RestrictedStructConstructor::RawPointer) => {
            "raw_memory_pointer"
        }
        StructConstructorPolicy::OwnerBackedAggregateBoundaryOnly => "owner_backed_aggregate",
    });
    out
}

fn enum_public_signature(ctx: &TypeCtx, info: &EnumInfo) -> String {
    let generics = signature_generic_names(ctx, &info.type_params);
    let mut out = String::new();
    push_type_params(&mut out, ctx, &info.type_params, &generics);
    out.push_str(";variants=");
    for (index, variant) in info.variants.iter().enumerate() {
        if index > 0 {
            out.push('|');
        }
        out.push_str(&variant.name);
        if let Some(payload) = variant.payload {
            out.push(':');
            out.push_str(&signature_type_string(ctx, payload, &generics));
        }
    }
    out
}

fn trait_public_signature(ctx: &TypeCtx, info: &TraitInfo) -> String {
    let generics = signature_generic_names(ctx, &info.type_params);
    let mut out = String::new();
    push_type_params(&mut out, ctx, &info.type_params, &generics);
    out.push_str(";capabilities=");
    let mut capabilities = info
        .capabilities
        .iter()
        .map(|capability| match capability {
            TraitCapability::Copy => "Copy",
            TraitCapability::Clone => "Clone",
            TraitCapability::Drop => "Drop",
        })
        .collect::<Vec<_>>();
    capabilities.sort();
    for (index, capability) in capabilities.iter().enumerate() {
        if index > 0 {
            out.push('|');
        }
        out.push_str(capability);
    }
    out.push_str(";methods=");
    for (index, (name, method)) in info.methods.iter().enumerate() {
        if index > 0 {
            out.push('|');
        }
        out.push_str(name);
        out.push(':');
        out.push_str(&signature_type_string(ctx, *method, &generics));
    }
    out
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

fn impl_public_signature(ctx: &TypeCtx, info: &ImplInfo) -> String {
    let generics = BTreeMap::new();
    let target = signature_type_string(ctx, info.target_ty, &generics);
    match &info.kind {
        ImplKind::Inherent => format!("target={target}"),
        ImplKind::Trait {
            application,
            self_ty,
        } => {
            let trait_self = signature_type_string(ctx, *self_ty, &generics);
            format!(
                "trait={};self={};target={target}",
                application.display_name(ctx),
                trait_self
            )
        }
    }
}

fn push_type_params(
    out: &mut String,
    ctx: &TypeCtx,
    type_params: &[TypeId],
    generics: &BTreeMap<TypeId, String>,
) {
    out.push_str("params=");
    for (index, type_param) in type_params.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&signature_type_string(ctx, *type_param, generics));
    }
}
