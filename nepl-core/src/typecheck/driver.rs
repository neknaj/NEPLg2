use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::*;
use crate::compiler::{BuildProfile, CompileTarget};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::DiagnosticCode;
use crate::hir::*;
use crate::resolve::{DefId, ImportResolution};
use crate::source_map::SourceMap;
use crate::span::Span;
use crate::types::{EnumVariantInfo, TypeCtx, TypeId, TypeKind};

use super::binding_rules::{
    detect_field_accessor_fn, find_invalid_same_file_overload, find_nonshadow_same_signature_func,
    find_same_signature_func, is_callable_binding, shadow_blocked_by_nonshadow,
};
use super::check_function;
use super::driver_entry::resolve_entry_function;
use super::env::{Binding, BindingKind, Env};
use super::model::{EnumInfo, StructInfo};
use super::signature::{
    contains_same_type, function_signature_string, mangle_function_symbol, mangle_impl_method,
    push_unique_type, same_function_signature, type_contains_unbound_var,
};
use super::syntax_helpers::gate_allows;
use super::traits::{
    collect_type_params, format_trait_ref_name, insert_substitution_mapping, ImplInfo,
    TraitBoundRef, TraitCapability, TraitInfo, TraitSemantics,
};
use super::type_expr::{type_from_expr, LabelEnv, StringTable};

macro_rules! driver_log {
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
#[cfg(not(target_os = "none"))]
fn print_diagnostics_summary(diags: &alloc::vec::Vec<crate::diagnostic::Diagnostic>) {
    if diags.is_empty() {
        return;
    }
    // Print a short, readable summary of diagnostics (one line per diagnostic)
    driver_log!("Compiler diagnostics:");
    for d in diags.iter() {
        let sev = match d.severity {
            crate::diagnostic::Severity::Error => "error",
            crate::diagnostic::Severity::Warning => "warning",
        };
        // Display primary span as file_id:start-end for quick location.
        let span = &d.primary.span;
        driver_log!(
            "- {}: {} (span: {:?}:{:?}-{:?})",
            sev,
            d.message,
            span.file_id,
            span.start,
            span.end
        );
        for sec in d.secondary.iter() {
            driver_log!(
                "  note: {:?}:{:?}-{:?} {}",
                sec.span.file_id,
                sec.span.start,
                sec.span.end,
                sec.message
                    .as_ref()
                    .unwrap_or(&alloc::string::String::new())
            );
        }
    }
}

#[cfg(target_os = "none")]
fn print_diagnostics_summary(_diags: &alloc::vec::Vec<crate::diagnostic::Diagnostic>) {}
#[derive(Debug)]
pub struct TypeCheckResult {
    pub module: Option<HirModule>,
    pub diagnostics: Vec<Diagnostic>,
    pub types: TypeCtx,
}

pub fn typecheck(
    module: &crate::ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
    source_map: Option<&SourceMap>,
) -> TypeCheckResult {
    let mut ctx = TypeCtx::new();
    let mut label_env = LabelEnv::new();
    let mut env = Env::new();
    let mut diagnostics = Vec::new();
    diagnostics.extend(crate::target_gate::validate_module_gates(
        module, target, profile,
    ));
    let mut strings = StringTable::new();
    let mut enums: BTreeMap<String, EnumInfo> = BTreeMap::new();
    let mut structs: BTreeMap<String, StructInfo> = BTreeMap::new();
    let mut traits: BTreeMap<String, TraitInfo> = BTreeMap::new();
    let mut impls: Vec<ImplInfo> = Vec::new();
    let mut rejected_copy_targets: Vec<TypeId> = Vec::new();
    let mut pending_copy_clone_checks: Vec<(TypeId, Span)> = Vec::new();
    let mut duplicate_impl_spans: BTreeSet<(u32, u32, u32)> = BTreeSet::new();
    let import_resolution = ImportResolution::from_module(module, source_map);

    let mut entry: Option<(String, Span)> = None;
    let mut externs: Vec<HirExtern> = Vec::new();
    let mut seen_directive_spans: BTreeSet<(u32, u32, u32)> = BTreeSet::new();
    let mut instantiations: BTreeMap<String, Vec<Vec<TypeId>>> = BTreeMap::new();
    let mut apply_directive = |d: &Directive, allowed: bool| {
        if !allowed {
            return;
        }
        let sp = match d {
            Directive::Entry { name } => name.span,
            Directive::Extern { span, .. } => *span,
            Directive::Target { span, .. } => *span,
            Directive::Import { span, .. } => *span,
            Directive::Use { span, .. } => *span,
            Directive::IfTarget { span, .. } => *span,
            Directive::IfProfile { span, .. } => *span,
            Directive::IndentWidth { span, .. } => *span,
            Directive::Include { span, .. } => *span,
            Directive::Prelude { span, .. } => *span,
            Directive::NoPrelude { span } => *span,
        };
        let key = (sp.file_id.0, sp.start, sp.end);
        if !seen_directive_spans.insert(key) {
            return;
        }
        if let Directive::Entry { name } = d {
            entry = Some((name.name.clone(), name.span));
        } else if let Directive::Extern {
            module: m,
            name: n,
            func,
            signature,
            span,
        } = d
        {
            if matches!(target, CompileTarget::Wasm | CompileTarget::Llvm)
                && m == "wasi_snapshot_preview1"
            {
                diagnostics.push(
                    Diagnostic::error("WASI import is only allowed for #target wasi", *span)
                        .with_code(DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::ExternWasiTargetMismatch,
                        )),
                );
                return;
            }
            let ty = type_from_expr(&mut ctx, &mut label_env, signature);
            if let TypeKind::Function {
                params,
                result,
                effect,
                type_params: _,
            } = ctx.get(ty)
            {
                env.insert_global(Binding {
                    name: func.name.clone(),
                    ty,
                    mutable: false,
                    no_shadow: false,
                    defined: true,
                    span: *span,
                    kind: BindingKind::Func {
                        def_id: DefId::from_span(func.span),
                        symbol: func.name.clone(),
                        effect,
                        arity: params.len(),
                        builtin: None,
                        field_accessor: None,
                        type_param_bounds: BTreeMap::new(),
                        captures: Vec::new(),
                    },
                });
                externs.push(HirExtern {
                    module: m.clone(),
                    name: n.clone(),
                    local_name: func.name.clone(),
                    params,
                    result,
                    effect,
                    span: *span,
                });
            } else {
                diagnostics.push(
                    Diagnostic::error("extern signature must be a function type", *span).with_code(
                        DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::ExternSignatureNotFunction,
                        ),
                    ),
                );
            }
        }
    };

    let mut pending_if: Option<bool> = None;
    for d in &module.directives {
        if let Some(allowed) = gate_allows(d, target, profile) {
            pending_if = Some(allowed);
            continue;
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        apply_directive(d, allowed);
    }
    let mut pending_if: Option<bool> = None;
    for item in &module.root.items {
        let Stmt::Directive(d) = item else {
            pending_if = None;
            continue;
        };
        if let Some(allowed) = gate_allows(d, target, profile) {
            pending_if = Some(allowed);
            continue;
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        apply_directive(d, allowed);
    }

    // Builtins are defined in stdlib (e.g. std/mem) or via #extern.

    // Collect top-level function signatures (hoist)
    // Also hoist struct/enum definitions
    let mut pending_if: Option<bool> = None;
    let mut fn_aliases: Vec<&FnAlias> = Vec::new();
    for item in &module.root.items {
        if let Stmt::Directive(d) = item {
            if let Some(allowed) = gate_allows(d, target, profile) {
                pending_if = Some(allowed);
                continue;
            }
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }
        match item {
            Stmt::EnumDef(e) => {
                if enums.contains_key(&e.name.name)
                    || env.lookup_any_defined(&e.name.name).is_some()
                {
                    continue;
                }
                if env.lookup_any_defined(&e.name.name).is_some()
                    || structs.contains_key(&e.name.name)
                {
                    diagnostics.push(
                        Diagnostic::error("name already used by another item", e.name.span)
                            .with_code(DiagnosticCode::Resolve(
                                crate::diagnostic_codes::ResolveDiagnosticCode::ItemNameConflict,
                            )),
                    );
                    continue;
                }
                for p in &e.type_params {
                    if !p.bounds.is_empty() {
                        diagnostics.push(
                            Diagnostic::error(
                                "enum type parameter bounds are not supported yet",
                                p.name.span,
                            )
                            .with_code(DiagnosticCode::Type(crate::diagnostic_codes::TypeDiagnosticCode::EnumTypeParamBoundsUnsupported)),
                        );
                    }
                }
                let mut e_labels = LabelEnv::new();
                let mut tps = Vec::new();
                for p in &e.type_params {
                    let id = ctx.fresh_var(Some(p.name.name.clone()));
                    e_labels.insert(p.name.name.clone(), id);
                    tps.push(id);
                }
                let mut vars = Vec::new();
                for v in &e.variants {
                    let payload_ty = v
                        .payload
                        .as_ref()
                        .map(|p| type_from_expr(&mut ctx, &mut e_labels, p));
                    vars.push(EnumVariantInfo {
                        name: v.name.name.clone(),
                        payload: payload_ty,
                    });
                }
                let ty = ctx.register_named(
                    e.name.name.clone(),
                    TypeKind::Enum {
                        doc: e.doc.clone(),
                        name: e.name.name.clone(),
                        type_params: tps.clone(),
                        variants: vars.clone(),
                    },
                );
                label_env.insert(e.name.name.clone(), ty);
                enums.insert(
                    e.name.name.clone(),
                    EnumInfo {
                        ty,
                        type_params: tps.clone(),
                        variants: vars.clone(),
                    },
                );

                // Register variants as global functions
                for v in vars.iter() {
                    let mut params = Vec::new();
                    if let Some(pty) = v.payload {
                        params.push(pty);
                    }
                    let ret_ty = if tps.is_empty() {
                        ty
                    } else {
                        ctx.apply(ty, tps.clone())
                    };
                    let func_ty = ctx.function(tps.clone(), params.clone(), ret_ty, Effect::Pure);

                    // Simple name (e.g. "Some")
                    env.insert_global(Binding {
                        name: v.name.clone(),
                        ty: func_ty,
                        mutable: false,
                        no_shadow: false,
                        defined: true,
                        span: e.name.span,
                        kind: BindingKind::Func {
                            def_id: DefId::from_span(e.name.span),
                            symbol: v.name.clone(),
                            effect: Effect::Pure,
                            arity: params.len(),
                            builtin: None,
                            field_accessor: None,
                            type_param_bounds: BTreeMap::new(),
                            captures: Vec::new(),
                        },
                    });

                    // Qualified name (e.g. "Option::Some")
                    env.insert_global(Binding {
                        name: format!("{}::{}", e.name.name, v.name),
                        ty: func_ty,
                        mutable: false,
                        no_shadow: false,
                        defined: true,
                        span: e.name.span,
                        kind: BindingKind::Func {
                            def_id: DefId::from_span(e.name.span),
                            symbol: format!("{}::{}", e.name.name, v.name),
                            effect: Effect::Pure,
                            arity: params.len(),
                            builtin: None,
                            field_accessor: None,
                            type_param_bounds: BTreeMap::new(),
                            captures: Vec::new(),
                        },
                    });
                }
            }
            Stmt::StructDef(s) => {
                if structs.contains_key(&s.name.name)
                    || env.lookup_any_defined(&s.name.name).is_some()
                {
                    continue;
                }
                if env.lookup_any_defined(&s.name.name).is_some()
                    || enums.contains_key(&s.name.name)
                {
                    diagnostics.push(
                        Diagnostic::error("name already used by another item", s.name.span)
                            .with_code(DiagnosticCode::Resolve(
                                crate::diagnostic_codes::ResolveDiagnosticCode::ItemNameConflict,
                            )),
                    );
                    continue;
                }
                for p in &s.type_params {
                    if !p.bounds.is_empty() {
                        diagnostics.push(
                            Diagnostic::error(
                                "struct type parameter bounds are not supported yet",
                                p.name.span,
                            )
                            .with_code(DiagnosticCode::Type(crate::diagnostic_codes::TypeDiagnosticCode::StructTypeParamBoundsUnsupported)),
                        );
                    }
                }
                let mut s_labels = LabelEnv::new();
                let mut tps = Vec::new();
                for p in &s.type_params {
                    let id = ctx.fresh_var(Some(p.name.name.clone()));
                    s_labels.insert(p.name.name.clone(), id);
                    tps.push(id);
                }
                let mut fs = Vec::new();
                let mut f_names = Vec::new();
                for (ident, ty_expr) in &s.fields {
                    fs.push(type_from_expr(&mut ctx, &mut s_labels, ty_expr));
                    f_names.push(ident.name.clone());
                }
                let ty = ctx.register_named(
                    s.name.name.clone(),
                    TypeKind::Struct {
                        doc: s.doc.clone(),
                        name: s.name.name.clone(),
                        type_params: tps.clone(),
                        fields: fs.clone(),
                        field_names: f_names.clone(),
                    },
                );

                let is_tag_unit_struct = fs.len() == 1
                    && f_names.len() == 1
                    && f_names[0] == "tag"
                    && matches!(ctx.get(ctx.resolve_id(fs[0])), TypeKind::Unit);
                let ret_ty = if tps.is_empty() {
                    ty
                } else {
                    ctx.apply(ty, tps.clone())
                };
                let constructor_params = if is_tag_unit_struct {
                    Vec::new()
                } else {
                    fs.clone()
                };
                let constructor_ty =
                    ctx.function(tps.clone(), constructor_params, ret_ty, Effect::Pure);
                env.insert_global(Binding {
                    name: s.name.name.clone(),
                    ty: constructor_ty,
                    mutable: false,
                    no_shadow: false,
                    defined: true,
                    span: s.name.span,
                    kind: BindingKind::Func {
                        def_id: DefId::from_span(s.name.span),
                        symbol: s.name.name.clone(),
                        effect: Effect::Pure,
                        arity: if is_tag_unit_struct { 0 } else { fs.len() },
                        builtin: None,
                        field_accessor: None,
                        type_param_bounds: BTreeMap::new(),
                        captures: Vec::new(),
                    },
                });

                label_env.insert(s.name.name.clone(), ty);
                structs.insert(
                    s.name.name.clone(),
                    StructInfo {
                        ty,
                        type_params: tps,
                        fields: fs,
                        field_names: f_names,
                    },
                );
            }
            Stmt::Trait(t) => {
                let mut f_labels = LabelEnv::new();
                let (_tps, _bounds_vec, _bounds_map) = collect_type_params(
                    &mut ctx,
                    &mut f_labels,
                    &t.type_params,
                    &traits,
                    &mut diagnostics,
                );
                let mut capabilities = Vec::new();
                for cap in &t.capabilities {
                    match cap {
                        crate::ast::TraitCapability::Copy => {
                            if !capabilities.contains(&TraitCapability::Copy) {
                                capabilities.push(TraitCapability::Copy);
                            }
                        }
                        crate::ast::TraitCapability::Clone => {
                            if !capabilities.contains(&TraitCapability::Clone) {
                                capabilities.push(TraitCapability::Clone);
                            }
                        }
                        crate::ast::TraitCapability::Drop => {
                            if !capabilities.contains(&TraitCapability::Drop) {
                                capabilities.push(TraitCapability::Drop);
                            }
                        }
                        crate::ast::TraitCapability::Unknown(name) => {
                            diagnostics.push(
                                Diagnostic::error(
                                    format!("unknown trait capability '{}'", name.trim()),
                                    t.name.span,
                                )
                                .with_code(DiagnosticCode::Type(
                                    crate::diagnostic_codes::TypeDiagnosticCode::TraitCapabilityUnknown,
                                )),
                            );
                        }
                    }
                }
                let mut type_param_labels = LabelEnv::new();
                let (tps, _bounds_vec, _bounds_map) = collect_type_params(
                    &mut ctx,
                    &mut type_param_labels,
                    &t.type_params,
                    &traits,
                    &mut diagnostics,
                );
                let self_ty = ctx.fresh_var(Some(String::from("Self")));
                f_labels.insert(String::from("Self"), self_ty);
                for tp in &t.type_params {
                    if let Some(ty) = type_param_labels.get(&tp.name.name) {
                        f_labels.insert(tp.name.name.clone(), *ty);
                    }
                }
                let mut methods = BTreeMap::new();
                for m in &t.methods {
                    if !m.type_params.is_empty() {
                        diagnostics.push(
                            Diagnostic::error(
                                "trait methods cannot have type parameters yet",
                                m.name.span,
                            )
                            .with_code(DiagnosticCode::Type(crate::diagnostic_codes::TypeDiagnosticCode::TraitMethodTypeParamsUnsupported)),
                        );
                        continue;
                    }
                    let sig = type_from_expr(&mut ctx, &mut f_labels, &m.signature);
                    methods.insert(m.name.name.clone(), sig);
                }
                traits.insert(
                    t.name.name.clone(),
                    TraitInfo {
                        doc: t.doc.clone(),
                        type_params: tps,
                        capabilities,
                        methods,
                        self_ty,
                        span: t.name.span,
                    },
                );
            }
            Stmt::Impl(_) => {} // handled in later pass
            _ => {}
        }
    }

    // Constructors for enums/structs
    for (name, info) in enums.iter() {
        for (_idx, var) in info.variants.iter().enumerate() {
            let params = var.payload.iter().copied().collect::<Vec<TypeId>>();
            // 4 arguments: type_params, params, result, effect
            let func_ty = ctx.function(
                info.type_params.clone(),
                params.clone(),
                info.ty,
                Effect::Pure,
            );
            let vname = format!("{}::{}", name, var.name);
            if env.lookup_all_callables(&vname).is_empty() {
                env.insert_global(Binding {
                    name: vname.clone(),
                    ty: func_ty,
                    mutable: false,
                    no_shadow: false,
                    defined: true,
                    span: Span::dummy(),
                    kind: BindingKind::Func {
                        def_id: None,
                        symbol: vname.clone(),
                        effect: Effect::Pure,
                        arity: params.len(),
                        builtin: None,
                        field_accessor: None,
                        type_param_bounds: BTreeMap::new(),
                        captures: Vec::new(),
                    },
                });
            }
        }
    }

    let trait_semantics = TraitSemantics::detect(&traits);
    ctx.set_copy_trait_enabled(trait_semantics.has_any_copy_capability());

    // Process Impls separately or in the same loop?
    // Doing it here simplifies pending_if logic.
    pending_if = None;
    for item in &module.root.items {
        if let Stmt::Directive(d) = item {
            if let Some(allowed) = gate_allows(d, target, profile) {
                pending_if = Some(allowed);
                continue;
            }
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }
        if let Stmt::Impl(i) = item {
            if i.trait_ref.is_none() {
                diagnostics.push(
                    Diagnostic::error("inherent impl is not supported yet", i.span).with_code(
                        DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::ImplInherentUnsupported,
                        ),
                    ),
                );
                continue;
            }
            let trait_name = i.trait_ref.as_ref().map(|tr| tr.name.name.clone());
            let mut trait_self_ty = None;
            if let Some(tn) = &trait_name {
                if !traits.contains_key(tn) {
                    diagnostics.push(
                        Diagnostic::error(format!("unknown trait '{}'", tn), i.span).with_code(
                            DiagnosticCode::Type(
                                crate::diagnostic_codes::TypeDiagnosticCode::TraitUnknown,
                            ),
                        ),
                    );
                    continue;
                }
                trait_self_ty = traits.get(tn).map(|info| info.self_ty);
            }
            let mut f_labels = LabelEnv::new();
            let (_tps, _bounds_vec, _impl_bounds_map) = collect_type_params(
                &mut ctx,
                &mut f_labels,
                &i.type_params,
                &traits,
                &mut diagnostics,
            );
            let target_ty = type_from_expr(&mut ctx, &mut f_labels, &i.target_ty);
            let applied_trait_name = if let Some(trait_ref) = &i.trait_ref {
                let trait_info = traits.get(&trait_ref.name.name).unwrap();
                if trait_info.type_params.len() != trait_ref.args.len() {
                    diagnostics.push(
                        Diagnostic::error(
                            format!(
                                "trait '{}' expects {} type arguments, found {}",
                                trait_ref.name.name,
                                trait_info.type_params.len(),
                                trait_ref.args.len()
                            ),
                            trait_ref.name.span,
                        )
                        .with_code(DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::TraitTypeParamsUnsupported,
                        )),
                    );
                    continue;
                }
                let trait_args: Vec<TypeId> = trait_ref
                    .args
                    .iter()
                    .map(|arg| type_from_expr(&mut ctx, &mut f_labels, arg))
                    .collect();
                format_trait_ref_name(&trait_ref.name.name, &trait_args, &ctx)
            } else {
                trait_name.clone().unwrap_or_default()
            };
            f_labels.insert(String::from("Self"), target_ty);
            let generic_impl_target = type_contains_unbound_var(&ctx, target_ty);
            if generic_impl_target
                && !trait_semantics.has_copy_capability(trait_self_ty)
                && !trait_semantics.has_clone_capability(trait_self_ty)
                && !trait_semantics.has_drop_capability(trait_self_ty)
            {
                diagnostics.push(
                    Diagnostic::error("impl target type must be concrete", i.target_ty.span())
                        .with_code(DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::ImplTargetNotConcrete,
                        )),
                );
                continue;
            }
            if trait_semantics.has_copy_capability(trait_self_ty) {
                if !ctx.is_copy_impl_eligible(target_ty) {
                    diagnostics.push(
                        Diagnostic::error(
                            "copy impl target type is not copyable",
                            i.target_ty.span(),
                        )
                        .with_code(DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::CopyImplTargetNotCopy,
                        )),
                    );
                    push_unique_type(&ctx, &mut rejected_copy_targets, target_ty);
                    continue;
                }
                pending_copy_clone_checks.push((target_ty, i.span));
            }
            if impls.iter().any(|imp| {
                imp.trait_name.as_ref() == Some(&applied_trait_name)
                    && imp.trait_self_ty == trait_self_ty
                    && (ctx.type_pattern_matches(imp.target_ty, target_ty)
                        || ctx.type_pattern_matches(target_ty, imp.target_ty))
            }) {
                diagnostics.push(
                    Diagnostic::error("duplicate impl for same trait and target type", i.span)
                        .with_code(DiagnosticCode::Type(
                        crate::diagnostic_codes::TypeDiagnosticCode::ImplDuplicateForTraitTarget,
                    )),
                );
                duplicate_impl_spans.insert((i.span.file_id.0, i.span.start, i.span.end));
                continue;
            }

            impls.push(ImplInfo {
                trait_name: Some(applied_trait_name),
                trait_base_name: trait_name,
                trait_args: if let Some(trait_ref) = &i.trait_ref {
                    trait_ref
                        .args
                        .iter()
                        .map(|arg| type_from_expr(&mut ctx, &mut f_labels, arg))
                        .collect()
                } else {
                    Vec::new()
                },
                trait_self_ty,
                target_ty,
            });
        }
    }
    for (target_ty, span) in pending_copy_clone_checks {
        let has_clone_impl = impls.iter().any(|imp| {
            trait_semantics.has_clone_capability(imp.trait_self_ty)
                && (ctx.type_pattern_matches(imp.target_ty, target_ty)
                    || ctx.type_pattern_matches(target_ty, imp.target_ty))
        });
        if !has_clone_impl {
            diagnostics.push(
                Diagnostic::error(
                    "copy impl requires clone impl for the same target type",
                    span,
                )
                .with_code(DiagnosticCode::Type(
                    crate::diagnostic_codes::TypeDiagnosticCode::CopyImplRequiresClone,
                )),
            );
            push_unique_type(&ctx, &mut rejected_copy_targets, target_ty);
        }
    }
    impls.retain(|imp| {
        if !trait_semantics.has_copy_capability(imp.trait_self_ty) {
            return true;
        }
        !contains_same_type(&ctx, &rejected_copy_targets, imp.target_ty)
    });
    for imp in impls.iter() {
        if trait_semantics.has_copy_capability(imp.trait_self_ty) {
            ctx.register_copy_impl_target(imp.target_ty);
        }
        if trait_semantics.has_drop_capability(imp.trait_self_ty) {
            ctx.register_drop_impl_target(imp.target_ty);
        }
    }
    for (name, info) in structs.iter() {
        let func_ty = ctx.function(
            info.type_params.clone(),
            info.fields.clone(),
            info.ty,
            Effect::Pure,
        );
        if env.lookup_all_callables(name).is_empty() {
            env.insert_global(Binding {
                name: name.clone(),
                ty: func_ty,
                mutable: false,
                no_shadow: false,
                defined: true,
                span: Span::dummy(),
                kind: BindingKind::Func {
                    def_id: None,
                    symbol: name.clone(),
                    effect: Effect::Pure,
                    arity: info.fields.len(),
                    builtin: None,
                    field_accessor: None,
                    type_param_bounds: BTreeMap::new(),
                    captures: Vec::new(),
                },
            });
        }
    }

    let mut pending_if: Option<bool> = None;
    for item in &module.root.items {
        if let Stmt::Directive(d) = item {
            if let Some(allowed) = gate_allows(d, target, profile) {
                pending_if = Some(allowed);
                continue;
            }
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }
        if let Stmt::FnAlias(a) = item {
            fn_aliases.push(a);
            continue;
        }
        if let Stmt::FnDef(f) = item {
            let mut f_labels = LabelEnv::new();
            let (tps, _bounds_vec, bounds_map) = collect_type_params(
                &mut ctx,
                &mut f_labels,
                &f.type_params,
                &traits,
                &mut diagnostics,
            );

            let mut ty = type_from_expr(&mut ctx, &mut f_labels, &f.signature);
            // If it's a function type, we need to inject the type parameters
            if !tps.is_empty() {
                if let TypeKind::Function {
                    params,
                    result,
                    effect,
                    ..
                } = ctx.get(ty)
                {
                    ty = ctx.function(tps, params, result, effect);
                }
            }

            if let TypeKind::Function {
                type_params: _,
                params: _,
                result: _,
                effect,
            } = ctx.get(ty)
            {
                if env.lookup_value(&f.name.name).is_some() {
                    diagnostics.push(
                        Diagnostic::error("name already used by another item", f.name.span)
                            .with_code(DiagnosticCode::Resolve(
                                crate::diagnostic_codes::ResolveDiagnosticCode::ItemNameConflict,
                            )),
                    );
                    continue;
                }
                if enums.contains_key(&f.name.name) || structs.contains_key(&f.name.name) {
                    diagnostics.push(
                        Diagnostic::error("name already used by another item", f.name.span)
                            .with_code(DiagnosticCode::Resolve(
                                crate::diagnostic_codes::ResolveDiagnosticCode::ItemNameConflict,
                            )),
                    );
                    continue;
                }
                if crate::log::is_verbose() {
                    driver_log!("typecheck: registering global func {}", f.name.name);
                }
                if let Some(prev) = find_same_signature_func(&env, &f.name.name, ty, &ctx) {
                    diagnostics.push(
                        Diagnostic::warning(
                            format!(
                                "function '{}' with same signature is redefined (treated as shadowing)",
                                f.name.name
                            ),
                            f.name.span,
                        )
                        .with_secondary_label(
                            prev.span,
                            Some("previous definition with same signature".into()),
                        ),
                    );
                }
                if let Some(prev) =
                    find_invalid_same_file_overload(&env, &f.name.name, f.params.len(), f.name.span)
                {
                    diagnostics.push(
                        Diagnostic::error("ambiguous overload", f.name.span)
                            .with_code(DiagnosticCode::Type(
                                crate::diagnostic_codes::TypeDiagnosticCode::OverloadAmbiguous,
                            ))
                            .with_secondary_label(
                                prev.span,
                                Some("conflicting overload is defined here".into()),
                            ),
                    );
                    continue;
                }
                if let Some(blocked) = shadow_blocked_by_nonshadow(&env, &f.name.name) {
                    if is_callable_binding(blocked) {
                        if let Some(conflict) =
                            find_nonshadow_same_signature_func(&env, &f.name.name, ty, &ctx)
                        {
                            diagnostics.push(
                                Diagnostic::error(
                                    format!(
                                        "cannot shadow non-shadowable function '{}' with same signature",
                                        f.name.name
                                    ),
                                    f.name.span,
                                )
                                .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation)),
                            );
                            diagnostics.push(
                                Diagnostic::error(
                                    "non-shadowable function declaration is here",
                                    conflict.span,
                                )
                                .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation))
                                .with_secondary_label(f.name.span, Some("shadow attempt".into())),
                            );
                            continue;
                        }
                        // 関数同名はオーバーロードとして扱う（異なるシグネチャは許可）。
                    } else {
                        diagnostics.push(
                            Diagnostic::error(
                                format!("cannot shadow non-shadowable symbol '{}'", f.name.name),
                                f.name.span,
                            )
                            .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation)),
                        );
                        diagnostics.push(
                            Diagnostic::error("non-shadowable declaration is here", blocked.span)
                                .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation))
                                .with_secondary_label(f.name.span, Some("shadow attempt".into())),
                        );
                        continue;
                    }
                }
                if f.no_shadow
                    && (env
                        .lookup_all_any_defined(&f.name.name)
                        .iter()
                        .any(|b| !is_callable_binding(b))
                        || find_same_signature_func(&env, &f.name.name, ty, &ctx).is_some())
                {
                    diagnostics.push(
                        Diagnostic::error(
                            format!(
                                "noshadow declaration '{}' conflicts with existing symbol",
                                f.name.name
                            ),
                            f.name.span,
                        )
                        .with_code(DiagnosticCode::Resolve(
                            crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowConflict,
                        )),
                    );
                    continue;
                }
                env.remove_duplicate_func(&f.name.name, ty, f.name.span.file_id.0, &ctx);
                let symbol = mangle_function_symbol(&f.name.name, ty, &ctx);
                if crate::log::is_verbose() && f.name.name == "new" {
                    driver_log!(
                        "typecheck: registering global func new sig={}",
                        function_signature_string(&ctx, ty)
                    );
                }
                env.insert_global(Binding {
                    name: f.name.name.clone(),
                    ty,
                    mutable: false,
                    no_shadow: f.no_shadow,
                    defined: true,
                    span: f.name.span,
                    kind: BindingKind::Func {
                        def_id: DefId::from_span(f.name.span),
                        symbol,
                        effect,
                        arity: f.params.len(),
                        builtin: None,
                        field_accessor: detect_field_accessor_fn(f),
                        type_param_bounds: bounds_map.clone(),
                        captures: Vec::new(),
                    },
                });
            } else {
                diagnostics.push(
                    Diagnostic::error("function signature must be a function type", f.name.span)
                        .with_code(DiagnosticCode::Type(
                        crate::diagnostic_codes::TypeDiagnosticCode::FunctionSignatureNotFunction,
                    )),
                );
            }
        }
    }

    for alias in &fn_aliases {
        if enums.contains_key(&alias.name.name) || structs.contains_key(&alias.name.name) {
            diagnostics.push(
                Diagnostic::error("name already used by another item", alias.name.span).with_code(
                    DiagnosticCode::Resolve(
                        crate::diagnostic_codes::ResolveDiagnosticCode::ItemNameConflict,
                    ),
                ),
            );
            continue;
        }
        let targets = env.lookup_all_callables(&alias.target.name);
        if targets.is_empty() {
            diagnostics.push(
                Diagnostic::error("alias target not found", alias.target.span).with_code(
                    DiagnosticCode::Resolve(
                        crate::diagnostic_codes::ResolveDiagnosticCode::AliasTargetNotFound,
                    ),
                ),
            );
            continue;
        }
        let mut target_infos = Vec::new();
        for target in targets {
            let (symbol, effect, arity, builtin, field_accessor, bounds, captures) =
                match &target.kind {
                    BindingKind::Func {
                        symbol,
                        effect,
                        arity,
                        builtin,
                        field_accessor,
                        type_param_bounds,
                        captures,
                        ..
                    } => (
                        symbol.clone(),
                        *effect,
                        *arity,
                        *builtin,
                        *field_accessor,
                        type_param_bounds.clone(),
                        captures.clone(),
                    ),
                    _ => continue,
                };
            target_infos.push((
                target.ty,
                symbol,
                effect,
                arity,
                builtin,
                field_accessor,
                bounds,
                captures,
            ));
        }
        for (ty, symbol, effect, arity, builtin, field_accessor, bounds, captures) in target_infos {
            if let Some(prev) = find_same_signature_func(&env, &alias.name.name, ty, &ctx) {
                diagnostics.push(
                    Diagnostic::warning(
                        format!(
                            "function alias '{}' with same signature is redefined (treated as shadowing)",
                            alias.name.name
                        ),
                        alias.name.span,
                    )
                    .with_secondary_label(
                        prev.span,
                        Some("previous definition with same signature".into()),
                    ),
                );
            }
            if env.lookup_value(&alias.name.name).is_some() {
                diagnostics.push(
                    Diagnostic::error("name already used by another item", alias.name.span)
                        .with_code(DiagnosticCode::Resolve(
                            crate::diagnostic_codes::ResolveDiagnosticCode::ItemNameConflict,
                        )),
                );
                break;
            }
            if let Some(blocked) = shadow_blocked_by_nonshadow(&env, &alias.name.name) {
                if is_callable_binding(blocked) {
                    if let Some(conflict) =
                        find_nonshadow_same_signature_func(&env, &alias.name.name, ty, &ctx)
                    {
                        diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "cannot shadow non-shadowable function alias '{}' with same signature",
                                    alias.name.name
                                ),
                                alias.name.span,
                            )
                            .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation)),
                        );
                        diagnostics.push(
                            Diagnostic::error(
                                "non-shadowable function declaration is here",
                                conflict.span,
                            )
                            .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation))
                            .with_secondary_label(alias.name.span, Some("shadow attempt".into())),
                        );
                        break;
                    }
                    // 関数同名はオーバーロードとして扱う（異なるシグネチャは許可）。
                } else {
                    diagnostics.push(
                        Diagnostic::error(
                            format!("cannot shadow non-shadowable symbol '{}'", alias.name.name),
                            alias.name.span,
                        )
                        .with_code(DiagnosticCode::Resolve(
                            crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation,
                        )),
                    );
                    diagnostics.push(
                        Diagnostic::error("non-shadowable declaration is here", blocked.span)
                            .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation))
                            .with_secondary_label(alias.name.span, Some("shadow attempt".into())),
                    );
                    break;
                }
            }
            if alias.no_shadow
                && (env
                    .lookup_all_any_defined(&alias.name.name)
                    .iter()
                    .any(|b| !is_callable_binding(b))
                    || find_same_signature_func(&env, &alias.name.name, ty, &ctx).is_some())
            {
                diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "noshadow declaration '{}' conflicts with existing symbol",
                            alias.name.name
                        ),
                        alias.name.span,
                    )
                    .with_code(DiagnosticCode::Resolve(
                        crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowConflict,
                    )),
                );
                break;
            }
            env.remove_duplicate_func(&alias.name.name, ty, alias.name.span.file_id.0, &ctx);
            env.insert_global(Binding {
                name: alias.name.name.clone(),
                ty,
                mutable: false,
                no_shadow: alias.no_shadow,
                defined: true,
                span: alias.name.span,
                kind: BindingKind::Func {
                    def_id: DefId::from_span(alias.name.span),
                    symbol,
                    effect,
                    arity,
                    builtin,
                    field_accessor,
                    type_param_bounds: bounds,
                    captures,
                },
            });
        }
    }

    let mut functions = Vec::new();
    let mut pending_if = None;
    for item in &module.root.items {
        if let Stmt::Directive(d) = item {
            if let Some(allowed) = gate_allows(d, target, profile) {
                pending_if = Some(allowed);
                continue;
            }
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }
        if let Stmt::FnDef(f) = item {
            let f_ty = {
                let mut funcs: Vec<&Binding> = env.lookup_all_callables(&f.name.name);
                if funcs.is_empty() {
                    // The function was not hoisted (due to a prior error such as
                    // duplicate name). Skip type-checking its body to avoid panics.
                    continue;
                }
                if funcs.len() == 1 {
                    funcs[0].ty
                } else {
                    let mut tmp_labels = LabelEnv::new();
                    let mut sig_type_params = Vec::new();
                    for tp in &f.type_params {
                        let tv = ctx.fresh_var(Some(tp.name.name.clone()));
                        tmp_labels.insert(tp.name.name.clone(), tv);
                        sig_type_params.push(tv);
                    }
                    let sig_ty = match f.signature.as_unspanned() {
                        TypeExpr::Function {
                            params,
                            result,
                            effect,
                        } => {
                            let mut sig_params = Vec::new();
                            for p in params {
                                sig_params.push(type_from_expr(&mut ctx, &mut tmp_labels, p));
                            }
                            let sig_result = type_from_expr(&mut ctx, &mut tmp_labels, result);
                            ctx.function(sig_type_params, sig_params, sig_result, *effect)
                        }
                        _ => type_from_expr(&mut ctx, &mut tmp_labels, &f.signature),
                    };
                    let mut matched: Option<TypeId> = None;
                    for binding in funcs.drain(..) {
                        if same_function_signature(&ctx, binding.ty, sig_ty) {
                            matched = Some(binding.ty);
                            break;
                        }
                    }
                    match matched {
                        Some(ty) => ty,
                        None => {
                            diagnostics.push(
                                Diagnostic::error(
                                    "function signature does not match any overload",
                                    f.name.span,
                                )
                                .with_code(DiagnosticCode::Type(crate::diagnostic_codes::TypeDiagnosticCode::FunctionSignatureOverloadNotFound)),
                            );
                            continue;
                        }
                    }
                }
            };
            let mut type_param_bounds = BTreeMap::new();
            if let TypeKind::Function { type_params, .. } = ctx.get(f_ty) {
                for (p_node, p_id) in f.type_params.iter().zip(type_params.iter()) {
                    label_env.insert(p_node.name.name.clone(), *p_id);
                    if !p_node.bounds.is_empty() {
                        let mut bounds = Vec::new();
                        for b in &p_node.bounds {
                            if let Some(info) = traits.get(&b.name.name) {
                                if info.type_params.len() != b.args.len() {
                                    diagnostics.push(
                                        Diagnostic::error(
                                            format!(
                                                "trait bound '{}' expects {} type arguments, found {}",
                                                b.name.name,
                                                info.type_params.len(),
                                                b.args.len()
                                            ),
                                            b.name.span,
                                        )
                                        .with_code(DiagnosticCode::Type(crate::diagnostic_codes::TypeDiagnosticCode::TraitTypeParamsUnsupported)),
                                    );
                                    continue;
                                }
                                let arg_tys: Vec<TypeId> = b
                                    .args
                                    .iter()
                                    .map(|arg| type_from_expr(&mut ctx, &mut label_env, arg))
                                    .collect();
                                bounds.push(TraitBoundRef {
                                    name: format_trait_ref_name(&b.name.name, &arg_tys, &ctx),
                                    trait_base_name: b.name.name.clone(),
                                    trait_args: arg_tys,
                                    trait_self_ty: info.self_ty,
                                });
                            }
                        }
                        if !bounds.is_empty() {
                            type_param_bounds.insert(*p_id, bounds);
                        }
                    }
                }
            }
            let mut nested_functions = Vec::new();
            match check_function(
                f,
                f_ty,
                entry
                    .as_ref()
                    .map(|(n, _)| n == &f.name.name)
                    .unwrap_or(false),
                target,
                profile,
                &[],
                &mut ctx,
                &mut env,
                &mut label_env,
                &mut strings,
                &enums,
                &structs,
                &mut instantiations,
                type_param_bounds,
                &traits,
                &impls,
                &mut nested_functions,
                &import_resolution,
                source_map,
            ) {
                Ok(checked) => {
                    diagnostics.extend(checked.diagnostics);
                    functions.push(checked.function);
                    functions.extend(nested_functions);
                }
                Err(mut diags) => diagnostics.append(&mut diags),
            }
        }
    }

    let mut final_traits = Vec::new();
    for (name, info) in traits.iter() {
        final_traits.push(HirTrait {
            doc: info.doc.clone(),
            name: name.clone(),
            type_params: info.type_params.clone(),
            capabilities: info
                .capabilities
                .iter()
                .map(|cap| match cap {
                    TraitCapability::Copy => crate::ast::TraitCapability::Copy,
                    TraitCapability::Clone => crate::ast::TraitCapability::Clone,
                    TraitCapability::Drop => crate::ast::TraitCapability::Drop,
                })
                .collect(),
            methods: info.methods.clone(),
            span: info.span,
        });
    }

    let mut final_impls = Vec::new();
    pending_if = None;
    for item in &module.root.items {
        if let Stmt::Directive(d) = item {
            if let Some(allowed) = gate_allows(d, target, profile) {
                pending_if = Some(allowed);
                continue;
            }
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }
        if let Stmt::Impl(i) = item {
            let impl_key = (i.span.file_id.0, i.span.start, i.span.end);
            if duplicate_impl_spans.contains(&impl_key) {
                continue;
            }
            let trait_ref = match &i.trait_ref {
                Some(tr) => tr,
                None => {
                    diagnostics.push(
                        Diagnostic::error("inherent impl is not supported yet", i.span)
                            .with_code(DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::ImplInherentUnsupported,
                        )),
                    );
                    continue;
                }
            };
            let trait_name = trait_ref.name.name.clone();
            let trait_info = match traits.get(&trait_name) {
                Some(info) => info,
                None => {
                    diagnostics.push(
                        Diagnostic::error(format!("unknown trait '{}'", trait_name), i.span)
                            .with_code(DiagnosticCode::Type(
                                crate::diagnostic_codes::TypeDiagnosticCode::TraitUnknown,
                            )),
                    );
                    continue;
                }
            };
            let mut impl_methods = Vec::new();
            let mut f_labels = LabelEnv::new();
            let (tps, _bounds_vec, impl_bounds_map) = collect_type_params(
                &mut ctx,
                &mut f_labels,
                &i.type_params,
                &traits,
                &mut diagnostics,
            );
            let target_ty = type_from_expr(&mut ctx, &mut f_labels, &i.target_ty);
            if trait_info.type_params.len() != trait_ref.args.len() {
                diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "trait '{}' expects {} type arguments, found {}",
                            trait_name,
                            trait_info.type_params.len(),
                            trait_ref.args.len()
                        ),
                        trait_ref.name.span,
                    )
                    .with_code(DiagnosticCode::Type(
                        crate::diagnostic_codes::TypeDiagnosticCode::TraitTypeParamsUnsupported,
                    )),
                );
                continue;
            }
            let trait_args: Vec<TypeId> = trait_ref
                .args
                .iter()
                .map(|arg| type_from_expr(&mut ctx, &mut f_labels, arg))
                .collect();
            let applied_trait_name = format_trait_ref_name(&trait_name, &trait_args, &ctx);
            if type_contains_unbound_var(&ctx, target_ty)
                && !trait_semantics.has_copy_capability(Some(trait_info.self_ty))
                && !trait_semantics.has_clone_capability(Some(trait_info.self_ty))
                && !trait_semantics.has_drop_capability(Some(trait_info.self_ty))
            {
                diagnostics.push(
                    Diagnostic::error("impl target type must be concrete", i.target_ty.span())
                        .with_code(DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::ImplTargetNotConcrete,
                        )),
                );
                continue;
            }
            if trait_semantics.has_copy_capability(Some(trait_info.self_ty)) {
                if contains_same_type(&ctx, &rejected_copy_targets, target_ty) {
                    continue;
                }
            }
            f_labels.insert(String::from("Self"), target_ty);
            let prev_self = label_env.insert(String::from("Self"), target_ty);

            let mut seen_methods = BTreeSet::new();
            for m in &i.methods {
                if !seen_methods.insert(m.name.name.clone()) {
                    diagnostics.push(
                        Diagnostic::error("duplicate method in impl", m.name.span).with_code(
                            DiagnosticCode::Type(
                                crate::diagnostic_codes::TypeDiagnosticCode::ImplDuplicateMethod,
                            ),
                        ),
                    );
                    continue;
                }
                if !m.type_params.is_empty() {
                    diagnostics.push(
                        Diagnostic::error(
                            "impl methods cannot have type parameters yet",
                            m.name.span,
                        )
                        .with_code(DiagnosticCode::Type(crate::diagnostic_codes::TypeDiagnosticCode::TraitMethodTypeParamsUnsupported)),
                    );
                    continue;
                }
                let trait_sig = match trait_info.methods.get(&m.name.name) {
                    Some(sig) => *sig,
                    None => {
                        diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "method '{}' not found in trait '{}'",
                                    m.name.name, trait_name
                                ),
                                m.name.span,
                            )
                            .with_code(DiagnosticCode::Type(
                                crate::diagnostic_codes::TypeDiagnosticCode::ImplMethodNotInTrait,
                            )),
                        );
                        continue;
                    }
                };
                let mut mapping = BTreeMap::new();
                mapping.insert(
                    ctx.resolve_id(trait_info.self_ty),
                    ctx.resolve_id(target_ty),
                );
                for (tp, arg) in trait_info.type_params.iter().zip(trait_args.iter()) {
                    insert_substitution_mapping(&ctx, &mut mapping, *tp, *arg);
                }
                let expected_sig = ctx.substitute(trait_sig, &mapping);
                let actual_sig = type_from_expr(&mut ctx, &mut f_labels, &m.signature);
                if !ctx.same_type(expected_sig, actual_sig) {
                    diagnostics.push(
                        Diagnostic::error(
                            "impl method signature does not match trait",
                            m.name.span,
                        )
                        .with_code(DiagnosticCode::Type(crate::diagnostic_codes::TypeDiagnosticCode::ImplMethodSignatureMismatch)),
                    );
                    continue;
                }
                let checked_sig = match ctx.get(expected_sig) {
                    TypeKind::Function {
                        params,
                        result,
                        effect,
                        ..
                    } => ctx.function(tps.clone(), params.clone(), result, effect),
                    _ => expected_sig,
                };
                let mut nested_functions = Vec::new();
                let checked = match check_function(
                    m,
                    checked_sig,
                    false,
                    target,
                    profile,
                    &[],
                    &mut ctx,
                    &mut env,
                    &mut label_env,
                    &mut strings,
                    &enums,
                    &structs,
                    &mut instantiations,
                    impl_bounds_map.clone(),
                    &traits,
                    &impls,
                    &mut nested_functions,
                    &import_resolution,
                    source_map,
                ) {
                    Ok(checked) => checked,
                    Err(mut diags) => {
                        diagnostics.append(&mut diags);
                        continue;
                    }
                };
                diagnostics.extend(checked.diagnostics);
                let mut func = checked.function;
                let mangled =
                    mangle_impl_method(&applied_trait_name, &m.name.name, target_ty, &ctx);
                func.name = mangled.clone();
                functions.push(func.clone());
                functions.extend(nested_functions);
                impl_methods.push(HirImplMethod {
                    name: m.name.name.clone(),
                    func,
                });
            }

            for trait_method in trait_info.methods.keys() {
                if !seen_methods.contains(trait_method) {
                    diagnostics.push(
                        Diagnostic::error(
                            format!(
                                "missing method '{}' for trait '{}'",
                                trait_method, trait_name
                            ),
                            i.span,
                        )
                        .with_code(DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::ImplMissingTraitMethod,
                        )),
                    );
                }
            }

            if let Some(prev) = prev_self {
                label_env.insert(String::from("Self"), prev);
            } else {
                label_env.remove("Self");
            }

            final_impls.push(HirImpl {
                doc: i.doc.clone(),
                trait_name: applied_trait_name,
                trait_base_name: Some(trait_name.clone()),
                trait_args: trait_args.clone(),
                type_args: tps,
                target_ty,
                methods: impl_methods,
                span: i.target_ty.span(),
            });
        }
    }

    let resolved_entry =
        resolve_entry_function(module, target, profile, &env, entry, &mut diagnostics);

    let has_error = diagnostics
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error));
    if has_error && crate::log::is_verbose() {
        print_diagnostics_summary(&diagnostics);
    }

    TypeCheckResult {
        module: if has_error {
            None
        } else {
            Some(HirModule {
                functions,
                entry: resolved_entry,
                externs,
                string_literals: strings.into_vec(),
                traits: final_traits,
                impls: final_impls,
            })
        },
        diagnostics,
        types: ctx,
    }
}
