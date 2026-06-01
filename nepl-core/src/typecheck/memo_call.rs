use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::function_identity::FunctionValueIdentity;
use crate::hir::{HirExpr, HirExprKind};
use crate::resource_primitives::compiler_memory_type_of_type;
use crate::source_map::SourceMap;
use crate::span::{FileId, Span};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::diagnostics::type_error;
use super::env::{Binding, BindingKind};
use super::signature::type_contains_unbound_var;
use super::traits::{TraitApplication, TraitBound, TraitId, TraitInfo};
use super::type_expectation::TypeExpectation;
use super::{BlockChecker, StackEntry};

const MEMO_CALL_NAME: &str = "memo_call";
const MEMO_CALL_STDLIB_SUFFIX: &str = "/stdlib/core/memo.nepl";
const MEMO_TRAITS_STDLIB_SUFFIX: &str = "/stdlib/core/traits/memo.nepl";
const MEMO_KEY_TRAIT_NAME: &str = "MemoKey";
const MEMO_VALUE_TRAIT_NAME: &str = "MemoValue";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CompilerKnownPrimitive {
    MemoCall,
}

pub(super) fn compiler_known_primitive_for_callable(
    source_map: Option<&SourceMap>,
    source_name: &str,
    binding: &Binding,
) -> Option<CompilerKnownPrimitive> {
    if source_name != MEMO_CALL_NAME || binding.name != MEMO_CALL_NAME {
        return None;
    }
    let BindingKind::Func {
        def_id: Some(def_id),
        builtin: None,
        captures,
        ..
    } = &binding.kind
    else {
        return None;
    };
    if !captures.is_empty() {
        return None;
    }
    let source_map = source_map?;
    let path = source_map.path(FileId(def_id.file_id))?.as_str();
    let normalized = path.replace('\\', "/");
    (normalized == "stdlib/core/memo.nepl" || normalized.ends_with(MEMO_CALL_STDLIB_SUFFIX))
        .then_some(CompilerKnownPrimitive::MemoCall)
}

impl<'a> BlockChecker<'a> {
    pub(super) fn apply_compiler_known_primitive(
        &mut self,
        primitive: CompilerKnownPrimitive,
        args: &[StackEntry],
        result_ty: TypeId,
        span: crate::span::Span,
        expected_ret: Option<TypeExpectation>,
    ) -> Option<StackEntry> {
        match primitive {
            CompilerKnownPrimitive::MemoCall => {
                self.apply_memo_call_phase1(args, result_ty, span, expected_ret)
            }
        }
    }

    fn apply_memo_call_phase1(
        &mut self,
        args: &[StackEntry],
        result_ty: TypeId,
        span: crate::span::Span,
        expected_ret: Option<TypeExpectation>,
    ) -> Option<StackEntry> {
        if args.len() != 1 {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MemoCallRequiresFunctionValue,
                "memo_call phase1 expects exactly one explicit function value",
                span,
            ));
            return None;
        }
        let Some(mut identity) = explicit_function_value_argument(&args[0]) else {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MemoCallRequiresFunctionValue,
                "memo_call phase1 requires an explicit @function argument",
                args[0].expr.span,
            ));
            return None;
        };
        if identity.def_id.is_none() {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MemoCallUnresolvedFunctionIdentity,
                "memo_call phase1 requires a resolved named function identity",
                args[0].expr.span,
            ));
            return None;
        }
        if !matches!(identity.effect, Effect::Pure) {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MemoCallRequiresPureFunction,
                "memo_call can only memoize pure functions",
                args[0].expr.span,
            ));
            return None;
        }
        if identity
            .type_args
            .iter()
            .any(|ty| type_contains_unbound_var(self.ctx, *ty))
            || type_contains_unbound_var(self.ctx, identity.function_ty)
        {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MemoCallUnresolvedFunctionIdentity,
                "memo_call phase1 does not accept unresolved generic function values",
                args[0].expr.span,
            ));
            return None;
        }

        identity.function_ty = self.ctx.resolve_id(identity.function_ty);
        let function_ty = identity.function_ty;
        let (params, value_ty, effect) = match self.ctx.get(function_ty) {
            TypeKind::Function {
                type_params,
                params,
                result,
                effect,
            } if type_params.is_empty() => (params, result, effect),
            TypeKind::Function { .. } => {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::MemoCallUnresolvedFunctionIdentity,
                    "memo_call phase1 requires a monomorphic function value",
                    args[0].expr.span,
                ));
                return None;
            }
            _ => {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::MemoCallRequiresFunctionValue,
                    "memo_call argument must have function type",
                    args[0].expr.span,
                ));
                return None;
            }
        };
        if !matches!(effect, Effect::Pure) {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MemoCallRequiresPureFunction,
                "memo_call can only memoize pure functions",
                args[0].expr.span,
            ));
            return None;
        }
        if params.len() != 1 {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MemoCallUnsupportedKey,
                "memo_call phase1 memoizes a single key argument; use a tuple key for multiple inputs",
                args[0].expr.span,
            ));
            return None;
        }
        let key_ty = self.ctx.resolve_id(params[0]);
        let value_ty = self.ctx.resolve_id(value_ty);
        if !memo_phase1_key_type_supported(self.ctx, key_ty) {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MemoCallUnsupportedKey,
                format!(
                    "memo_call phase1 does not support key type {}",
                    self.ctx.type_to_string(key_ty)
                ),
                args[0].expr.span,
            ));
            return None;
        }
        if !self.memo_phase1_key_has_memo_key(key_ty) {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MemoCallUnsupportedKey,
                format!(
                    "memo_call phase1 key type {} must implement MemoKey",
                    self.ctx.type_to_string(key_ty)
                ),
                args[0].expr.span,
            ));
            return None;
        }
        if !memo_phase1_value_type_supported(self.ctx, value_ty) {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MemoCallUnsupportedValue,
                format!(
                    "memo_call phase1 does not support value type {}",
                    self.ctx.type_to_string(value_ty)
                ),
                args[0].expr.span,
            ));
            return None;
        }
        if !self.memo_phase1_value_has_memo_value(value_ty) {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MemoCallUnsupportedValue,
                format!(
                    "memo_call phase1 value type {} must implement MemoValue",
                    self.ctx.type_to_string(value_ty)
                ),
                args[0].expr.span,
            ));
            return None;
        }

        if let Some(expectation) = expected_ret {
            if self.ctx.unify(function_ty, expectation.target()).is_err() {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::MemoCallBoundaryRestricted,
                    "memo_call result type must remain the memoized function type",
                    expectation.diagnostic_span(span),
                ));
                return None;
            }
        } else if !type_contains_unbound_var(self.ctx, result_ty)
            && !self
                .ctx
                .same_type(self.ctx.resolve_id(result_ty), function_ty)
        {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MemoCallBoundaryRestricted,
                "memo_call result type must remain the memoized function type",
                span,
            ));
            return None;
        }

        Some(StackEntry {
            ty: function_ty,
            expr: HirExpr {
                ty: function_ty,
                kind: HirExprKind::MemoizedFunctionValue(identity),
                span,
            },
            type_args: Vec::new(),
            assign: None,
            explicit_function_ref: false,
            auto_call: false,
        })
    }

    fn memo_phase1_key_has_memo_key(&self, key_ty: TypeId) -> bool {
        self.memo_phase1_type_has_trait(key_ty, MEMO_KEY_TRAIT_NAME)
    }

    fn memo_phase1_value_has_memo_value(&self, value_ty: TypeId) -> bool {
        self.memo_phase1_type_has_trait(value_ty, MEMO_VALUE_TRAIT_NAME)
    }

    fn memo_phase1_type_has_trait(&self, ty: TypeId, trait_name: &str) -> bool {
        let Some(trait_info) = self.memo_phase1_stdlib_trait_info(trait_name) else {
            return false;
        };
        let bound = TraitBound {
            application: TraitApplication {
                trait_id: TraitId::from_name(trait_name),
                args: Vec::new(),
            },
            trait_self_ty: trait_info.self_ty,
        };
        self.trait_bound_satisfied(&bound, ty)
    }

    fn memo_phase1_stdlib_trait_info(&self, trait_name: &str) -> Option<&TraitInfo> {
        let trait_info = self.traits.get(trait_name)?;
        memo_phase1_trait_defined_in_stdlib(self.source_map, trait_info.span).then_some(trait_info)
    }
}

fn explicit_function_value_argument(arg: &StackEntry) -> Option<FunctionValueIdentity> {
    if !arg.explicit_function_ref {
        return None;
    }
    match &arg.expr.kind {
        HirExprKind::FnValue(identity) => Some(identity.clone()),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoPhase1TypeRole {
    Key,
    Value,
}

fn memo_phase1_key_type_supported(ctx: &mut TypeCtx, ty: TypeId) -> bool {
    memo_phase1_type_supported(ctx, ty, MemoPhase1TypeRole::Key)
}

fn memo_phase1_value_type_supported(ctx: &mut TypeCtx, ty: TypeId) -> bool {
    memo_phase1_type_supported(ctx, ty, MemoPhase1TypeRole::Value)
}

fn memo_phase1_type_supported(ctx: &mut TypeCtx, ty: TypeId, role: MemoPhase1TypeRole) -> bool {
    let mut visiting = BTreeSet::new();
    memo_phase1_type_supported_inner(ctx, ty, role, &mut visiting)
}

fn memo_phase1_type_supported_inner(
    ctx: &mut TypeCtx,
    ty: TypeId,
    role: MemoPhase1TypeRole,
    visiting: &mut BTreeSet<TypeId>,
) -> bool {
    let resolved = ctx.resolve_named_type_id(ctx.resolve_id(ty));
    if compiler_memory_type_of_type(ctx, resolved).is_some()
        || ctx.has_drop(resolved)
        || !ctx.is_copy(resolved)
    {
        return false;
    }
    if !visiting.insert(resolved) {
        return false;
    }
    let kind = ctx.get_ref(resolved).clone();
    let result = match kind {
        TypeKind::Unit | TypeKind::I32 | TypeKind::U8 | TypeKind::Bool | TypeKind::Char => true,
        TypeKind::F32 => matches!(role, MemoPhase1TypeRole::Value),
        TypeKind::Tuple { items } => items
            .iter()
            .all(|item| memo_phase1_type_supported_inner(ctx, *item, role, visiting)),
        TypeKind::Struct {
            type_params,
            fields,
            ..
        } if type_params.is_empty() => fields
            .iter()
            .all(|field| memo_phase1_type_supported_inner(ctx, *field, role, visiting)),
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } if type_params.is_empty() => variants.iter().all(|variant| {
            variant
                .payload
                .map(|payload| memo_phase1_type_supported_inner(ctx, payload, role, visiting))
                .unwrap_or(true)
        }),
        TypeKind::Apply { base, args } => {
            memo_phase1_apply_supported(ctx, base, &args, role, visiting)
        }
        TypeKind::Never
        | TypeKind::Str
        | TypeKind::Named(_)
        | TypeKind::Struct { .. }
        | TypeKind::Enum { .. }
        | TypeKind::Function { .. }
        | TypeKind::Var(_)
        | TypeKind::Box(_)
        | TypeKind::Reference(_, _) => false,
    };
    visiting.remove(&resolved);
    result
}

fn memo_phase1_apply_supported(
    ctx: &mut TypeCtx,
    base: TypeId,
    args: &[TypeId],
    role: MemoPhase1TypeRole,
    visiting: &mut BTreeSet<TypeId>,
) -> bool {
    let base = ctx.resolve_named_type_id(ctx.resolve_id(base));
    let kind = ctx.get_ref(base).clone();
    match kind {
        TypeKind::Struct {
            type_params,
            fields,
            ..
        } if type_params.len() == args.len() => {
            let mapping = type_param_mapping(ctx, &type_params, args);
            fields.iter().all(|field| {
                let substituted = ctx.substitute(*field, &mapping);
                memo_phase1_type_supported_inner(ctx, substituted, role, visiting)
            })
        }
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } if type_params.len() == args.len() => {
            let mapping = type_param_mapping(ctx, &type_params, args);
            variants.iter().all(|variant| {
                variant
                    .payload
                    .map(|payload| {
                        let substituted = ctx.substitute(payload, &mapping);
                        memo_phase1_type_supported_inner(ctx, substituted, role, visiting)
                    })
                    .unwrap_or(true)
            })
        }
        _ => false,
    }
}

fn type_param_mapping(
    ctx: &TypeCtx,
    type_params: &[TypeId],
    args: &[TypeId],
) -> BTreeMap<TypeId, TypeId> {
    type_params
        .iter()
        .zip(args.iter())
        .map(|(param, arg)| (ctx.resolve_id(*param), ctx.resolve_id(*arg)))
        .collect()
}

fn memo_phase1_trait_defined_in_stdlib(source_map: Option<&SourceMap>, span: Span) -> bool {
    let Some(source_map) = source_map else {
        return false;
    };
    let Some(path) = source_map.path(span.file_id) else {
        return false;
    };
    let normalized = path.as_str().replace('\\', "/");
    normalized == "stdlib/core/traits/memo.nepl" || normalized.ends_with(MEMO_TRAITS_STDLIB_SUFFIX)
}
