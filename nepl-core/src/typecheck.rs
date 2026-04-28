extern crate alloc;
#[cfg(not(target_os = "none"))]
extern crate std;

mod ascription;
mod binding_rules;
mod block_check;
mod call_reduction;
mod call_resolution;
mod driver;
mod effect_check;
mod env;
mod field_access;
mod function_apply;
mod hir_finalize;
mod match_check;
mod model;
mod name_lookup;
mod prefix_check;
mod signature;
mod syntax_helpers;
mod trait_check;
mod traits;
mod type_expr;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::*;
use crate::compiler::{BuildProfile, CompileTarget};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::*;
use crate::resolve::ImportResolution;
use crate::source_map::SourceMap;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use binding_rules::emit_shadow_warning;
use env::{Binding, BindingKind, Env};
use hir_finalize::resolve_type_ids_in_function;
use model::{
    AssignKind, CheckedFunction, EnumInfo, FieldAccessorKind, FieldIdx, ScalarMatchKind,
    StackEntry, StructInfo,
};
use signature::{mangle_function_symbol, type_contains_unbound_var};
use traits::{
    trait_application_matches, type_param_has_trait_bound, ImplInfo, TraitBoundRef, TraitInfo,
};
use type_expr::{LabelEnv, StringTable};

pub use driver::{typecheck, TypeCheckResult};

macro_rules! typecheck_log {
    ($($arg:tt)*) => {{
        #[cfg(target_os = "none")]
        {
            let _ = core::format_args!($($arg)*);
        }
        #[cfg(not(target_os = "none"))]
        {
            std::eprintln!($($arg)*);
        }
    }};
}

// ---------------------------------------------------------------------
// Function checking
// ---------------------------------------------------------------------

fn snapshot_function_type_param_bindings(
    ctx: &TypeCtx,
    func_ty: TypeId,
) -> BTreeMap<TypeId, Option<TypeId>> {
    let mut out = BTreeMap::new();
    let TypeKind::Function { type_params, .. } = ctx.get(func_ty) else {
        return out;
    };
    for tp in type_params {
        out.extend(ctx.snapshot_type_var_bindings(tp));
    }
    out
}

fn check_function(
    f: &FnDef,
    func_ty: TypeId,
    _is_entry: bool,
    target: CompileTarget,
    profile: BuildProfile,
    captured_params: &[(String, TypeId)],
    ctx: &mut TypeCtx,
    env: &mut Env,
    labels: &mut LabelEnv,
    strings: &mut StringTable,
    enums: &BTreeMap<String, EnumInfo>,
    structs: &BTreeMap<String, StructInfo>,
    instantiations: &mut BTreeMap<String, Vec<Vec<TypeId>>>,
    type_param_bounds: BTreeMap<TypeId, Vec<TraitBoundRef>>,
    traits: &BTreeMap<String, TraitInfo>,
    impls: &Vec<ImplInfo>,
    generated_functions: &mut Vec<HirFunction>,
    import_resolution: &ImportResolution,
    source_map: Option<&SourceMap>,
) -> Result<CheckedFunction, Vec<Diagnostic>> {
    let mut diags = Vec::new();
    let func_ty_snapshot = ctx.snapshot_type_var_bindings(func_ty);
    let type_param_snapshot = snapshot_function_type_param_bindings(ctx, func_ty);
    let (params_ty, result_ty, effect) = match ctx.get(func_ty) {
        TypeKind::Function {
            params,
            result,
            effect,
            ..
        } => (params, result, effect),
        _ => {
            diags.push(
                Diagnostic::error("function signature must be a function type", f.name.span)
                    .with_id(DiagnosticId::TypeFunctionSignatureMustBeFunction),
            );
            return Err(diags);
        }
    };
    if params_ty.len() != captured_params.len() + f.params.len() {
        diags.push(
            Diagnostic::error("parameter count mismatch with signature", f.name.span)
                .with_id(DiagnosticId::TypeArgumentArityMismatch),
        );
        return Err(diags);
    }
    diags.extend(crate::target_precheck::precheck_function_raw_body_target(
        f, target, profile,
    ));
    if diags
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
    {
        return Err(diags);
    }

    env.push_scope();
    for (name, ty) in captured_params.iter() {
        emit_shadow_warning(&mut diags, env, name, f.name.span, "captured parameter");
        let _ = env.insert_local(Binding {
            name: name.clone(),
            ty: *ty,
            mutable: false,
            no_shadow: false,
            defined: true,
            span: f.name.span,
            kind: BindingKind::Var,
        });
    }
    for (param, ty) in f
        .params
        .iter()
        .zip(params_ty.iter().skip(captured_params.len()))
    {
        emit_shadow_warning(&mut diags, env, &param.name, param.span, "parameter");
        let _ = env.insert_local(Binding {
            name: param.name.clone(),
            ty: *ty,
            mutable: false,
            no_shadow: false,
            defined: true,
            span: param.span,
            kind: BindingKind::Var,
        });
    }

    let (body, mut diag_out, pending_trait_checks) = {
        let mut checker = BlockChecker {
            ctx,
            env,
            labels,
            string_table: strings,
            diagnostics: Vec::new(),
            pending_trait_bound_checks: Vec::new(),
            current_effect: effect,
            enums,
            structs,
            instantiations,
            type_param_bounds: type_param_bounds.clone(),
            import_resolution,
            traits,
            impls,
            generated_functions,
            target,
            profile,
            source_map,
        };

        let body_res = match &f.body {
            FnBody::Parsed(b) => {
                if let Some(raw) = checker.select_target_raw_body(b) {
                    if !checker.validate_raw_body_effect(&raw, f.name.span) {
                        checker.ctx.restore_type_var_bindings(&func_ty_snapshot);
                        return Err(checker.diagnostics);
                    }
                    raw
                } else {
                    match checker.check_block(b, 0, true, Some(result_ty)) {
                        Some((blk, _val)) => {
                            if checker.ctx.unify(blk.ty, result_ty).is_err() {
                                checker.diagnostics.push(
                                    Diagnostic::error(
                                        "return type does not match signature",
                                        f.name.span,
                                    )
                                    .with_id(DiagnosticId::TypeReturnTypeMismatch),
                                );
                            }
                            HirBody::Block(blk)
                        }
                        None => {
                            checker.ctx.restore_type_var_bindings(&func_ty_snapshot);
                            return Err(checker.diagnostics);
                        }
                    }
                }
            }
            FnBody::Wasm(wb) => {
                let raw = HirBody::Wasm(wb.clone());
                if !checker.validate_raw_body_effect(&raw, f.name.span) {
                    checker.ctx.restore_type_var_bindings(&func_ty_snapshot);
                    return Err(checker.diagnostics);
                }
                raw
            }
            FnBody::LlvmIr(lb) => {
                let raw = HirBody::LlvmIr(lb.clone());
                if !checker.validate_raw_body_effect(&raw, f.name.span) {
                    checker.ctx.restore_type_var_bindings(&func_ty_snapshot);
                    return Err(checker.diagnostics);
                }
                raw
            }
        };
        (
            body_res,
            checker.diagnostics,
            checker.pending_trait_bound_checks,
        )
    };
    for (bound, ty, span) in pending_trait_checks {
        let resolved = ctx.resolve_id(ty);
        let satisfied = type_param_has_trait_bound(ctx, &type_param_bounds, ty, &bound.name)
            || type_param_has_trait_bound(ctx, &type_param_bounds, resolved, &bound.name)
            || impls.iter().any(|imp| {
                imp.trait_base_name
                    .as_deref()
                    .map(|base| {
                        trait_application_matches(
                            ctx,
                            &bound.trait_base_name,
                            &bound.trait_args,
                            base,
                            &imp.trait_args,
                        )
                    })
                    .unwrap_or(false)
                    && ctx.type_pattern_matches(imp.target_ty, resolved)
            });
        if !satisfied {
            diag_out.push(
                Diagnostic::error(
                    format!("type does not satisfy trait bound '{}'", bound.name),
                    span,
                )
                .with_id(DiagnosticId::TypeTraitBoundUnsatisfied),
            );
        }
    }
    env.pop_scope();
    let has_error = diag_out
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error));
    if has_error {
        ctx.restore_type_var_bindings(&func_ty_snapshot);
        return Err(diag_out);
    }

    ctx.restore_type_var_bindings(&type_param_snapshot);
    let out_name = env
        .lookup_func_symbol(&f.name.name, func_ty, ctx)
        .unwrap_or_else(|| {
            if type_contains_unbound_var(ctx, func_ty) {
                f.name.name.clone()
            } else {
                mangle_function_symbol(&f.name.name, func_ty, ctx)
            }
        });
    let mut function = HirFunction {
        doc: f.doc.clone(),
        name: out_name,
        func_ty, // assigned here
        params: {
            let mut out = Vec::new();
            for (name, ty) in captured_params.iter() {
                out.push(HirParam {
                    name: name.clone(),
                    ty: *ty,
                    mutable: false,
                });
            }
            for (p, ty) in f
                .params
                .iter()
                .zip(params_ty.iter().skip(captured_params.len()))
            {
                out.push(HirParam {
                    name: p.name.clone(),
                    ty: *ty,
                    mutable: false,
                });
            }
            out
        },
        result: result_ty,
        effect,
        body,
        span: f.name.span,
    };
    resolve_type_ids_in_function(ctx, &mut function);
    if crate::log::is_verbose() && function.name.contains("partition") {
        let block_ty = match &function.body {
            HirBody::Block(block) => ctx.type_to_string(block.ty),
            _ => String::from("<non-block>"),
        };
        let tail_ty = match &function.body {
            HirBody::Block(block) => block
                .lines
                .last()
                .map(|line| ctx.type_to_string(line.expr.ty))
                .unwrap_or_else(|| String::from("<empty-block>")),
            _ => String::from("<non-block>"),
        };
        typecheck_log!(
            "check_function result debug: name={} result={} block_ty={} tail_ty={} func_ty={}",
            function.name,
            ctx.type_to_string(function.result),
            block_ty,
            tail_ty,
            ctx.type_to_string(function.func_ty)
        );
    }
    Ok(CheckedFunction {
        function,
        diagnostics: diag_out,
    })
}

// ---------------------------------------------------------------------
// Block checker
// ---------------------------------------------------------------------

struct BlockChecker<'a> {
    ctx: &'a mut TypeCtx,
    env: &'a mut Env,
    labels: &'a mut LabelEnv,
    string_table: &'a mut StringTable,
    diagnostics: Vec<Diagnostic>,
    pending_trait_bound_checks: Vec<(TraitBoundRef, TypeId, Span)>,
    current_effect: Effect,
    enums: &'a BTreeMap<String, EnumInfo>,
    structs: &'a BTreeMap<String, StructInfo>,
    instantiations: &'a mut BTreeMap<String, Vec<Vec<TypeId>>>, // new
    type_param_bounds: BTreeMap<TypeId, Vec<TraitBoundRef>>,
    import_resolution: &'a ImportResolution,
    traits: &'a BTreeMap<String, TraitInfo>,
    impls: &'a Vec<ImplInfo>,
    generated_functions: &'a mut Vec<HirFunction>,
    target: CompileTarget,
    profile: BuildProfile,
    source_map: Option<&'a SourceMap>,
}
