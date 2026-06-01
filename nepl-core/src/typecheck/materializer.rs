extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Effect, Visibility};
use crate::backend_scalar_type::BackendScalarType;
use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::span::Span;
use crate::types::{
    EnumVariantInfo, NominalStableTypeIdentity, NominalStableTypeKind, TypeCtx, TypeId, TypeKind,
};

use super::env::{same_callable_signature_and_bounds, Binding, BindingKind, Env};
use super::model::{EnumInfo, RestrictedStructConstructor, StructConstructorPolicy, StructInfo};
use super::public_signature::TypedPublicSignatureKind;
use super::public_surface::{
    materialized_callable_symbol_for_link_symbol, public_enum_definition_hash,
    public_struct_definition_hash, public_trait_definition_hash, public_type_term_stable_hash,
    PublicCallableLinkSymbol, PublicCallableSurface, PublicEffect, PublicEnumSurface,
    PublicFieldAccessorKind, PublicImplKind, PublicImplSurface, PublicNominalTypeIdentity,
    PublicNominalTypeKind, PublicStructConstructorPolicy, PublicStructSurface, PublicSurfaceShape,
    PublicTraitCapability, PublicTraitIdentity, PublicTraitRef, PublicTraitSurface,
    PublicTypeParam, PublicTypeParamBoundTarget, PublicTypeParamBounds, PublicTypeParamRef,
    PublicTypeTerm, TypedPublicSurfaceEntry, TypedPublicSurfaceTable,
};
use super::struct_shape::StructConstructorShape;
use super::traits::{
    BoundEnv, ImplInfo, ImplKind, TraitApplication, TraitBound, TraitCapability, TraitId,
    TraitInfo, TraitStableIdentity, TypeParamId,
};
use super::FieldAccessorKind;

/// `.neplmeta` の public surface を現在 session の typecheck 環境へ投影した結果。
///
/// この値は body skip の証明ではない。公開 callable の型と ABI symbol を再利用できる
/// ところまでを示し、Resource IR proof や codegen object の再利用可否は別の artifact
/// boundary で検査する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PublicSurfaceMaterializeReport {
    pub entries_seen: usize,
    pub callables_inserted: usize,
    pub callables_skipped_existing: usize,
    pub structs_inserted: usize,
    pub structs_skipped_existing: usize,
    pub enums_inserted: usize,
    pub enums_skipped_existing: usize,
    pub traits_inserted: usize,
    pub traits_skipped_existing: usize,
    pub impls_inserted: usize,
    pub impls_skipped_existing: usize,
}

/// `.neplmeta` public surface を typecheck 環境へ戻せなかった理由。
///
/// materializer は推測で型や symbol を補わない。永続 artifact は session-local な
/// `TypeId` や `Span` を持たないため、必要な authority が足りない場合は通常の source
/// load / typecheck へ戻れるように fail-closed で拒否する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicSurfaceMaterializeReject {
    pub entry_kind: TypedPublicSignatureKind,
    pub entry_name: String,
    pub reason: PublicSurfaceMaterializeRejectReason,
}

/// public surface materializer の fail-closed な拒否分類。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicSurfaceMaterializeRejectReason {
    EntryKindMismatch {
        expected: TypedPublicSignatureKind,
        actual: TypedPublicSignatureKind,
    },
    UnsupportedSurfaceKind {
        kind: TypedPublicSignatureKind,
    },
    MissingCallableLinkSymbol,
    CallableLinkNameMismatch {
        link_name: String,
    },
    CallableTypeExpected,
    CallableArityMismatch {
        surface_arity: u32,
        parameter_count: usize,
    },
    CallableEffectMismatch {
        surface_effect: PublicEffect,
        type_effect: PublicEffect,
    },
    CallableSignatureHashMismatch {
        expected: u64,
        actual: u64,
    },
    FieldAccessorUnsupported {
        kind: PublicFieldAccessorKind,
    },
    TypeParamBoundsUnsupported,
    DuplicateLinkSymbolConflict {
        symbol: String,
    },
    NonCallableNameConflict,
    NoShadowConflict,
    UnboundGenericParam {
        binder_depth: u32,
        index: u32,
    },
    TraitSelfUnsupported,
    UnboundGenericParamTerm {
        name: String,
    },
    NamedTypeUnsupported {
        name: String,
        identity: Option<PublicNominalTypeIdentity>,
    },
    ApplyUnsupported,
    MissingNominalTypeIdentity {
        kind: TypedPublicSignatureKind,
    },
    NominalTypeIdentityKindMismatch {
        expected: PublicNominalTypeKind,
        actual: PublicNominalTypeKind,
    },
    NominalTypeIdentityNameMismatch {
        identity_name: String,
    },
    NominalTypeIdentityArityMismatch {
        identity_arity: u32,
        surface_arity: u32,
    },
    NominalDefinitionHashUnavailable,
    NominalDefinitionHashMismatch {
        expected: u64,
        actual: u64,
    },
    NominalTypeIdentityConflict {
        name: String,
    },
    DuplicateStructField {
        field_name: String,
    },
    DuplicateEnumVariant {
        variant_name: String,
    },
    MissingTraitIdentity,
    TraitIdentityNameMismatch {
        identity_name: String,
    },
    TraitIdentityArityMismatch {
        identity_arity: u32,
        surface_arity: u32,
    },
    TraitDefinitionHashMismatch {
        expected: u64,
        actual: u64,
    },
    TraitIdentityConflict {
        name: String,
    },
    DuplicateTraitMethod {
        method_name: String,
    },
    UnsupportedImplKind,
    MissingTraitRefIdentity {
        trait_name: String,
    },
    TraitRefIdentityNameMismatch {
        trait_name: String,
        identity_name: String,
    },
    TraitRefIdentityConflict {
        trait_name: String,
    },
    TraitRefArityMismatch {
        trait_name: String,
        expected: u32,
        actual: u32,
    },
    UnboundTraitBoundTarget {
        param_name: String,
    },
    DuplicateImplConflict,
    CopyImplRequiresClone,
    DropImplTargetCopy,
}

impl PublicSurfaceMaterializeRejectReason {
    pub fn diagnostic_code(&self) -> TypeDiagnosticCode {
        match self {
            Self::EntryKindMismatch { .. } => {
                TypeDiagnosticCode::PublicSurfaceMaterializerEntryKindMismatch
            }
            Self::UnsupportedSurfaceKind { .. } => {
                TypeDiagnosticCode::PublicSurfaceMaterializerUnsupportedSurfaceKind
            }
            Self::MissingCallableLinkSymbol => {
                TypeDiagnosticCode::PublicSurfaceMaterializerCallableMissingLinkSymbol
            }
            Self::CallableLinkNameMismatch { .. } => {
                TypeDiagnosticCode::PublicSurfaceMaterializerCallableLinkNameMismatch
            }
            Self::CallableTypeExpected => {
                TypeDiagnosticCode::PublicSurfaceMaterializerCallableTypeExpected
            }
            Self::CallableArityMismatch { .. } => {
                TypeDiagnosticCode::PublicSurfaceMaterializerCallableArityMismatch
            }
            Self::CallableEffectMismatch { .. } => {
                TypeDiagnosticCode::PublicSurfaceMaterializerCallableEffectMismatch
            }
            Self::CallableSignatureHashMismatch { .. } => {
                TypeDiagnosticCode::PublicSurfaceMaterializerCallableSignatureHashMismatch
            }
            Self::FieldAccessorUnsupported { .. } => {
                TypeDiagnosticCode::PublicSurfaceMaterializerFieldAccessorUnsupported
            }
            Self::TypeParamBoundsUnsupported => {
                TypeDiagnosticCode::PublicSurfaceMaterializerTypeParamBoundsUnsupported
            }
            Self::DuplicateLinkSymbolConflict { .. }
            | Self::NonCallableNameConflict
            | Self::NoShadowConflict => TypeDiagnosticCode::PublicSurfaceMaterializerConflict,
            Self::UnboundGenericParam { .. }
            | Self::UnboundGenericParamTerm { .. }
            | Self::UnboundTraitBoundTarget { .. } => {
                TypeDiagnosticCode::PublicSurfaceMaterializerGenericUnsupported
            }
            Self::TraitSelfUnsupported => {
                TypeDiagnosticCode::PublicSurfaceMaterializerTraitSelfUnsupported
            }
            Self::NamedTypeUnsupported { .. } => {
                TypeDiagnosticCode::PublicSurfaceMaterializerNamedTypeUnsupported
            }
            Self::ApplyUnsupported => TypeDiagnosticCode::PublicSurfaceMaterializerApplyUnsupported,
            Self::MissingNominalTypeIdentity { .. }
            | Self::NominalTypeIdentityKindMismatch { .. }
            | Self::NominalTypeIdentityNameMismatch { .. }
            | Self::NominalTypeIdentityArityMismatch { .. }
            | Self::NominalTypeIdentityConflict { .. } => {
                TypeDiagnosticCode::PublicSurfaceMaterializerNominalIdentityRejected
            }
            Self::NominalDefinitionHashUnavailable
            | Self::NominalDefinitionHashMismatch { .. }
            | Self::DuplicateStructField { .. }
            | Self::DuplicateEnumVariant { .. } => {
                TypeDiagnosticCode::PublicSurfaceMaterializerNominalDefinitionRejected
            }
            Self::MissingTraitIdentity
            | Self::TraitIdentityNameMismatch { .. }
            | Self::TraitIdentityArityMismatch { .. }
            | Self::TraitDefinitionHashMismatch { .. }
            | Self::TraitIdentityConflict { .. }
            | Self::DuplicateTraitMethod { .. }
            | Self::MissingTraitRefIdentity { .. }
            | Self::TraitRefIdentityNameMismatch { .. }
            | Self::TraitRefIdentityConflict { .. }
            | Self::TraitRefArityMismatch { .. } => {
                TypeDiagnosticCode::PublicSurfaceMaterializerTraitIdentityRejected
            }
            Self::UnsupportedImplKind
            | Self::DuplicateImplConflict
            | Self::CopyImplRequiresClone
            | Self::DropImplTargetCopy => TypeDiagnosticCode::PublicSurfaceMaterializerImplRejected,
        }
    }
}

/// public callable surface を `Env` に注入する内部 materializer。
///
/// この checkpoint は依存 module の body を読まずに callable 候補を復元するための
/// 最小単位である。primitive / tuple / function / generic parameter / box / reference
/// 型は復元するが、名義型、trait bound、impl lookup はまだ別 materializer の authority が
/// 必要なので拒否する。
#[allow(dead_code)]
pub(super) fn materialize_public_surface_mvp(
    ctx: &mut TypeCtx,
    env: &mut Env,
    table: &TypedPublicSurfaceTable,
    origin_span: Span,
) -> Result<PublicSurfaceMaterializeReport, PublicSurfaceMaterializeReject> {
    if let Some(entry) = table
        .entries
        .iter()
        .find(|entry| !matches!(entry.surface, PublicSurfaceShape::Callable(_)))
    {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::UnsupportedSurfaceKind { kind: entry.kind },
        ));
    }
    let mut structs = BTreeMap::new();
    let mut enums = BTreeMap::new();
    let mut traits = BTreeMap::new();
    let mut impls = Vec::new();
    materialize_public_surface_with_semantics_mvp(
        ctx,
        env,
        &mut structs,
        &mut enums,
        &mut traits,
        &mut impls,
        table,
        origin_span,
    )
}

pub(super) fn materialize_public_surface_with_semantics_mvp(
    ctx: &mut TypeCtx,
    env: &mut Env,
    structs: &mut BTreeMap<String, StructInfo>,
    enums: &mut BTreeMap<String, EnumInfo>,
    traits: &mut BTreeMap<String, TraitInfo>,
    impls: &mut Vec<ImplInfo>,
    table: &TypedPublicSurfaceTable,
    origin_span: Span,
) -> Result<PublicSurfaceMaterializeReport, PublicSurfaceMaterializeReject> {
    let checkpoint = ctx.checkpoint();
    let staged = match stage_public_surface_with_semantics(
        ctx,
        env,
        structs,
        enums,
        traits,
        impls,
        table,
        origin_span,
    ) {
        Ok(staged) => staged,
        Err(reject) => {
            ctx.rollback(checkpoint);
            return Err(reject);
        }
    };
    ctx.commit(checkpoint);
    for (name, info) in staged.structs {
        structs.insert(name, info);
    }
    for (name, info) in staged.enums {
        enums.insert(name, info);
    }
    for (name, info) in staged.traits {
        traits.insert(name, info);
    }
    if has_copy_capability_trait(traits) {
        ctx.set_copy_trait_enabled(true);
    }
    for info in staged.impls {
        register_impl_capability_target(ctx, traits, &info);
        impls.push(info);
    }
    for binding in staged.bindings {
        env.insert_global(binding);
    }
    Ok(staged.report)
}

struct PublicSurfaceStaging {
    report: PublicSurfaceMaterializeReport,
    bindings: Vec<Binding>,
    structs: BTreeMap<String, StructInfo>,
    enums: BTreeMap<String, EnumInfo>,
    traits: BTreeMap<String, TraitInfo>,
    impls: Vec<ImplInfo>,
}

fn stage_public_surface_with_semantics(
    ctx: &mut TypeCtx,
    env: &Env,
    structs: &BTreeMap<String, StructInfo>,
    enums: &BTreeMap<String, EnumInfo>,
    traits: &BTreeMap<String, TraitInfo>,
    impls: &[ImplInfo],
    table: &TypedPublicSurfaceTable,
    origin_span: Span,
) -> Result<PublicSurfaceStaging, PublicSurfaceMaterializeReject> {
    let mut report = PublicSurfaceMaterializeReport::default();
    let mut staged_bindings = Vec::new();
    let mut staged_structs = BTreeMap::new();
    let mut staged_enums = BTreeMap::new();
    let mut staged_traits = BTreeMap::new();
    let mut staged_impls = Vec::new();

    for entry in table.entries.iter() {
        match &entry.surface {
            PublicSurfaceShape::Struct(surface) => {
                predeclare_struct_surface(ctx, entry, surface)?;
            }
            PublicSurfaceShape::Enum(surface) => {
                predeclare_enum_surface(ctx, entry, surface)?;
            }
            _ => {}
        }
    }

    for entry in &table.entries {
        report.entries_seen += 1;
        match &entry.surface {
            PublicSurfaceShape::Struct(surface) => {
                match stage_struct_surface(
                    ctx,
                    env,
                    structs,
                    &staged_structs,
                    &staged_bindings,
                    entry,
                    surface,
                    origin_span,
                )? {
                    StructMaterializeOutcome::Staged {
                        name,
                        info,
                        bindings,
                    } => {
                        staged_structs.insert(name, info);
                        staged_bindings.extend(bindings);
                        report.structs_inserted += 1;
                    }
                    StructMaterializeOutcome::AlreadyPresent => {
                        report.structs_skipped_existing += 1;
                    }
                }
            }
            PublicSurfaceShape::Enum(surface) => {
                match stage_enum_surface(
                    ctx,
                    env,
                    enums,
                    &staged_enums,
                    &staged_bindings,
                    entry,
                    surface,
                    origin_span,
                )? {
                    EnumMaterializeOutcome::Staged {
                        name,
                        info,
                        bindings,
                    } => {
                        staged_enums.insert(name, info);
                        staged_bindings.extend(bindings);
                        report.enums_inserted += 1;
                    }
                    EnumMaterializeOutcome::AlreadyPresent => {
                        report.enums_skipped_existing += 1;
                    }
                }
            }
            PublicSurfaceShape::Trait(surface) => {
                match stage_trait_surface(ctx, traits, &staged_traits, entry, surface, origin_span)?
                {
                    TraitMaterializeOutcome::Staged { name, info } => {
                        staged_traits.insert(name, info);
                        report.traits_inserted += 1;
                    }
                    TraitMaterializeOutcome::AlreadyPresent => {
                        report.traits_skipped_existing += 1;
                    }
                }
            }
            PublicSurfaceShape::Callable(surface) => {
                let outcome = stage_callable_surface(
                    ctx,
                    env,
                    &staged_bindings,
                    entry,
                    surface,
                    origin_span,
                )?;
                match outcome {
                    CallableMaterializeOutcome::Staged(binding) => {
                        staged_bindings.push(binding);
                        report.callables_inserted += 1;
                    }
                    CallableMaterializeOutcome::AlreadyPresent => {
                        report.callables_skipped_existing += 1;
                    }
                }
            }
            PublicSurfaceShape::Impl(surface) => {
                match stage_impl_surface(
                    ctx,
                    traits,
                    &staged_traits,
                    impls,
                    &staged_impls,
                    entry,
                    surface,
                    origin_span,
                )? {
                    ImplMaterializeOutcome::Staged(info) => {
                        staged_impls.push(info);
                        report.impls_inserted += 1;
                    }
                    ImplMaterializeOutcome::AlreadyPresent => {
                        report.impls_skipped_existing += 1;
                    }
                }
            }
        }
    }
    if let Some(entry) = table
        .entries
        .iter()
        .find(|entry| matches!(entry.surface, PublicSurfaceShape::Impl(_)))
    {
        validate_impl_capability_contracts(
            ctx,
            traits,
            &staged_traits,
            impls,
            &staged_impls,
            entry,
        )?;
    }
    Ok(PublicSurfaceStaging {
        report,
        bindings: staged_bindings,
        structs: staged_structs,
        enums: staged_enums,
        traits: staged_traits,
        impls: staged_impls,
    })
}

enum CallableMaterializeOutcome {
    Staged(Binding),
    AlreadyPresent,
}

enum StructMaterializeOutcome {
    Staged {
        name: String,
        info: StructInfo,
        bindings: Vec<Binding>,
    },
    AlreadyPresent,
}

enum EnumMaterializeOutcome {
    Staged {
        name: String,
        info: EnumInfo,
        bindings: Vec<Binding>,
    },
    AlreadyPresent,
}

enum TraitMaterializeOutcome {
    Staged { name: String, info: TraitInfo },
    AlreadyPresent,
}

enum ImplMaterializeOutcome {
    Staged(ImplInfo),
    AlreadyPresent,
}

fn predeclare_struct_surface(
    ctx: &mut TypeCtx,
    entry: &TypedPublicSurfaceEntry,
    surface: &PublicStructSurface,
) -> Result<TypeId, PublicSurfaceMaterializeReject> {
    if entry.kind != TypedPublicSignatureKind::Struct {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::EntryKindMismatch {
                expected: TypedPublicSignatureKind::Struct,
                actual: entry.kind,
            },
        ));
    }
    let identity = required_nominal_identity(entry, surface.identity.as_ref())?;
    if identity.kind != PublicNominalTypeKind::Struct {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NominalTypeIdentityKindMismatch {
                expected: PublicNominalTypeKind::Struct,
                actual: identity.kind,
            },
        ));
    }
    validate_struct_surface_definition(entry, surface, identity)?;
    predeclare_nominal_surface(ctx, entry, identity)
}

fn predeclare_enum_surface(
    ctx: &mut TypeCtx,
    entry: &TypedPublicSurfaceEntry,
    surface: &PublicEnumSurface,
) -> Result<TypeId, PublicSurfaceMaterializeReject> {
    if entry.kind != TypedPublicSignatureKind::Enum {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::EntryKindMismatch {
                expected: TypedPublicSignatureKind::Enum,
                actual: entry.kind,
            },
        ));
    }
    let identity = required_nominal_identity(entry, surface.identity.as_ref())?;
    if identity.kind != PublicNominalTypeKind::Enum {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NominalTypeIdentityKindMismatch {
                expected: PublicNominalTypeKind::Enum,
                actual: identity.kind,
            },
        ));
    }
    validate_enum_surface_definition(entry, surface, identity)?;
    predeclare_nominal_surface(ctx, entry, identity)
}

fn required_nominal_identity<'a>(
    entry: &TypedPublicSurfaceEntry,
    identity: Option<&'a PublicNominalTypeIdentity>,
) -> Result<&'a PublicNominalTypeIdentity, PublicSurfaceMaterializeReject> {
    let Some(identity) = identity else {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::MissingNominalTypeIdentity { kind: entry.kind },
        ));
    };
    if identity.name != entry.name {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NominalTypeIdentityNameMismatch {
                identity_name: identity.name.clone(),
            },
        ));
    }
    Ok(identity)
}

fn validate_struct_surface_definition(
    entry: &TypedPublicSurfaceEntry,
    surface: &PublicStructSurface,
    identity: &PublicNominalTypeIdentity,
) -> Result<(), PublicSurfaceMaterializeReject> {
    validate_nominal_identity_arity(entry, identity.arity, surface.type_params.len())?;
    let mut field_names = BTreeSet::new();
    for field in &surface.fields {
        if !field_names.insert(field.name.clone()) {
            return Err(reject(
                entry,
                PublicSurfaceMaterializeRejectReason::DuplicateStructField {
                    field_name: field.name.clone(),
                },
            ));
        }
    }
    let Some(actual) = public_struct_definition_hash(&surface.type_params, &surface.fields) else {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NominalDefinitionHashUnavailable,
        ));
    };
    if actual != identity.definition_hash {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NominalDefinitionHashMismatch {
                expected: identity.definition_hash,
                actual,
            },
        ));
    }
    Ok(())
}

fn validate_enum_surface_definition(
    entry: &TypedPublicSurfaceEntry,
    surface: &PublicEnumSurface,
    identity: &PublicNominalTypeIdentity,
) -> Result<(), PublicSurfaceMaterializeReject> {
    validate_nominal_identity_arity(entry, identity.arity, surface.type_params.len())?;
    let mut variant_names = BTreeSet::new();
    for variant in &surface.variants {
        if !variant_names.insert(variant.name.clone()) {
            return Err(reject(
                entry,
                PublicSurfaceMaterializeRejectReason::DuplicateEnumVariant {
                    variant_name: variant.name.clone(),
                },
            ));
        }
    }
    let Some(actual) = public_enum_definition_hash(&surface.type_params, &surface.variants) else {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NominalDefinitionHashUnavailable,
        ));
    };
    if actual != identity.definition_hash {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NominalDefinitionHashMismatch {
                expected: identity.definition_hash,
                actual,
            },
        ));
    }
    Ok(())
}

fn validate_nominal_identity_arity(
    entry: &TypedPublicSurfaceEntry,
    identity_arity: u32,
    surface_arity: usize,
) -> Result<(), PublicSurfaceMaterializeReject> {
    let surface_arity = surface_arity as u32;
    if identity_arity != surface_arity {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NominalTypeIdentityArityMismatch {
                identity_arity,
                surface_arity,
            },
        ));
    }
    Ok(())
}

fn validate_materialized_nominal_kind_hash(
    ctx: &TypeCtx,
    entry: &TypedPublicSurfaceEntry,
    identity: &PublicNominalTypeIdentity,
    kind: &TypeKind,
) -> Result<(), PublicSurfaceMaterializeReject> {
    let Some(actual) = ctx.nominal_definition_hash(kind) else {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NominalDefinitionHashUnavailable,
        ));
    };
    if actual != identity.definition_hash {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NominalDefinitionHashMismatch {
                expected: identity.definition_hash,
                actual,
            },
        ));
    }
    Ok(())
}

fn predeclare_nominal_surface(
    ctx: &mut TypeCtx,
    entry: &TypedPublicSurfaceEntry,
    identity: &PublicNominalTypeIdentity,
) -> Result<TypeId, PublicSurfaceMaterializeReject> {
    let stable_identity = nominal_identity_from_public(identity);
    if let Some(existing) = existing_nominal_type(ctx, entry, &entry.name, &stable_identity)? {
        return Ok(existing);
    }
    Ok(ctx.register_named_with_stable_identity(
        entry.name.clone(),
        TypeKind::Named(entry.name.clone()),
        stable_identity,
    ))
}

fn stage_struct_surface(
    ctx: &mut TypeCtx,
    env: &Env,
    structs: &BTreeMap<String, StructInfo>,
    staged_structs: &BTreeMap<String, StructInfo>,
    staged_bindings: &[Binding],
    entry: &TypedPublicSurfaceEntry,
    surface: &PublicStructSurface,
    origin_span: Span,
) -> Result<StructMaterializeOutcome, PublicSurfaceMaterializeReject> {
    let identity = required_nominal_identity(entry, surface.identity.as_ref())?;
    if structs.contains_key(&entry.name) || staged_structs.contains_key(&entry.name) {
        if existing_nominal_type(
            ctx,
            entry,
            &entry.name,
            &nominal_identity_from_public(identity),
        )?
        .is_none()
        {
            return Err(reject(
                entry,
                PublicSurfaceMaterializeRejectReason::NominalTypeIdentityConflict {
                    name: entry.name.clone(),
                },
            ));
        }
        return Ok(StructMaterializeOutcome::AlreadyPresent);
    }
    let ty = predeclare_struct_surface(ctx, entry, surface)?;
    let mut materializer = TypeTermMaterializer::new(ctx);
    let type_params = materializer.fresh_type_params(&surface.type_params);
    materializer.push_binder(type_params.clone());
    let mut fields = Vec::new();
    let mut field_names = Vec::new();
    for field in &surface.fields {
        field_names.push(field.name.clone());
        fields.push(materializer.materialize(entry, &field.ty)?);
    }
    materializer.pop_binder();
    let kind = TypeKind::Struct {
        name: entry.name.clone(),
        type_params: type_params.clone(),
        fields: fields.clone(),
        field_names: field_names.clone(),
    };
    validate_materialized_nominal_kind_hash(materializer.ctx, entry, identity, &kind)?;
    materializer.ctx.register_named_with_stable_identity(
        entry.name.clone(),
        kind,
        nominal_identity_from_public(identity),
    );
    let constructor_shape =
        StructConstructorShape::classify(materializer.ctx, &fields, &field_names);
    let constructor_policy = struct_constructor_policy_from_public(surface.constructor_policy);
    let ret_ty = if type_params.is_empty() {
        ty
    } else {
        materializer.ctx.apply(ty, type_params.clone())
    };
    let constructor_params = constructor_shape.constructor_params(&fields);
    let constructor_ty = materializer.ctx.function(
        type_params.clone(),
        constructor_params,
        ret_ty,
        Effect::Pure,
    );
    let binding = constructor_binding(
        entry.name.clone(),
        constructor_ty,
        constructor_shape.constructor_arity(fields.len()),
        origin_span,
    );
    reject_non_callable_name_conflict(env, staged_bindings, entry, &binding)?;
    Ok(StructMaterializeOutcome::Staged {
        name: entry.name.clone(),
        info: StructInfo {
            ty,
            visibility: Visibility::Pub,
            span: origin_span,
            type_params,
            fields,
            field_names,
            constructor_shape,
            constructor_policy,
        },
        bindings: Vec::from([binding]),
    })
}

fn stage_enum_surface(
    ctx: &mut TypeCtx,
    env: &Env,
    enums: &BTreeMap<String, EnumInfo>,
    staged_enums: &BTreeMap<String, EnumInfo>,
    staged_bindings: &[Binding],
    entry: &TypedPublicSurfaceEntry,
    surface: &PublicEnumSurface,
    origin_span: Span,
) -> Result<EnumMaterializeOutcome, PublicSurfaceMaterializeReject> {
    let identity = required_nominal_identity(entry, surface.identity.as_ref())?;
    if enums.contains_key(&entry.name) || staged_enums.contains_key(&entry.name) {
        if existing_nominal_type(
            ctx,
            entry,
            &entry.name,
            &nominal_identity_from_public(identity),
        )?
        .is_none()
        {
            return Err(reject(
                entry,
                PublicSurfaceMaterializeRejectReason::NominalTypeIdentityConflict {
                    name: entry.name.clone(),
                },
            ));
        }
        return Ok(EnumMaterializeOutcome::AlreadyPresent);
    }
    let ty = predeclare_enum_surface(ctx, entry, surface)?;
    let mut materializer = TypeTermMaterializer::new(ctx);
    let type_params = materializer.fresh_type_params(&surface.type_params);
    materializer.push_binder(type_params.clone());
    let mut variants = Vec::new();
    for variant in &surface.variants {
        variants.push(EnumVariantInfo {
            name: variant.name.clone(),
            payload: match &variant.payload {
                Some(payload) => Some(materializer.materialize(entry, payload)?),
                None => None,
            },
        });
    }
    materializer.pop_binder();
    let kind = TypeKind::Enum {
        name: entry.name.clone(),
        type_params: type_params.clone(),
        variants: variants.clone(),
    };
    validate_materialized_nominal_kind_hash(materializer.ctx, entry, identity, &kind)?;
    materializer.ctx.register_named_with_stable_identity(
        entry.name.clone(),
        kind,
        nominal_identity_from_public(identity),
    );
    let mut bindings = Vec::new();
    for variant in &variants {
        let params = variant.payload.iter().copied().collect::<Vec<TypeId>>();
        let ret_ty = if type_params.is_empty() {
            ty
        } else {
            materializer.ctx.apply(ty, type_params.clone())
        };
        let func_ty = materializer
            .ctx
            .function(type_params.clone(), params, ret_ty, Effect::Pure);
        let arity = if variant.payload.is_some() { 1 } else { 0 };
        let simple = constructor_binding(variant.name.clone(), func_ty, arity, origin_span);
        reject_non_callable_name_conflict(env, staged_bindings, entry, &simple)?;
        reject_non_callable_name_conflict(env, &bindings, entry, &simple)?;
        bindings.push(simple);
        let qualified_name = format!("{}::{}", entry.name, variant.name);
        let qualified = constructor_binding(qualified_name, func_ty, arity, origin_span);
        reject_non_callable_name_conflict(env, staged_bindings, entry, &qualified)?;
        reject_non_callable_name_conflict(env, &bindings, entry, &qualified)?;
        bindings.push(qualified);
    }
    Ok(EnumMaterializeOutcome::Staged {
        name: entry.name.clone(),
        info: EnumInfo {
            ty,
            visibility: Visibility::Pub,
            span: origin_span,
            type_params,
            variants,
        },
        bindings,
    })
}

fn stage_trait_surface(
    ctx: &mut TypeCtx,
    traits: &BTreeMap<String, TraitInfo>,
    staged_traits: &BTreeMap<String, TraitInfo>,
    entry: &TypedPublicSurfaceEntry,
    surface: &PublicTraitSurface,
    origin_span: Span,
) -> Result<TraitMaterializeOutcome, PublicSurfaceMaterializeReject> {
    if entry.kind != TypedPublicSignatureKind::Trait {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::EntryKindMismatch {
                expected: TypedPublicSignatureKind::Trait,
                actual: entry.kind,
            },
        ));
    }
    let identity = required_trait_identity(entry, surface.identity.as_ref())?;
    validate_trait_surface_definition(entry, surface, identity)?;
    let stable_identity = trait_identity_from_public(identity);
    if let Some(existing) = traits.get(&entry.name) {
        if existing.stable_identity.as_ref() == Some(&stable_identity) {
            return Ok(TraitMaterializeOutcome::AlreadyPresent);
        }
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::TraitIdentityConflict {
                name: entry.name.clone(),
            },
        ));
    }
    if let Some(existing) = staged_traits.get(&entry.name) {
        if existing.stable_identity.as_ref() == Some(&stable_identity) {
            return Ok(TraitMaterializeOutcome::AlreadyPresent);
        }
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::TraitIdentityConflict {
                name: entry.name.clone(),
            },
        ));
    }
    let mut materializer = TypeTermMaterializer::new(ctx);
    let type_params = materializer.fresh_type_params(&surface.type_params);
    let self_ty = materializer.ctx.fresh_var(Some(String::from("Self")));
    materializer.push_binder(type_params.clone());
    materializer.set_trait_self(Some(self_ty));
    let mut methods = BTreeMap::new();
    for method in &surface.methods {
        methods.insert(
            method.name.clone(),
            materializer.materialize(entry, &method.ty)?,
        );
    }
    materializer.set_trait_self(None);
    materializer.pop_binder();
    Ok(TraitMaterializeOutcome::Staged {
        name: entry.name.clone(),
        info: TraitInfo {
            doc: None,
            visibility: Visibility::Pub,
            type_params,
            capabilities: surface
                .capabilities
                .iter()
                .copied()
                .map(trait_capability_from_public)
                .collect(),
            methods,
            self_ty,
            span: origin_span,
            stable_identity: Some(stable_identity),
        },
    })
}

fn stage_impl_surface(
    ctx: &mut TypeCtx,
    traits: &BTreeMap<String, TraitInfo>,
    staged_traits: &BTreeMap<String, TraitInfo>,
    impls: &[ImplInfo],
    staged_impls: &[ImplInfo],
    entry: &TypedPublicSurfaceEntry,
    surface: &PublicImplSurface,
    origin_span: Span,
) -> Result<ImplMaterializeOutcome, PublicSurfaceMaterializeReject> {
    if entry.kind != TypedPublicSignatureKind::Impl {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::EntryKindMismatch {
                expected: TypedPublicSignatureKind::Impl,
                actual: entry.kind,
            },
        ));
    }
    let mut materializer = TypeTermMaterializer::new(ctx);
    let type_params = materializer.fresh_type_params(&surface.type_params);
    materializer.push_binder(type_params.clone());
    let type_param_bounds = materialize_public_type_param_bounds(
        &mut materializer,
        traits,
        staged_traits,
        entry,
        &surface.type_param_bounds,
    )?;
    let target_ty = materializer.materialize(entry, &surface.target)?;
    let kind = match &surface.kind {
        PublicImplKind::Inherent => {
            return Err(reject(
                entry,
                PublicSurfaceMaterializeRejectReason::UnsupportedImplKind,
            ));
        }
        PublicImplKind::Trait { application } => {
            let (application, self_ty) = materialize_public_trait_application(
                &mut materializer,
                traits,
                staged_traits,
                entry,
                application,
            )?;
            ImplKind::Trait {
                application,
                self_ty,
            }
        }
    };
    materializer.pop_binder();
    let candidate = ImplInfo {
        type_params,
        type_param_bounds,
        kind,
        target_ty,
        span: origin_span,
    };
    if impls
        .iter()
        .chain(staged_impls.iter())
        .any(|imp| same_materialized_impl(materializer.ctx, imp, &candidate))
    {
        return Ok(ImplMaterializeOutcome::AlreadyPresent);
    }
    if impls
        .iter()
        .chain(staged_impls.iter())
        .any(|imp| overlapping_trait_impl(materializer.ctx, imp, &candidate))
    {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::DuplicateImplConflict,
        ));
    }
    Ok(ImplMaterializeOutcome::Staged(candidate))
}

fn same_materialized_impl(ctx: &TypeCtx, left: &ImplInfo, right: &ImplInfo) -> bool {
    same_materialized_impl_kind(ctx, &left.kind, &right.kind)
        && ctx.type_pattern_matches(left.target_ty, right.target_ty)
        && ctx.type_pattern_matches(right.target_ty, left.target_ty)
        && left.type_param_bounds.signature_equivalent(
            ctx,
            &left.type_params,
            &right.type_param_bounds,
            &right.type_params,
        )
}

fn same_materialized_impl_kind(ctx: &TypeCtx, left: &ImplKind, right: &ImplKind) -> bool {
    match (left, right) {
        (ImplKind::Inherent, ImplKind::Inherent) => true,
        (
            ImplKind::Trait {
                application: left_application,
                self_ty: left_self_ty,
            },
            ImplKind::Trait {
                application: right_application,
                self_ty: right_self_ty,
            },
        ) => {
            left_self_ty == right_self_ty
                && left_application.matches_parts(
                    ctx,
                    &right_application.trait_id,
                    &right_application.args,
                )
                && right_application.matches_parts(
                    ctx,
                    &left_application.trait_id,
                    &left_application.args,
                )
        }
        _ => false,
    }
}

fn overlapping_trait_impl(ctx: &TypeCtx, existing: &ImplInfo, candidate: &ImplInfo) -> bool {
    match &candidate.kind {
        ImplKind::Inherent => false,
        ImplKind::Trait {
            application,
            self_ty,
        } => {
            existing.matches_same_trait_impl(ctx, application, *self_ty)
                && (ctx.type_pattern_matches(existing.target_ty, candidate.target_ty)
                    || ctx.type_pattern_matches(candidate.target_ty, existing.target_ty))
        }
    }
}

fn materialize_public_type_param_bounds(
    materializer: &mut TypeTermMaterializer<'_>,
    traits: &BTreeMap<String, TraitInfo>,
    staged_traits: &BTreeMap<String, TraitInfo>,
    entry: &TypedPublicSurfaceEntry,
    bounds: &[PublicTypeParamBounds],
) -> Result<BoundEnv, PublicSurfaceMaterializeReject> {
    let mut out = BoundEnv::new();
    for bound_set in bounds {
        let target = match &bound_set.param {
            PublicTypeParamBoundTarget::Ref(param_ref) => {
                materializer.generic_param(entry, param_ref)?
            }
            PublicTypeParamBoundTarget::Unbound(param) => {
                return Err(reject(
                    entry,
                    PublicSurfaceMaterializeRejectReason::UnboundTraitBoundTarget {
                        param_name: param.name.clone(),
                    },
                ));
            }
        };
        let mut trait_bounds = Vec::new();
        for bound in &bound_set.bounds {
            let (application, trait_self_ty) = materialize_public_trait_application(
                materializer,
                traits,
                staged_traits,
                entry,
                bound,
            )?;
            trait_bounds.push(TraitBound {
                application,
                trait_self_ty,
            });
        }
        if !trait_bounds.is_empty() {
            out.insert(TypeParamId::new(target), trait_bounds);
        }
    }
    Ok(out)
}

fn materialize_public_trait_application(
    materializer: &mut TypeTermMaterializer<'_>,
    traits: &BTreeMap<String, TraitInfo>,
    staged_traits: &BTreeMap<String, TraitInfo>,
    entry: &TypedPublicSurfaceEntry,
    trait_ref: &PublicTraitRef,
) -> Result<(TraitApplication, TypeId), PublicSurfaceMaterializeReject> {
    let info = trait_info_for_ref(traits, staged_traits, entry, trait_ref)?;
    let expected_arity = info.type_params.len() as u32;
    let actual_arity = trait_ref.args.len() as u32;
    if expected_arity != actual_arity {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::TraitRefArityMismatch {
                trait_name: trait_ref.name.clone(),
                expected: expected_arity,
                actual: actual_arity,
            },
        ));
    }
    let args = materializer.materialize_list(entry, &trait_ref.args)?;
    Ok((
        TraitApplication {
            trait_id: TraitId::from_name(trait_ref.name.as_str()),
            args,
        },
        info.self_ty,
    ))
}

fn trait_info_for_ref<'a>(
    traits: &'a BTreeMap<String, TraitInfo>,
    staged_traits: &'a BTreeMap<String, TraitInfo>,
    entry: &TypedPublicSurfaceEntry,
    trait_ref: &PublicTraitRef,
) -> Result<&'a TraitInfo, PublicSurfaceMaterializeReject> {
    let Some(identity) = trait_ref.identity.as_ref() else {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::MissingTraitRefIdentity {
                trait_name: trait_ref.name.clone(),
            },
        ));
    };
    if identity.name != trait_ref.name {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::TraitRefIdentityNameMismatch {
                trait_name: trait_ref.name.clone(),
                identity_name: identity.name.clone(),
            },
        ));
    }
    let Some(info) = staged_traits
        .get(&trait_ref.name)
        .or_else(|| traits.get(&trait_ref.name))
    else {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::MissingTraitRefIdentity {
                trait_name: trait_ref.name.clone(),
            },
        ));
    };
    if info.stable_identity.as_ref() != Some(&trait_identity_from_public(identity)) {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::TraitRefIdentityConflict {
                trait_name: trait_ref.name.clone(),
            },
        ));
    }
    let actual_arity = trait_ref.args.len() as u32;
    if identity.arity != actual_arity {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::TraitRefArityMismatch {
                trait_name: trait_ref.name.clone(),
                expected: identity.arity,
                actual: actual_arity,
            },
        ));
    }
    Ok(info)
}

fn register_impl_capability_target(
    ctx: &mut TypeCtx,
    traits: &BTreeMap<String, TraitInfo>,
    info: &ImplInfo,
) {
    let Some(trait_self_ty) = info.trait_self_ty() else {
        return;
    };
    for capability in trait_capabilities_for_self_ty(ctx, traits, &BTreeMap::new(), trait_self_ty) {
        match capability {
            TraitCapability::Copy => ctx.register_copy_impl_target(info.target_ty),
            TraitCapability::Clone => ctx.register_clone_impl_target(info.target_ty),
            TraitCapability::Drop => ctx.register_drop_impl_target(info.target_ty),
        }
    }
}

fn validate_impl_capability_contracts(
    ctx: &TypeCtx,
    traits: &BTreeMap<String, TraitInfo>,
    staged_traits: &BTreeMap<String, TraitInfo>,
    impls: &[ImplInfo],
    staged_impls: &[ImplInfo],
    entry: &TypedPublicSurfaceEntry,
) -> Result<(), PublicSurfaceMaterializeReject> {
    for imp in staged_impls {
        if impl_has_capability(ctx, traits, staged_traits, imp, TraitCapability::Copy) {
            let has_clone_impl = impls.iter().chain(staged_impls.iter()).any(|candidate| {
                impl_has_capability(
                    ctx,
                    traits,
                    staged_traits,
                    candidate,
                    TraitCapability::Clone,
                ) && type_patterns_overlap(ctx, candidate.target_ty, imp.target_ty)
            });
            if !has_clone_impl {
                return Err(reject(
                    entry,
                    PublicSurfaceMaterializeRejectReason::CopyImplRequiresClone,
                ));
            }
        }
        if impl_has_capability(ctx, traits, staged_traits, imp, TraitCapability::Drop) {
            let overlaps_copy_impl = impls.iter().chain(staged_impls.iter()).any(|candidate| {
                impl_has_capability(ctx, traits, staged_traits, candidate, TraitCapability::Copy)
                    && type_patterns_overlap(ctx, candidate.target_ty, imp.target_ty)
            });
            if ctx.is_copy(imp.target_ty) || overlaps_copy_impl {
                return Err(reject(
                    entry,
                    PublicSurfaceMaterializeRejectReason::DropImplTargetCopy,
                ));
            }
        }
    }
    Ok(())
}

fn impl_has_capability(
    ctx: &TypeCtx,
    traits: &BTreeMap<String, TraitInfo>,
    staged_traits: &BTreeMap<String, TraitInfo>,
    info: &ImplInfo,
    capability: TraitCapability,
) -> bool {
    let Some(trait_self_ty) = info.trait_self_ty() else {
        return false;
    };
    trait_capabilities_for_self_ty(ctx, traits, staged_traits, trait_self_ty).contains(&capability)
}

fn trait_capabilities_for_self_ty(
    ctx: &TypeCtx,
    traits: &BTreeMap<String, TraitInfo>,
    staged_traits: &BTreeMap<String, TraitInfo>,
    self_ty: TypeId,
) -> Vec<TraitCapability> {
    let mut capabilities = Vec::new();
    for info in traits.values().chain(staged_traits.values()) {
        if ctx.resolve_id(info.self_ty) == ctx.resolve_id(self_ty) {
            for capability in info.capabilities.iter().copied() {
                if !capabilities.contains(&capability) {
                    capabilities.push(capability);
                }
            }
        }
    }
    capabilities
}

fn type_patterns_overlap(ctx: &TypeCtx, lhs: TypeId, rhs: TypeId) -> bool {
    ctx.type_pattern_matches(lhs, rhs) || ctx.type_pattern_matches(rhs, lhs)
}

fn has_copy_capability_trait(traits: &BTreeMap<String, TraitInfo>) -> bool {
    traits
        .values()
        .any(|info| info.capabilities.contains(&TraitCapability::Copy))
}

fn validate_trait_surface_definition(
    entry: &TypedPublicSurfaceEntry,
    surface: &PublicTraitSurface,
    identity: &PublicTraitIdentity,
) -> Result<(), PublicSurfaceMaterializeReject> {
    let surface_arity = surface.type_params.len() as u32;
    if identity.arity != surface_arity {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::TraitIdentityArityMismatch {
                identity_arity: identity.arity,
                surface_arity,
            },
        ));
    }
    let mut method_names = BTreeSet::new();
    for method in &surface.methods {
        if !method_names.insert(method.name.clone()) {
            return Err(reject(
                entry,
                PublicSurfaceMaterializeRejectReason::DuplicateTraitMethod {
                    method_name: method.name.clone(),
                },
            ));
        }
    }
    let actual = public_trait_definition_hash(
        &surface.type_params,
        &surface.capabilities,
        &surface.methods,
    );
    if actual != identity.definition_hash {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::TraitDefinitionHashMismatch {
                expected: identity.definition_hash,
                actual,
            },
        ));
    }
    Ok(())
}

fn required_trait_identity<'a>(
    entry: &TypedPublicSurfaceEntry,
    identity: Option<&'a PublicTraitIdentity>,
) -> Result<&'a PublicTraitIdentity, PublicSurfaceMaterializeReject> {
    let Some(identity) = identity else {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::MissingTraitIdentity,
        ));
    };
    if identity.name != entry.name {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::TraitIdentityNameMismatch {
                identity_name: identity.name.clone(),
            },
        ));
    }
    Ok(identity)
}

fn nominal_identity_from_public(identity: &PublicNominalTypeIdentity) -> NominalStableTypeIdentity {
    NominalStableTypeIdentity::new(
        match identity.kind {
            PublicNominalTypeKind::Enum => NominalStableTypeKind::Enum,
            PublicNominalTypeKind::Struct => NominalStableTypeKind::Struct,
        },
        identity.source_path.clone(),
        identity.name.clone(),
        identity.arity as usize,
        identity.definition_hash,
    )
}

fn existing_nominal_type(
    ctx: &TypeCtx,
    entry: &TypedPublicSurfaceEntry,
    lookup_name: &str,
    stable_identity: &NominalStableTypeIdentity,
) -> Result<Option<TypeId>, PublicSurfaceMaterializeReject> {
    let Some(existing) = ctx.lookup_named(lookup_name) else {
        return Ok(None);
    };
    let Some(existing_identity) = ctx.nominal_stable_identity(existing) else {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NominalTypeIdentityConflict {
                name: entry.name.clone(),
            },
        ));
    };
    if existing_identity != stable_identity {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NominalTypeIdentityConflict {
                name: entry.name.clone(),
            },
        ));
    }
    Ok(Some(existing))
}

fn trait_identity_from_public(identity: &PublicTraitIdentity) -> TraitStableIdentity {
    TraitStableIdentity {
        source_path: identity.source_path.clone(),
        name: identity.name.clone(),
        arity: identity.arity,
        definition_hash: identity.definition_hash,
    }
}

fn trait_capability_from_public(capability: PublicTraitCapability) -> TraitCapability {
    match capability {
        PublicTraitCapability::Copy => TraitCapability::Copy,
        PublicTraitCapability::Clone => TraitCapability::Clone,
        PublicTraitCapability::Drop => TraitCapability::Drop,
    }
}

fn struct_constructor_policy_from_public(
    policy: PublicStructConstructorPolicy,
) -> StructConstructorPolicy {
    match policy {
        PublicStructConstructorPolicy::Public => StructConstructorPolicy::Public,
        PublicStructConstructorPolicy::RawMemoryOwnerToken => {
            StructConstructorPolicy::RawMemoryBoundaryOnly(RestrictedStructConstructor::OwnerToken)
        }
        PublicStructConstructorPolicy::RawMemoryPointer => {
            StructConstructorPolicy::RawMemoryBoundaryOnly(RestrictedStructConstructor::RawPointer)
        }
        PublicStructConstructorPolicy::OwnerBackedAggregate => {
            StructConstructorPolicy::OwnerBackedAggregateBoundaryOnly
        }
    }
}

fn constructor_binding(name: String, ty: TypeId, arity: usize, origin_span: Span) -> Binding {
    Binding {
        name: name.clone(),
        ty,
        visibility: Visibility::Pub,
        mutable: false,
        no_shadow: false,
        defined: true,
        span: origin_span,
        kind: BindingKind::Func {
            def_id: None,
            symbol: name,
            effect: Effect::Pure,
            arity,
            builtin: None,
            field_accessor: None,
            type_param_bounds: BoundEnv::new(),
            captures: Vec::new(),
        },
    }
}

fn reject_non_callable_name_conflict(
    env: &Env,
    staged_bindings: &[Binding],
    entry: &TypedPublicSurfaceEntry,
    binding: &Binding,
) -> Result<(), PublicSurfaceMaterializeReject> {
    if env
        .lookup_all_any_defined(&binding.name)
        .iter()
        .any(|binding| !binding.kind.is_callable())
        || staged_bindings
            .iter()
            .any(|staged| staged.name == binding.name && !staged.kind.is_callable())
    {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NonCallableNameConflict,
        ));
    }
    Ok(())
}

fn stage_callable_surface(
    ctx: &mut TypeCtx,
    env: &Env,
    staged: &[Binding],
    entry: &TypedPublicSurfaceEntry,
    surface: &PublicCallableSurface,
    origin_span: Span,
) -> Result<CallableMaterializeOutcome, PublicSurfaceMaterializeReject> {
    if entry.kind != TypedPublicSignatureKind::Callable {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::EntryKindMismatch {
                expected: TypedPublicSignatureKind::Callable,
                actual: entry.kind,
            },
        ));
    }
    let Some(link_symbol) = surface.link_symbol.as_ref() else {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::MissingCallableLinkSymbol,
        ));
    };
    if link_symbol.name != entry.name {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::CallableLinkNameMismatch {
                link_name: link_symbol.name.clone(),
            },
        ));
    }
    let PublicTypeTerm::Function {
        params,
        effect: type_effect,
        ..
    } = &surface.ty
    else {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::CallableTypeExpected,
        ));
    };
    if surface.arity != params.len() as u32 {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::CallableArityMismatch {
                surface_arity: surface.arity,
                parameter_count: params.len(),
            },
        ));
    }
    if surface.effect != *type_effect {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::CallableEffectMismatch {
                surface_effect: surface.effect,
                type_effect: *type_effect,
            },
        ));
    }
    let expected_signature_hash = public_type_term_stable_hash(&surface.ty);
    if link_symbol.signature_hash != expected_signature_hash {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::CallableSignatureHashMismatch {
                expected: expected_signature_hash,
                actual: link_symbol.signature_hash,
            },
        ));
    }
    let field_accessor = surface.field_accessor.map(field_accessor_from_public);
    if let Some(field_accessor) = field_accessor {
        let expected_arity = field_accessor.argument_count();
        if surface.arity as usize != expected_arity {
            return Err(reject(
                entry,
                PublicSurfaceMaterializeRejectReason::CallableArityMismatch {
                    surface_arity: surface.arity,
                    parameter_count: expected_arity,
                },
            ));
        }
    }
    if !surface.type_param_bounds.is_empty() {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::TypeParamBoundsUnsupported,
        ));
    }

    let mut materializer = TypeTermMaterializer::new(ctx);
    let ty = materializer.materialize(entry, &surface.ty)?;
    let bounds = BoundEnv::new();
    let symbol = stable_callable_symbol(link_symbol);

    let existing_same_symbol = env.lookup_all_callables_by_symbol(symbol.as_str());
    if existing_same_symbol
        .iter()
        .any(|binding| !same_callable_signature_and_bounds(ctx, binding, ty, &bounds))
    {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::DuplicateLinkSymbolConflict { symbol },
        ));
    }
    if existing_same_symbol
        .iter()
        .any(|binding| same_callable_signature_and_bounds(ctx, binding, ty, &bounds))
    {
        return Ok(CallableMaterializeOutcome::AlreadyPresent);
    }
    if staged.iter().any(|binding| {
        matches!(
            &binding.kind,
            BindingKind::Func { symbol: staged_symbol, .. } if staged_symbol == &symbol
        ) && !same_callable_signature_and_bounds(ctx, binding, ty, &bounds)
    }) {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::DuplicateLinkSymbolConflict { symbol },
        ));
    }
    if staged.iter().any(|binding| {
        matches!(
            &binding.kind,
            BindingKind::Func { symbol: staged_symbol, .. } if staged_symbol == &symbol
        ) && same_callable_signature_and_bounds(ctx, binding, ty, &bounds)
    }) {
        return Ok(CallableMaterializeOutcome::AlreadyPresent);
    }
    if env
        .lookup_all_any_defined(&entry.name)
        .iter()
        .any(|binding| !binding.kind.is_callable())
    {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NonCallableNameConflict,
        ));
    }
    if env.lookup_all_callables(&entry.name).iter().any(|binding| {
        (binding.no_shadow || surface.no_shadow)
            && same_callable_signature_and_bounds(ctx, binding, ty, &bounds)
    }) || staged.iter().any(|binding| {
        binding.name == entry.name
            && (binding.no_shadow || surface.no_shadow)
            && same_callable_signature_and_bounds(ctx, binding, ty, &bounds)
    }) {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::NoShadowConflict,
        ));
    }

    Ok(CallableMaterializeOutcome::Staged(Binding {
        name: entry.name.clone(),
        ty,
        visibility: Visibility::Pub,
        mutable: false,
        no_shadow: surface.no_shadow,
        defined: true,
        span: origin_span,
        kind: BindingKind::Func {
            def_id: None,
            symbol,
            effect: effect_from_public(surface.effect),
            arity: surface.arity as usize,
            builtin: None,
            field_accessor,
            type_param_bounds: bounds,
            captures: Vec::new(),
        },
    }))
}

struct TypeTermMaterializer<'a> {
    ctx: &'a mut TypeCtx,
    binders: Vec<Vec<TypeId>>,
    trait_self: Option<TypeId>,
}

impl<'a> TypeTermMaterializer<'a> {
    fn new(ctx: &'a mut TypeCtx) -> Self {
        Self {
            ctx,
            binders: Vec::new(),
            trait_self: None,
        }
    }

    fn push_binder(&mut self, binder: Vec<TypeId>) {
        self.binders.push(binder);
    }

    fn pop_binder(&mut self) {
        self.binders.pop();
    }

    fn set_trait_self(&mut self, trait_self: Option<TypeId>) {
        self.trait_self = trait_self;
    }

    fn materialize(
        &mut self,
        entry: &TypedPublicSurfaceEntry,
        term: &PublicTypeTerm,
    ) -> Result<TypeId, PublicSurfaceMaterializeReject> {
        match term {
            PublicTypeTerm::Unit => Ok(self.ctx.unit()),
            PublicTypeTerm::I32 => Ok(self.ctx.i32()),
            PublicTypeTerm::U8 => Ok(self.ctx.u8()),
            PublicTypeTerm::F32 => Ok(self.ctx.f32()),
            PublicTypeTerm::Bool => Ok(self.ctx.bool()),
            PublicTypeTerm::Char => Ok(self.ctx.char()),
            PublicTypeTerm::Str => Ok(self.ctx.str()),
            PublicTypeTerm::Never => Ok(self.ctx.never()),
            PublicTypeTerm::TraitSelf => self.trait_self.ok_or_else(|| {
                reject(
                    entry,
                    PublicSurfaceMaterializeRejectReason::TraitSelfUnsupported,
                )
            }),
            PublicTypeTerm::Named { name, identity } => {
                if identity.is_none() {
                    if let Some(scalar) = BackendScalarType::from_name(name.as_str()) {
                        return Ok(scalar.type_id(self.ctx));
                    }
                }
                let Some(identity) = identity else {
                    return Err(reject(
                        entry,
                        PublicSurfaceMaterializeRejectReason::NamedTypeUnsupported {
                            name: name.clone(),
                            identity: None,
                        },
                    ));
                };
                let stable_identity = nominal_identity_from_public(identity);
                let Some(existing) =
                    existing_nominal_type(self.ctx, entry, name.as_str(), &stable_identity)?
                else {
                    return Err(reject(
                        entry,
                        PublicSurfaceMaterializeRejectReason::NamedTypeUnsupported {
                            name: name.clone(),
                            identity: Some(identity.clone()),
                        },
                    ));
                };
                Ok(existing)
            }
            PublicTypeTerm::GenericParam(param_ref) => self.generic_param(entry, param_ref),
            PublicTypeTerm::UnboundGenericParam(param) => Err(reject(
                entry,
                PublicSurfaceMaterializeRejectReason::UnboundGenericParamTerm {
                    name: param.name.clone(),
                },
            )),
            PublicTypeTerm::Tuple(items) => {
                let items = self.materialize_list(entry, items)?;
                Ok(self.ctx.tuple(items))
            }
            PublicTypeTerm::Function {
                type_params,
                params,
                result,
                effect,
            } => {
                let function_type_params = self.fresh_type_params(type_params);
                let (params, result) = if function_type_params.is_empty() {
                    let params = self.materialize_list(entry, params)?;
                    let result = self.materialize(entry, result)?;
                    (params, result)
                } else {
                    self.binders.push(function_type_params.clone());
                    let params = self.materialize_list(entry, params)?;
                    let result = self.materialize(entry, result)?;
                    self.binders.pop();
                    (params, result)
                };
                Ok(self.ctx.function(
                    function_type_params,
                    params,
                    result,
                    effect_from_public(*effect),
                ))
            }
            PublicTypeTerm::Apply { base, args } => {
                let base = self.materialize(entry, base)?;
                let args = self.materialize_list(entry, args)?;
                Ok(self.ctx.apply(base, args))
            }
            PublicTypeTerm::Boxed(inner) => {
                let inner = self.materialize(entry, inner)?;
                Ok(self.ctx.box_ty(inner))
            }
            PublicTypeTerm::Reference { inner, mutable } => {
                let inner = self.materialize(entry, inner)?;
                Ok(self.ctx.reference(inner, *mutable))
            }
        }
    }

    fn materialize_list(
        &mut self,
        entry: &TypedPublicSurfaceEntry,
        terms: &[PublicTypeTerm],
    ) -> Result<Vec<TypeId>, PublicSurfaceMaterializeReject> {
        terms
            .iter()
            .map(|term| self.materialize(entry, term))
            .collect()
    }

    fn fresh_type_params(&mut self, params: &[PublicTypeParam]) -> Vec<TypeId> {
        params
            .iter()
            .map(|param| {
                let id = self.ctx.fresh_var(Some(param.name.clone()));
                self.ctx
                    .set_var_capabilities(id, param.copy_cap, param.clone_cap, param.drop_cap);
                id
            })
            .collect()
    }

    fn generic_param(
        &self,
        entry: &TypedPublicSurfaceEntry,
        param_ref: &PublicTypeParamRef,
    ) -> Result<TypeId, PublicSurfaceMaterializeReject> {
        let binder_depth = param_ref.binder_depth as usize;
        let index = param_ref.index as usize;
        let Some(binder_index) = self.binders.len().checked_sub(1 + binder_depth) else {
            return Err(reject(
                entry,
                PublicSurfaceMaterializeRejectReason::UnboundGenericParam {
                    binder_depth: param_ref.binder_depth,
                    index: param_ref.index,
                },
            ));
        };
        self.binders[binder_index]
            .get(index)
            .copied()
            .ok_or_else(|| {
                reject(
                    entry,
                    PublicSurfaceMaterializeRejectReason::UnboundGenericParam {
                        binder_depth: param_ref.binder_depth,
                        index: param_ref.index,
                    },
                )
            })
    }
}

fn effect_from_public(effect: PublicEffect) -> Effect {
    match effect {
        PublicEffect::Pure => Effect::Pure,
        PublicEffect::Impure => Effect::Impure,
    }
}

fn field_accessor_from_public(kind: PublicFieldAccessorKind) -> FieldAccessorKind {
    match kind {
        PublicFieldAccessorKind::Get => FieldAccessorKind::Get,
        PublicFieldAccessorKind::GetRef => FieldAccessorKind::GetRef,
        PublicFieldAccessorKind::Put => FieldAccessorKind::Put,
    }
}

fn stable_callable_symbol(symbol: &PublicCallableLinkSymbol) -> String {
    materialized_callable_symbol_for_link_symbol(symbol)
}

fn reject(
    entry: &TypedPublicSurfaceEntry,
    reason: PublicSurfaceMaterializeRejectReason,
) -> PublicSurfaceMaterializeReject {
    PublicSurfaceMaterializeReject {
        entry_kind: entry.kind,
        entry_name: entry.name.clone(),
        reason,
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::collections::BTreeMap;
    use alloc::string::String;
    use alloc::vec::Vec;

    use crate::ast::Visibility;
    use crate::diagnostic_codes::TypeDiagnosticCode;
    use crate::span::Span;
    use crate::typecheck::materializer::{
        materialize_public_surface_mvp, materialize_public_surface_with_semantics_mvp,
        PublicSurfaceMaterializeRejectReason,
    };
    use crate::typecheck::public_signature::TypedPublicSignatureKind;
    use crate::typecheck::public_surface::{
        PublicCallableLinkSymbol, PublicCallableSurface, PublicEffect, PublicEnumSurface,
        PublicEnumVariantSurface, PublicFieldAccessorKind, PublicFieldSurface, PublicImplKind,
        PublicImplSurface, PublicNominalTypeIdentity, PublicNominalTypeKind,
        PublicStructConstructorPolicy, PublicStructSurface, PublicSurfaceShape,
        PublicTraitCapability, PublicTraitIdentity, PublicTraitMethodSurface, PublicTraitRef,
        PublicTraitSurface, PublicTypeParam, PublicTypeParamRef, PublicTypeTerm,
        TypedPublicSurfaceEntry, TypedPublicSurfaceTable,
    };
    use crate::typecheck::FieldAccessorKind;
    use crate::types::{TypeCtx, TypeKind};

    use super::super::env::{
        resolved_function_value_identity, Binding, BindingKind, Env, FunctionValueIdentityReject,
    };
    use super::{
        public_enum_definition_hash, public_struct_definition_hash, public_trait_definition_hash,
        public_type_term_stable_hash,
    };

    fn link_symbol(name: &str, ty: &PublicTypeTerm) -> PublicCallableLinkSymbol {
        link_symbol_with_path("stdlib/core/math.nepl", name, ty)
    }

    #[test]
    fn materializer_reject_reason_maps_to_stable_diagnostic_code() {
        assert_eq!(
            PublicSurfaceMaterializeRejectReason::MissingCallableLinkSymbol.diagnostic_code(),
            TypeDiagnosticCode::PublicSurfaceMaterializerCallableMissingLinkSymbol,
        );
        assert_eq!(
            PublicSurfaceMaterializeRejectReason::CallableLinkNameMismatch {
                link_name: String::from("old")
            }
            .diagnostic_code(),
            TypeDiagnosticCode::PublicSurfaceMaterializerCallableLinkNameMismatch,
        );
        assert_eq!(
            PublicSurfaceMaterializeRejectReason::CallableTypeExpected.diagnostic_code(),
            TypeDiagnosticCode::PublicSurfaceMaterializerCallableTypeExpected,
        );
        assert_eq!(
            PublicSurfaceMaterializeRejectReason::CallableArityMismatch {
                surface_arity: 1,
                parameter_count: 2,
            }
            .diagnostic_code(),
            TypeDiagnosticCode::PublicSurfaceMaterializerCallableArityMismatch,
        );
        assert_eq!(
            PublicSurfaceMaterializeRejectReason::CallableEffectMismatch {
                surface_effect: PublicEffect::Pure,
                type_effect: PublicEffect::Impure,
            }
            .diagnostic_code(),
            TypeDiagnosticCode::PublicSurfaceMaterializerCallableEffectMismatch,
        );
        assert_eq!(
            PublicSurfaceMaterializeRejectReason::CallableSignatureHashMismatch {
                expected: 1,
                actual: 2,
            }
            .diagnostic_code(),
            TypeDiagnosticCode::PublicSurfaceMaterializerCallableSignatureHashMismatch,
        );
        assert_eq!(
            PublicSurfaceMaterializeRejectReason::FieldAccessorUnsupported {
                kind: PublicFieldAccessorKind::Get,
            }
            .diagnostic_code(),
            TypeDiagnosticCode::PublicSurfaceMaterializerFieldAccessorUnsupported,
        );
        assert_eq!(
            PublicSurfaceMaterializeRejectReason::NamedTypeUnsupported {
                name: String::from("Item"),
                identity: None,
            }
            .diagnostic_code(),
            TypeDiagnosticCode::PublicSurfaceMaterializerNamedTypeUnsupported,
        );
        assert_eq!(
            PublicSurfaceMaterializeRejectReason::DuplicateImplConflict.diagnostic_code(),
            TypeDiagnosticCode::PublicSurfaceMaterializerImplRejected,
        );
    }

    fn link_symbol_with_path(
        source_path: &str,
        name: &str,
        ty: &PublicTypeTerm,
    ) -> PublicCallableLinkSymbol {
        PublicCallableLinkSymbol {
            source_path: String::from(source_path),
            name: String::from(name),
            signature_hash: public_type_term_stable_hash(ty),
        }
    }

    fn callable_entry(name: &str, surface: PublicCallableSurface) -> TypedPublicSurfaceEntry {
        TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Callable,
            name: String::from(name),
            exported: true,
            surface: PublicSurfaceShape::Callable(surface),
        }
    }

    fn nominal_identity(
        kind: PublicNominalTypeKind,
        name: &str,
        arity: u32,
        definition_hash: u64,
    ) -> PublicNominalTypeIdentity {
        PublicNominalTypeIdentity {
            kind,
            source_path: String::from("stdlib/core/data.nepl"),
            name: String::from(name),
            arity,
            definition_hash,
        }
    }

    fn struct_identity_for_fields(
        name: &str,
        type_params: &[PublicTypeParam],
        fields: &[PublicFieldSurface],
    ) -> PublicNominalTypeIdentity {
        nominal_identity(
            PublicNominalTypeKind::Struct,
            name,
            type_params.len() as u32,
            public_struct_definition_hash(type_params, fields).expect("struct hash"),
        )
    }

    fn enum_identity_for_variants(
        name: &str,
        type_params: &[PublicTypeParam],
        variants: &[PublicEnumVariantSurface],
    ) -> PublicNominalTypeIdentity {
        nominal_identity(
            PublicNominalTypeKind::Enum,
            name,
            type_params.len() as u32,
            public_enum_definition_hash(type_params, variants).expect("enum hash"),
        )
    }

    fn struct_entry(name: &str, fields: Vec<PublicFieldSurface>) -> TypedPublicSurfaceEntry {
        let type_params = Vec::new();
        let identity = struct_identity_for_fields(name, &type_params, &fields);
        TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Struct,
            name: String::from(name),
            exported: true,
            surface: PublicSurfaceShape::Struct(PublicStructSurface {
                identity: Some(identity),
                type_params,
                fields,
                constructor_policy: PublicStructConstructorPolicy::Public,
            }),
        }
    }

    fn enum_entry(name: &str, variants: Vec<PublicEnumVariantSurface>) -> TypedPublicSurfaceEntry {
        let type_params = Vec::new();
        let identity = enum_identity_for_variants(name, &type_params, &variants);
        TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Enum,
            name: String::from(name),
            exported: true,
            surface: PublicSurfaceShape::Enum(PublicEnumSurface {
                identity: Some(identity),
                type_params,
                variants,
            }),
        }
    }

    fn trait_entry(name: &str, methods: Vec<PublicTraitMethodSurface>) -> TypedPublicSurfaceEntry {
        let type_params = Vec::new();
        let capabilities = Vec::from([PublicTraitCapability::Clone]);
        let identity = trait_identity_for_contract(name, &type_params, &capabilities, &methods);
        TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Trait,
            name: String::from(name),
            exported: true,
            surface: PublicSurfaceShape::Trait(PublicTraitSurface {
                identity: Some(identity),
                type_params,
                capabilities,
                methods,
            }),
        }
    }

    fn trait_identity_for_contract(
        name: &str,
        type_params: &[PublicTypeParam],
        capabilities: &[PublicTraitCapability],
        methods: &[PublicTraitMethodSurface],
    ) -> PublicTraitIdentity {
        PublicTraitIdentity {
            source_path: String::from("stdlib/core/data.nepl"),
            name: String::from(name),
            arity: type_params.len() as u32,
            definition_hash: public_trait_definition_hash(type_params, capabilities, methods),
        }
    }

    fn primitive_answer_ty() -> PublicTypeTerm {
        PublicTypeTerm::Function {
            type_params: Vec::new(),
            params: Vec::from([PublicTypeTerm::Unit]),
            result: Box::new(PublicTypeTerm::I32),
            effect: PublicEffect::Pure,
        }
    }

    fn primitive_answer_link(name: &str) -> PublicCallableLinkSymbol {
        link_symbol(name, &primitive_answer_ty())
    }

    fn primitive_answer_surface(
        link_symbol: Option<PublicCallableLinkSymbol>,
    ) -> PublicCallableSurface {
        PublicCallableSurface {
            ty: primitive_answer_ty(),
            no_shadow: false,
            arity: 1,
            effect: PublicEffect::Pure,
            field_accessor: None,
            link_symbol,
            type_param_bounds: Vec::new(),
        }
    }

    fn field_accessor_ty(kind: PublicFieldAccessorKind) -> PublicTypeTerm {
        let field_ref = PublicTypeTerm::Reference {
            inner: Box::new(PublicTypeTerm::I32),
            mutable: false,
        };
        match kind {
            PublicFieldAccessorKind::Get => PublicTypeTerm::Function {
                type_params: Vec::new(),
                params: Vec::from([PublicTypeTerm::I32, PublicTypeTerm::Str]),
                result: Box::new(PublicTypeTerm::I32),
                effect: PublicEffect::Pure,
            },
            PublicFieldAccessorKind::GetRef => PublicTypeTerm::Function {
                type_params: Vec::new(),
                params: Vec::from([field_ref.clone(), PublicTypeTerm::Str]),
                result: Box::new(field_ref),
                effect: PublicEffect::Pure,
            },
            PublicFieldAccessorKind::Put => PublicTypeTerm::Function {
                type_params: Vec::new(),
                params: Vec::from([
                    PublicTypeTerm::I32,
                    PublicTypeTerm::Str,
                    PublicTypeTerm::I32,
                ]),
                result: Box::new(PublicTypeTerm::Unit),
                effect: PublicEffect::Pure,
            },
        }
    }

    fn field_accessor_surface(name: &str, kind: PublicFieldAccessorKind) -> PublicCallableSurface {
        let ty = field_accessor_ty(kind);
        let arity = match &ty {
            PublicTypeTerm::Function { params, .. } => params.len() as u32,
            _ => 0,
        };
        PublicCallableSurface {
            ty: ty.clone(),
            no_shadow: false,
            arity,
            effect: PublicEffect::Pure,
            field_accessor: Some(kind),
            link_symbol: Some(link_symbol(name, &ty)),
            type_param_bounds: Vec::new(),
        }
    }

    #[test]
    fn materializer_mvp_inserts_primitive_public_callable() {
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "answer",
            primitive_answer_surface(Some(primitive_answer_link("answer"))),
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        let report =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap();

        assert_eq!(report.entries_seen, 1);
        assert_eq!(report.callables_inserted, 1);
        let bindings = env.lookup_all_callables("answer");
        assert_eq!(bindings.len(), 1);
        let binding = bindings[0];
        assert_eq!(binding.visibility, Visibility::Pub);
        assert!(!binding.no_shadow);
        let BindingKind::Func {
            symbol,
            arity,
            builtin,
            field_accessor,
            type_param_bounds,
            ..
        } = &binding.kind
        else {
            panic!("materialized binding must be callable");
        };
        assert!(symbol.starts_with("neplmeta$answer$"));
        assert_eq!(*arity, 1);
        assert!(builtin.is_none());
        assert!(field_accessor.is_none());
        assert!(type_param_bounds.is_empty());
        match ctx.get(binding.ty) {
            TypeKind::Function {
                type_params,
                params,
                result,
                effect,
            } => {
                assert!(type_params.is_empty());
                assert_eq!(params, Vec::from([ctx.unit()]));
                assert_eq!(result, ctx.i32());
                assert_eq!(effect, crate::ast::Effect::Pure);
            }
            other => panic!("unexpected materialized type: {:?}", other),
        }
    }

    #[test]
    fn materializer_mvp_materializes_nominal_and_trait_definitions_before_callables() {
        let item_fields = Vec::from([PublicFieldSurface {
            name: String::from("value"),
            ty: PublicTypeTerm::I32,
        }]);
        let item_identity = struct_identity_for_fields("Item", &Vec::new(), &item_fields);
        let take_item_ty = PublicTypeTerm::Function {
            type_params: Vec::new(),
            params: Vec::from([PublicTypeTerm::Named {
                name: String::from("Item"),
                identity: Some(item_identity.clone()),
            }]),
            result: Box::new(PublicTypeTerm::Unit),
            effect: PublicEffect::Pure,
        };
        let table = TypedPublicSurfaceTable::new(Vec::from([
            callable_entry(
                "take_item",
                PublicCallableSurface {
                    ty: take_item_ty.clone(),
                    no_shadow: false,
                    arity: 1,
                    effect: PublicEffect::Pure,
                    field_accessor: None,
                    link_symbol: Some(link_symbol("take_item", &take_item_ty)),
                    type_param_bounds: Vec::new(),
                },
            ),
            struct_entry("Item", item_fields),
            enum_entry(
                "MaybeItem",
                Vec::from([
                    PublicEnumVariantSurface {
                        name: String::from("Some"),
                        payload: Some(PublicTypeTerm::Named {
                            name: String::from("Item"),
                            identity: Some(item_identity),
                        }),
                    },
                    PublicEnumVariantSurface {
                        name: String::from("None"),
                        payload: None,
                    },
                ]),
            ),
            trait_entry(
                "Show",
                Vec::from([PublicTraitMethodSurface {
                    name: String::from("show"),
                    ty: PublicTypeTerm::Function {
                        type_params: Vec::new(),
                        params: Vec::from([PublicTypeTerm::TraitSelf]),
                        result: Box::new(PublicTypeTerm::I32),
                        effect: PublicEffect::Pure,
                    },
                }]),
            ),
        ]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();
        let mut structs = BTreeMap::new();
        let mut enums = BTreeMap::new();
        let mut traits = BTreeMap::new();
        let mut impls = Vec::new();

        let report = materialize_public_surface_with_semantics_mvp(
            &mut ctx,
            &mut env,
            &mut structs,
            &mut enums,
            &mut traits,
            &mut impls,
            &table,
            Span::dummy(),
        )
        .unwrap();

        assert_eq!(report.structs_inserted, 1);
        assert_eq!(report.enums_inserted, 1);
        assert_eq!(report.traits_inserted, 1);
        assert_eq!(report.callables_inserted, 1);
        let item_ty = ctx.lookup_named("Item").expect("Item type");
        assert!(structs.contains_key("Item"));
        assert!(enums.contains_key("MaybeItem"));
        assert!(traits.contains_key("Show"));
        assert_eq!(env.lookup_all_callables("Item").len(), 1);
        assert_eq!(env.lookup_all_callables("MaybeItem::Some").len(), 1);
        assert_eq!(env.lookup_all_callables("take_item").len(), 1);
        let take_item = env.lookup_all_callables("take_item")[0];
        let TypeKind::Function { params, .. } = ctx.get(take_item.ty) else {
            panic!("take_item must be function");
        };
        assert_eq!(
            ctx.resolve_named_type_id(params[0]),
            ctx.resolve_named_type_id(item_ty)
        );
    }

    #[test]
    fn materializer_mvp_hashes_forward_nominal_placeholders_by_stable_identity() {
        let item_fields = Vec::from([PublicFieldSurface {
            name: String::from("value"),
            ty: PublicTypeTerm::I32,
        }]);
        let item_identity = struct_identity_for_fields("Item", &Vec::new(), &item_fields);
        let holder_fields = Vec::from([PublicFieldSurface {
            name: String::from("item"),
            ty: PublicTypeTerm::Named {
                name: String::from("Item"),
                identity: Some(item_identity),
            },
        }]);
        let table = TypedPublicSurfaceTable::new(Vec::from([
            struct_entry("Holder", holder_fields),
            struct_entry("Item", item_fields),
        ]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();
        let mut structs = BTreeMap::new();
        let mut enums = BTreeMap::new();
        let mut traits = BTreeMap::new();
        let mut impls = Vec::new();

        let report = materialize_public_surface_with_semantics_mvp(
            &mut ctx,
            &mut env,
            &mut structs,
            &mut enums,
            &mut traits,
            &mut impls,
            &table,
            Span::dummy(),
        )
        .unwrap();

        assert_eq!(report.structs_inserted, 2);
        let item_ty = ctx.lookup_named("Item").expect("Item type");
        let holder = structs.get("Holder").expect("Holder surface");
        assert_eq!(
            ctx.resolve_named_type_id(holder.fields[0]),
            ctx.resolve_named_type_id(item_ty)
        );
    }

    #[test]
    fn materializer_mvp_rolls_back_nominal_definitions_when_later_entry_rejects() {
        let table = TypedPublicSurfaceTable::new(Vec::from([
            struct_entry(
                "Item",
                Vec::from([PublicFieldSurface {
                    name: String::from("value"),
                    ty: PublicTypeTerm::I32,
                }]),
            ),
            callable_entry("bad", primitive_answer_surface(None)),
        ]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();
        let mut structs = BTreeMap::new();
        let mut enums = BTreeMap::new();
        let mut traits = BTreeMap::new();
        let mut impls = Vec::new();

        let reject = materialize_public_surface_with_semantics_mvp(
            &mut ctx,
            &mut env,
            &mut structs,
            &mut enums,
            &mut traits,
            &mut impls,
            &table,
            Span::dummy(),
        )
        .unwrap_err();

        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::MissingCallableLinkSymbol
        );
        assert!(ctx.lookup_named("Item").is_none());
        assert!(structs.is_empty());
        assert!(enums.is_empty());
        assert!(traits.is_empty());
        assert!(env.lookup_all_callables("Item").is_empty());
    }

    #[test]
    fn materializer_mvp_rejects_nominal_surface_whose_definition_hash_is_stale() {
        let mut item = struct_entry(
            "Item",
            Vec::from([PublicFieldSurface {
                name: String::from("value"),
                ty: PublicTypeTerm::I32,
            }]),
        );
        let expected = match &item.surface {
            PublicSurfaceShape::Struct(surface) => {
                surface.identity.as_ref().unwrap().definition_hash
            }
            _ => panic!("Item must be struct surface"),
        };
        let actual = match &mut item.surface {
            PublicSurfaceShape::Struct(surface) => {
                surface.fields.push(PublicFieldSurface {
                    name: String::from("extra"),
                    ty: PublicTypeTerm::I32,
                });
                public_struct_definition_hash(&surface.type_params, &surface.fields)
                    .expect("mutated struct hash")
            }
            _ => panic!("Item must be struct surface"),
        };
        let table = TypedPublicSurfaceTable::new(Vec::from([item]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();
        let mut structs = BTreeMap::new();
        let mut enums = BTreeMap::new();
        let mut traits = BTreeMap::new();
        let mut impls = Vec::new();

        let reject = materialize_public_surface_with_semantics_mvp(
            &mut ctx,
            &mut env,
            &mut structs,
            &mut enums,
            &mut traits,
            &mut impls,
            &table,
            Span::dummy(),
        )
        .unwrap_err();

        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::NominalDefinitionHashMismatch {
                expected,
                actual
            }
        );
        assert!(ctx.lookup_named("Item").is_none());
        assert!(structs.is_empty());
    }

    #[test]
    fn materializer_mvp_rejects_trait_surface_with_duplicate_methods() {
        let method_ty = PublicTypeTerm::Function {
            type_params: Vec::new(),
            params: Vec::from([PublicTypeTerm::TraitSelf]),
            result: Box::new(PublicTypeTerm::I32),
            effect: PublicEffect::Pure,
        };
        let table = TypedPublicSurfaceTable::new(Vec::from([trait_entry(
            "Show",
            Vec::from([
                PublicTraitMethodSurface {
                    name: String::from("show"),
                    ty: method_ty.clone(),
                },
                PublicTraitMethodSurface {
                    name: String::from("show"),
                    ty: method_ty,
                },
            ]),
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();
        let mut structs = BTreeMap::new();
        let mut enums = BTreeMap::new();
        let mut traits = BTreeMap::new();
        let mut impls = Vec::new();

        let reject = materialize_public_surface_with_semantics_mvp(
            &mut ctx,
            &mut env,
            &mut structs,
            &mut enums,
            &mut traits,
            &mut impls,
            &table,
            Span::dummy(),
        )
        .unwrap_err();

        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::DuplicateTraitMethod {
                method_name: String::from("show")
            }
        );
        assert!(traits.is_empty());
    }

    #[test]
    fn materializer_mvp_materializes_trait_impl_and_registers_capability_target() {
        let item_fields = Vec::from([PublicFieldSurface {
            name: String::from("value"),
            ty: PublicTypeTerm::I32,
        }]);
        let item_identity = struct_identity_for_fields("Item", &Vec::new(), &item_fields);
        let clone_trait = trait_entry("Clone", Vec::new());
        let clone_identity = match &clone_trait.surface {
            PublicSurfaceShape::Trait(surface) => surface.identity.as_ref().unwrap().clone(),
            _ => panic!("Clone must be trait surface"),
        };
        let table = TypedPublicSurfaceTable::new(Vec::from([
            struct_entry("Item", item_fields),
            clone_trait,
            TypedPublicSurfaceEntry {
                kind: TypedPublicSignatureKind::Impl,
                name: String::from("impl Clone for Item"),
                exported: false,
                surface: PublicSurfaceShape::Impl(PublicImplSurface {
                    source_path: String::from("stdlib/core/item.nepl"),
                    type_params: Vec::new(),
                    type_param_bounds: Vec::new(),
                    kind: PublicImplKind::Trait {
                        application: PublicTraitRef {
                            name: String::from("Clone"),
                            identity: Some(clone_identity),
                            args: Vec::new(),
                        },
                    },
                    target: PublicTypeTerm::Named {
                        name: String::from("Item"),
                        identity: Some(item_identity),
                    },
                }),
            },
        ]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();
        let mut structs = BTreeMap::new();
        let mut enums = BTreeMap::new();
        let mut traits = BTreeMap::new();
        let mut impls = Vec::new();

        let report = materialize_public_surface_with_semantics_mvp(
            &mut ctx,
            &mut env,
            &mut structs,
            &mut enums,
            &mut traits,
            &mut impls,
            &table,
            Span::dummy(),
        )
        .unwrap();

        assert_eq!(report.impls_inserted, 1);
        assert_eq!(impls.len(), 1);
        let item_ty = ctx.lookup_named("Item").expect("Item type");
        assert!(ctx.has_clone_impl_target(item_ty));
    }

    #[test]
    fn materializer_mvp_rejects_impl_trait_identity_mismatch_without_pollution() {
        let item_fields = Vec::from([PublicFieldSurface {
            name: String::from("value"),
            ty: PublicTypeTerm::I32,
        }]);
        let item_identity = struct_identity_for_fields("Item", &Vec::new(), &item_fields);
        let clone_trait = trait_entry("Clone", Vec::new());
        let wrong_identity = trait_identity_for_contract(
            "OtherClone",
            &Vec::new(),
            &[PublicTraitCapability::Clone],
            &[],
        );
        let table = TypedPublicSurfaceTable::new(Vec::from([
            struct_entry("Item", item_fields),
            clone_trait,
            TypedPublicSurfaceEntry {
                kind: TypedPublicSignatureKind::Impl,
                name: String::from("impl Clone for Item"),
                exported: false,
                surface: PublicSurfaceShape::Impl(PublicImplSurface {
                    source_path: String::from("stdlib/core/item.nepl"),
                    type_params: Vec::new(),
                    type_param_bounds: Vec::new(),
                    kind: PublicImplKind::Trait {
                        application: PublicTraitRef {
                            name: String::from("Clone"),
                            identity: Some(wrong_identity),
                            args: Vec::new(),
                        },
                    },
                    target: PublicTypeTerm::Named {
                        name: String::from("Item"),
                        identity: Some(item_identity),
                    },
                }),
            },
        ]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();
        let mut structs = BTreeMap::new();
        let mut enums = BTreeMap::new();
        let mut traits = BTreeMap::new();
        let mut impls = Vec::new();

        let reject = materialize_public_surface_with_semantics_mvp(
            &mut ctx,
            &mut env,
            &mut structs,
            &mut enums,
            &mut traits,
            &mut impls,
            &table,
            Span::dummy(),
        )
        .unwrap_err();

        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::TraitRefIdentityNameMismatch {
                trait_name: String::from("Clone"),
                identity_name: String::from("OtherClone")
            }
        );
        assert!(ctx.lookup_named("Item").is_none());
        assert!(impls.is_empty());
        assert!(structs.is_empty());
        assert!(traits.is_empty());
    }

    #[test]
    fn materializer_mvp_callables_do_not_claim_function_value_identity() {
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "answer",
            primitive_answer_surface(Some(primitive_answer_link("answer"))),
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap();

        let binding = env.lookup_all_callables("answer")[0];
        assert_eq!(
            resolved_function_value_identity(binding, binding.ty, Vec::new()),
            Err(FunctionValueIdentityReject::UnresolvedIdentity)
        );
    }

    #[test]
    fn materializer_mvp_is_idempotent_for_same_link_symbol() {
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "answer",
            primitive_answer_surface(Some(primitive_answer_link("answer"))),
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        let first =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap();
        let second =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap();

        assert_eq!(first.callables_inserted, 1);
        assert_eq!(second.callables_inserted, 0);
        assert_eq!(second.callables_skipped_existing, 1);
        assert_eq!(env.lookup_all_callables("answer").len(), 1);
    }

    #[test]
    fn materializer_mvp_rejects_missing_link_symbol() {
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "answer",
            primitive_answer_surface(None),
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();

        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::MissingCallableLinkSymbol
        );
    }

    #[test]
    fn materializer_mvp_rejects_named_type_before_nominal_materializer() {
        let identity = PublicNominalTypeIdentity {
            kind: PublicNominalTypeKind::Struct,
            source_path: String::from("stdlib/core/item.nepl"),
            name: String::from("Item"),
            arity: 0,
            definition_hash: 1,
        };
        let ty = PublicTypeTerm::Function {
            type_params: Vec::new(),
            params: Vec::from([PublicTypeTerm::Named {
                name: String::from("Item"),
                identity: Some(identity.clone()),
            }]),
            result: Box::new(PublicTypeTerm::Unit),
            effect: PublicEffect::Pure,
        };
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "take_item",
            PublicCallableSurface {
                ty: ty.clone(),
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: Some(link_symbol("take_item", &ty)),
                type_param_bounds: Vec::new(),
            },
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();

        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::NamedTypeUnsupported {
                name: String::from("Item"),
                identity: Some(identity)
            }
        );
    }

    #[test]
    fn materializer_mvp_accepts_backend_scalar_named_types() {
        let ty = PublicTypeTerm::Function {
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
        };
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "wide_id",
            PublicCallableSurface {
                ty: ty.clone(),
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: Some(link_symbol("wide_id", &ty)),
                type_param_bounds: Vec::new(),
            },
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap();

        let binding = env.lookup_all_callables("wide_id")[0];
        match ctx.get(binding.ty) {
            TypeKind::Function { params, result, .. } => {
                assert_eq!(ctx.get(params[0]), TypeKind::Named(String::from("i64")));
                assert_eq!(ctx.get(result), TypeKind::Named(String::from("u64")));
            }
            other => panic!(
                "unexpected materialized backend scalar callable type: {:?}",
                other
            ),
        }
    }

    #[test]
    fn materializer_mvp_handles_binder_indexed_generic_callable() {
        let param = PublicTypeParam {
            name: String::from("T"),
            copy_cap: true,
            clone_cap: true,
            drop_cap: true,
        };
        let ty = PublicTypeTerm::Function {
            type_params: Vec::from([param]),
            params: Vec::from([PublicTypeTerm::GenericParam(PublicTypeParamRef {
                binder_depth: 0,
                index: 0,
            })]),
            result: Box::new(PublicTypeTerm::GenericParam(PublicTypeParamRef {
                binder_depth: 0,
                index: 0,
            })),
            effect: PublicEffect::Pure,
        };
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "id",
            PublicCallableSurface {
                ty: ty.clone(),
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: Some(link_symbol("id", &ty)),
                type_param_bounds: Vec::new(),
            },
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap();

        let binding = env.lookup_all_callables("id")[0];
        match ctx.get(binding.ty) {
            TypeKind::Function {
                type_params,
                params,
                result,
                ..
            } => {
                assert_eq!(type_params.len(), 1);
                assert_eq!(params, Vec::from([type_params[0]]));
                assert_eq!(result, type_params[0]);
                match ctx.get(type_params[0]) {
                    TypeKind::Var(var) => {
                        assert_eq!(var.label.as_deref(), Some("T"));
                        assert!(var.copy_cap);
                        assert!(var.clone_cap);
                        assert!(var.drop_cap);
                    }
                    other => panic!("generic parameter must stay a type variable: {:?}", other),
                }
            }
            other => panic!("unexpected materialized generic type: {:?}", other),
        }
    }

    #[test]
    fn materializer_mvp_keeps_outer_generic_visible_inside_non_generic_function_type() {
        let outer_param = PublicTypeParam {
            name: String::from("T"),
            copy_cap: false,
            clone_cap: true,
            drop_cap: true,
        };
        let inner_ty = PublicTypeTerm::Function {
            type_params: Vec::new(),
            params: Vec::from([PublicTypeTerm::GenericParam(PublicTypeParamRef {
                binder_depth: 0,
                index: 0,
            })]),
            result: Box::new(PublicTypeTerm::GenericParam(PublicTypeParamRef {
                binder_depth: 0,
                index: 0,
            })),
            effect: PublicEffect::Pure,
        };
        let ty = PublicTypeTerm::Function {
            type_params: Vec::from([outer_param]),
            params: Vec::from([inner_ty]),
            result: Box::new(PublicTypeTerm::Unit),
            effect: PublicEffect::Pure,
        };
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "takes_fn",
            PublicCallableSurface {
                ty: ty.clone(),
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: Some(link_symbol("takes_fn", &ty)),
                type_param_bounds: Vec::new(),
            },
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap();

        let binding = env.lookup_all_callables("takes_fn")[0];
        let TypeKind::Function {
            type_params,
            params,
            ..
        } = ctx.get(binding.ty)
        else {
            panic!("outer callable must be function");
        };
        assert_eq!(type_params.len(), 1);
        let outer_t = type_params[0];
        let TypeKind::Function {
            type_params: inner_type_params,
            params: inner_params,
            result: inner_result,
            ..
        } = ctx.get(params[0])
        else {
            panic!("parameter must be an inner function type");
        };
        assert!(inner_type_params.is_empty());
        assert_eq!(inner_params, Vec::from([outer_t]));
        assert_eq!(inner_result, outer_t);
    }

    #[test]
    fn materializer_mvp_handles_tuple_box_and_reference_terms() {
        let tuple_ty = PublicTypeTerm::Tuple(Vec::from([
            PublicTypeTerm::Boxed(Box::new(PublicTypeTerm::I32)),
            PublicTypeTerm::Reference {
                inner: Box::new(PublicTypeTerm::Str),
                mutable: false,
            },
        ]));
        let ty = PublicTypeTerm::Function {
            type_params: Vec::new(),
            params: Vec::from([tuple_ty]),
            result: Box::new(PublicTypeTerm::Unit),
            effect: PublicEffect::Pure,
        };
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "use_pair",
            PublicCallableSurface {
                ty: ty.clone(),
                no_shadow: false,
                arity: 1,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: Some(link_symbol("use_pair", &ty)),
                type_param_bounds: Vec::new(),
            },
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap();

        let binding = env.lookup_all_callables("use_pair")[0];
        let TypeKind::Function { params, .. } = ctx.get(binding.ty) else {
            panic!("callable must be function");
        };
        let TypeKind::Tuple { items } = ctx.get(params[0]) else {
            panic!("parameter must be tuple");
        };
        assert_eq!(items.len(), 2);
        assert!(matches!(ctx.get(items[0]), TypeKind::Box(inner) if inner == ctx.i32()));
        assert!(matches!(
            ctx.get(items[1]),
            TypeKind::Reference(inner, false) if inner == ctx.str()
        ));
    }

    #[test]
    fn materializer_mvp_rejects_malformed_callable_surface_metadata() {
        let non_function = PublicTypeTerm::Unit;
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "bad",
            PublicCallableSurface {
                ty: non_function.clone(),
                no_shadow: false,
                arity: 0,
                effect: PublicEffect::Pure,
                field_accessor: None,
                link_symbol: Some(link_symbol("bad", &non_function)),
                type_param_bounds: Vec::new(),
            },
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();
        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();
        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::CallableTypeExpected
        );

        let mut arity_mismatch = primitive_answer_surface(Some(primitive_answer_link("bad_arity")));
        arity_mismatch.arity = 2;
        let table =
            TypedPublicSurfaceTable::new(Vec::from([callable_entry("bad_arity", arity_mismatch)]));
        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();
        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::CallableArityMismatch {
                surface_arity: 2,
                parameter_count: 1
            }
        );

        let mut effect_mismatch =
            primitive_answer_surface(Some(primitive_answer_link("bad_effect")));
        effect_mismatch.effect = PublicEffect::Impure;
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "bad_effect",
            effect_mismatch,
        )]));
        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();
        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::CallableEffectMismatch {
                surface_effect: PublicEffect::Impure,
                type_effect: PublicEffect::Pure
            }
        );

        let mut signature_mismatch = primitive_answer_surface(Some(PublicCallableLinkSymbol {
            source_path: String::from("stdlib/core/math.nepl"),
            name: String::from("bad_signature"),
            signature_hash: 0xdead,
        }));
        signature_mismatch.no_shadow = false;
        let expected = public_type_term_stable_hash(&signature_mismatch.ty);
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "bad_signature",
            signature_mismatch,
        )]));
        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();
        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::CallableSignatureHashMismatch {
                expected,
                actual: 0xdead
            }
        );
    }

    #[test]
    fn materializer_mvp_rejects_callable_surface_with_non_callable_entry_kind() {
        let table = TypedPublicSurfaceTable::new(Vec::from([TypedPublicSurfaceEntry {
            kind: TypedPublicSignatureKind::Struct,
            name: String::from("answer"),
            exported: true,
            surface: PublicSurfaceShape::Callable(primitive_answer_surface(Some(
                primitive_answer_link("answer"),
            ))),
        }]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();

        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::EntryKindMismatch {
                expected: TypedPublicSignatureKind::Callable,
                actual: TypedPublicSignatureKind::Struct
            }
        );
    }

    #[test]
    fn materializer_mvp_does_not_pollute_env_when_later_entry_rejects() {
        let table = TypedPublicSurfaceTable::new(Vec::from([
            callable_entry(
                "a_valid",
                primitive_answer_surface(Some(primitive_answer_link("a_valid"))),
            ),
            callable_entry("z_invalid", primitive_answer_surface(None)),
        ]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();

        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::MissingCallableLinkSymbol
        );
        assert!(env.lookup_all_callables("a_valid").is_empty());
        assert!(env.lookup_all_callables("z_invalid").is_empty());
    }

    #[test]
    fn materializer_mvp_materializes_field_accessor_callable() {
        let cases = Vec::from([
            (
                "get",
                PublicFieldAccessorKind::Get,
                FieldAccessorKind::Get,
                2usize,
            ),
            (
                "get_ref",
                PublicFieldAccessorKind::GetRef,
                FieldAccessorKind::GetRef,
                2usize,
            ),
            (
                "put",
                PublicFieldAccessorKind::Put,
                FieldAccessorKind::Put,
                3usize,
            ),
        ]);
        let table = TypedPublicSurfaceTable::new(
            cases
                .iter()
                .map(|(name, public_kind, _, _)| {
                    callable_entry(name, field_accessor_surface(name, *public_kind))
                })
                .collect(),
        );
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        let report =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap();

        assert_eq!(report.callables_inserted, cases.len());
        for (name, _, expected_kind, expected_arity) in cases {
            let binding = env.lookup_all_callables(name)[0];
            let BindingKind::Func {
                arity,
                field_accessor,
                def_id,
                ..
            } = &binding.kind
            else {
                panic!("materialized field accessor must be callable");
            };
            assert_eq!(*arity, expected_arity);
            assert_eq!(*field_accessor, Some(expected_kind));
            assert!(def_id.is_none());
            assert_eq!(
                resolved_function_value_identity(binding, binding.ty, Vec::new()),
                Err(FunctionValueIdentityReject::UnresolvedIdentity)
            );
        }
    }

    #[test]
    fn materializer_mvp_rejects_field_accessor_arity_mismatch_and_bounds() {
        let mut field_surface = primitive_answer_surface(Some(primitive_answer_link("get_x")));
        field_surface.field_accessor = Some(PublicFieldAccessorKind::Get);
        let table =
            TypedPublicSurfaceTable::new(Vec::from([callable_entry("get_x", field_surface)]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();
        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::CallableArityMismatch {
                surface_arity: 1,
                parameter_count: 2
            }
        );

        let mut bounds_surface = primitive_answer_surface(Some(primitive_answer_link("bounded")));
        bounds_surface.type_param_bounds = Vec::from([]);
        let mut non_empty_bounds_surface = bounds_surface;
        non_empty_bounds_surface.type_param_bounds =
            Vec::from([crate::typecheck::PublicTypeParamBounds {
                param: crate::typecheck::PublicTypeParamBoundTarget::Ref(PublicTypeParamRef {
                    binder_depth: 0,
                    index: 0,
                }),
                bounds: Vec::new(),
            }]);
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "bounded",
            non_empty_bounds_surface,
        )]));
        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();
        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::TypeParamBoundsUnsupported
        );
    }

    #[test]
    fn materializer_mvp_rejects_no_shadow_signature_conflict() {
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "answer",
            primitive_answer_surface(Some(primitive_answer_link("answer"))),
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();
        materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap();

        let mut shadowed = primitive_answer_surface(Some(link_symbol_with_path(
            "stdlib/core/other.nepl",
            "answer",
            &primitive_answer_ty(),
        )));
        shadowed.no_shadow = true;
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry("answer", shadowed)]));

        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();

        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::NoShadowConflict
        );
    }

    #[test]
    fn materializer_mvp_rejects_non_callable_name_conflict() {
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry(
            "answer",
            primitive_answer_surface(Some(primitive_answer_link("answer"))),
        )]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();
        env.insert_global(Binding {
            name: String::from("answer"),
            ty: ctx.i32(),
            visibility: Visibility::Pub,
            mutable: false,
            no_shadow: false,
            defined: true,
            span: Span::dummy(),
            kind: BindingKind::Var,
        });

        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();

        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::NonCallableNameConflict
        );
    }
}
