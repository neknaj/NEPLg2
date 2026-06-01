extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Effect, Visibility};
use crate::span::Span;
use crate::types::{TypeCtx, TypeId};

use super::env::{same_callable_signature_and_bounds, Binding, BindingKind, Env};
use super::public_signature::TypedPublicSignatureKind;
use super::public_surface::{
    public_type_term_stable_hash,
    PublicCallableLinkSymbol, PublicCallableSurface, PublicEffect, PublicFieldAccessorKind,
    PublicNominalTypeIdentity, PublicSurfaceShape, PublicTypeParam, PublicTypeParamRef,
    PublicTypeTerm, TypedPublicSurfaceEntry, TypedPublicSurfaceTable,
};
use super::traits::BoundEnv;

const FNV1A64_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x100000001b3;

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
    UnsupportedSurfaceKind { kind: TypedPublicSignatureKind },
    MissingCallableLinkSymbol,
    CallableLinkNameMismatch { link_name: String },
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
    FieldAccessorUnsupported { kind: PublicFieldAccessorKind },
    TypeParamBoundsUnsupported,
    DuplicateLinkSymbolConflict { symbol: String },
    NonCallableNameConflict,
    NoShadowConflict,
    UnboundGenericParam { binder_depth: u32, index: u32 },
    TraitSelfUnsupported,
    UnboundGenericParamTerm { name: String },
    NamedTypeUnsupported {
        name: String,
        identity: Option<PublicNominalTypeIdentity>,
    },
    ApplyUnsupported,
}

/// public callable surface を `Env` に注入する内部 materializer。
///
/// この checkpoint は依存 module の body を読まずに callable 候補を復元するための
/// 最小単位である。primitive / tuple / function / generic parameter / box / reference
/// 型は復元するが、名義型、trait bound、field accessor、impl lookup はまだ別 materializer
/// の authority が必要なので拒否する。
#[allow(dead_code)]
pub(super) fn materialize_public_surface_mvp(
    ctx: &mut TypeCtx,
    env: &mut Env,
    table: &TypedPublicSurfaceTable,
    origin_span: Span,
) -> Result<PublicSurfaceMaterializeReport, PublicSurfaceMaterializeReject> {
    let mut report = PublicSurfaceMaterializeReport::default();
    let mut staged = Vec::new();
    for entry in &table.entries {
        report.entries_seen += 1;
        match &entry.surface {
            PublicSurfaceShape::Callable(surface) => {
                let outcome =
                    stage_callable_surface(ctx, env, &staged, entry, surface, origin_span)?;
                match outcome {
                    CallableMaterializeOutcome::Staged(binding) => {
                        staged.push(binding);
                    }
                    CallableMaterializeOutcome::AlreadyPresent => {
                        report.callables_skipped_existing += 1;
                    }
                }
            }
            _ => {
                return Err(reject(
                    entry,
                    PublicSurfaceMaterializeRejectReason::UnsupportedSurfaceKind {
                        kind: entry.kind,
                    },
                ));
            }
        }
    }
    report.callables_inserted = staged.len();
    for binding in staged {
        env.insert_global(binding);
    }
    Ok(report)
}

enum CallableMaterializeOutcome {
    Staged(Binding),
    AlreadyPresent,
}

fn stage_callable_surface(
    ctx: &mut TypeCtx,
    env: &mut Env,
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
    if let Some(kind) = surface.field_accessor {
        return Err(reject(
            entry,
            PublicSurfaceMaterializeRejectReason::FieldAccessorUnsupported { kind },
        ));
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
    if env
        .lookup_all_callables(&entry.name)
        .iter()
        .any(|binding| {
            (binding.no_shadow || surface.no_shadow)
                && same_callable_signature_and_bounds(ctx, binding, ty, &bounds)
        })
        || staged.iter().any(|binding| {
            binding.name == entry.name
                && (binding.no_shadow || surface.no_shadow)
                && same_callable_signature_and_bounds(ctx, binding, ty, &bounds)
        })
    {
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
            field_accessor: None,
            type_param_bounds: bounds,
            captures: Vec::new(),
        },
    }))
}

struct TypeTermMaterializer<'a> {
    ctx: &'a mut TypeCtx,
    binders: Vec<Vec<TypeId>>,
}

impl<'a> TypeTermMaterializer<'a> {
    fn new(ctx: &'a mut TypeCtx) -> Self {
        Self {
            ctx,
            binders: Vec::new(),
        }
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
            PublicTypeTerm::TraitSelf => Err(reject(
                entry,
                PublicSurfaceMaterializeRejectReason::TraitSelfUnsupported,
            )),
            PublicTypeTerm::Named { name, identity } => Err(reject(
                entry,
                PublicSurfaceMaterializeRejectReason::NamedTypeUnsupported {
                    name: name.clone(),
                    identity: identity.clone(),
                },
            )),
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
            PublicTypeTerm::Apply { .. } => Err(reject(
                entry,
                PublicSurfaceMaterializeRejectReason::ApplyUnsupported,
            )),
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

fn stable_callable_symbol(symbol: &PublicCallableLinkSymbol) -> String {
    format!(
        "neplmeta${}${:016x}${:016x}",
        stable_symbol_component(&symbol.name),
        fnv1a64(symbol.source_path.as_str()),
        symbol.signature_hash
    )
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
    use alloc::string::String;
    use alloc::vec::Vec;

    use crate::ast::Visibility;
    use crate::span::Span;
    use crate::typecheck::materializer::{
        materialize_public_surface_mvp, PublicSurfaceMaterializeRejectReason,
    };
    use crate::typecheck::public_signature::TypedPublicSignatureKind;
    use crate::typecheck::public_surface::{
        PublicCallableLinkSymbol, PublicCallableSurface, PublicEffect, PublicFieldAccessorKind,
        PublicNominalTypeIdentity, PublicNominalTypeKind, PublicSurfaceShape, PublicTypeParam,
        PublicTypeParamRef, PublicTypeTerm, TypedPublicSurfaceEntry, TypedPublicSurfaceTable,
    };
    use crate::types::{TypeCtx, TypeKind};

    use super::public_type_term_stable_hash;
    use super::super::env::{
        resolved_function_value_identity, Binding, BindingKind, Env, FunctionValueIdentityReject,
    };

    fn link_symbol(name: &str, ty: &PublicTypeTerm) -> PublicCallableLinkSymbol {
        link_symbol_with_path("stdlib/core/math.nepl", name, ty)
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
            surface: PublicSurfaceShape::Callable(surface),
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

    fn primitive_answer_surface(link_symbol: Option<PublicCallableLinkSymbol>) -> PublicCallableSurface {
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
    fn materializer_mvp_rejects_field_accessor_and_bounds() {
        let mut field_surface = primitive_answer_surface(Some(primitive_answer_link("get_x")));
        field_surface.field_accessor = Some(PublicFieldAccessorKind::Get);
        let table = TypedPublicSurfaceTable::new(Vec::from([callable_entry("get_x", field_surface)]));
        let mut ctx = TypeCtx::new();
        let mut env = Env::new();

        let reject =
            materialize_public_surface_mvp(&mut ctx, &mut env, &table, Span::dummy()).unwrap_err();
        assert_eq!(
            reject.reason,
            PublicSurfaceMaterializeRejectReason::FieldAccessorUnsupported {
                kind: PublicFieldAccessorKind::Get
            }
        );

        let mut bounds_surface = primitive_answer_surface(Some(primitive_answer_link("bounded")));
        bounds_surface.type_param_bounds = Vec::from([]);
        let mut non_empty_bounds_surface = bounds_surface;
        non_empty_bounds_surface.type_param_bounds = Vec::from([crate::typecheck::PublicTypeParamBounds {
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
