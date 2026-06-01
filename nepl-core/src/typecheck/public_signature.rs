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
use super::traits::{BoundEnv, ImplInfo, ImplKind, TraitApplication, TraitCapability, TraitInfo};

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
    hash_str(&mut hash, "neplg2-typed-public-signature-v2");
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
    /// `.neplmeta` の性能統計で entry kind を文字列解析なしに分類するための stable code。
    ///
    /// この値は diagnostic 表示や serialized public signature ではなく、cache fallback の
    /// 根本原因を集計するための小さい数値である。
    pub fn code(self) -> u32 {
        match self {
            Self::Callable => 1,
            Self::Struct => 2,
            Self::Enum => 3,
            Self::Trait => 4,
            Self::Impl => 5,
        }
    }

    pub(super) fn as_str(self) -> &'static str {
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
    let generics = signature_generic_names(ctx, &info.type_params);
    match &info.kind {
        ImplKind::Inherent => format!(
            "impl:{}",
            signature_type_string(ctx, info.target_ty, &generics)
        ),
        ImplKind::Trait { application, .. } => {
            trait_application_signature_name(ctx, application, &generics)
        }
    }
}

fn impl_public_signature(ctx: &TypeCtx, info: &ImplInfo) -> String {
    let generics = signature_generic_names(ctx, &info.type_params);
    let target = signature_type_string(ctx, info.target_ty, &generics);
    let mut prefix = String::new();
    push_type_params(&mut prefix, ctx, &info.type_params, &generics);
    prefix.push_str(";bounds=");
    push_bound_env(&mut prefix, ctx, &info.type_param_bounds, &generics);
    match &info.kind {
        ImplKind::Inherent => format!("{prefix};target={target}"),
        ImplKind::Trait {
            application,
            self_ty,
        } => {
            let trait_self = signature_type_string(ctx, *self_ty, &generics);
            format!(
                "{prefix};trait={};self={};target={target}",
                trait_application_signature_name(ctx, application, &generics),
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

fn push_bound_env(
    out: &mut String,
    ctx: &TypeCtx,
    bounds: &BoundEnv,
    generics: &BTreeMap<TypeId, String>,
) {
    let mut rendered = Vec::new();
    for (type_param, trait_bounds) in bounds.iter() {
        let type_param_name = signature_type_string(ctx, type_param.type_id(), generics);
        for bound in trait_bounds {
            rendered.push(format!(
                "{}:{}",
                type_param_name,
                trait_application_signature_name(ctx, &bound.application, generics)
            ));
        }
    }
    rendered.sort();
    for (index, item) in rendered.iter().enumerate() {
        if index > 0 {
            out.push('|');
        }
        out.push_str(item);
    }
}

fn trait_application_signature_name(
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

#[cfg(test)]
mod tests {
    use crate::compiler::{BuildProfile, CompileTarget};
    use crate::lexer;
    use crate::parser;
    use crate::span::FileId;
    use crate::typecheck::{typecheck, TypeCheckResult};

    use super::TypedPublicSignatureKind;

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

    /// public signature table は関数本体ではなく、公開 callable の型境界を
    /// 表す。body-only edit で hash が変わらないことを固定し、後続の
    /// semantic cache が無関係な本文差分で過剰 invalidation しない根拠にする。
    #[test]
    fn typed_public_signature_hash_ignores_function_body_only_edits() {
        let first = typecheck_source("pub fn answer %fn unit i32 \\unit:\n    1\n");
        let second = typecheck_source("pub fn answer %fn unit i32 \\unit:\n    2\n");

        assert_eq!(
            first.public_signatures.stable_hash,
            second.public_signatures.stable_hash
        );
        assert!(first.public_signatures.entries.iter().any(|entry| {
            entry.kind == TypedPublicSignatureKind::Callable && entry.name == "answer"
        }));
    }

    /// 公開 callable の型が変わる場合は同じ名前でも semantic cache を
    /// invalidation する必要がある。typed public signature hash がその差分を
    /// 観測できることを確認する。
    #[test]
    fn typed_public_signature_hash_tracks_public_callable_type_edits() {
        let returns_i32 = typecheck_source("pub fn answer %fn unit i32 \\unit:\n    1\n");
        let returns_unit = typecheck_source("pub fn answer %fn unit unit \\unit:\n    unit\n");

        assert_ne!(
            returns_i32.public_signatures.stable_hash,
            returns_unit.public_signatures.stable_hash
        );
    }
}
