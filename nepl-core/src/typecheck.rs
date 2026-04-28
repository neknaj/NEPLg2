extern crate alloc;
#[cfg(not(target_os = "none"))]
extern crate std;

mod ascription;
mod binding_rules;
mod call_reduction;
mod call_resolution;
mod effect_check;
mod env;
mod field_access;
mod function_apply;
mod hir_finalize;
mod match_check;
mod name_lookup;
mod prefix_check;
mod signature;
mod syntax_helpers;
mod trait_check;
mod traits;
mod type_expr;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::*;
use crate::compiler::{BuildProfile, CompileTarget};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::*;
use crate::resolve::{DefId, ImportResolution};
use crate::source_map::SourceMap;
use crate::span::Span;
use crate::types::{EnumVariantInfo, TypeCtx, TypeId, TypeKind};

use binding_rules::{
    detect_field_accessor_fn, emit_shadow_warning, find_invalid_same_file_overload,
    find_nonshadow_same_signature_func, find_same_signature_func, is_callable_binding,
    shadow_blocked_by_nonshadow,
};
use env::{Binding, BindingKind, Env};
use hir_finalize::resolve_type_ids_in_function;
use signature::{
    contains_same_type, function_signature_string, mangle_function_symbol, mangle_impl_method,
    push_unique_type, same_function_signature, type_contains_unbound_var,
};
use syntax_helpers::gate_allows;
use traits::{
    collect_type_params, format_trait_ref_name, insert_substitution_mapping,
    trait_application_matches, type_param_has_trait_bound, ImplInfo, TraitBoundRef,
    TraitCapability, TraitInfo, TraitSemantics,
};
use type_expr::{type_from_expr, LabelEnv, StringTable};

// Helper to gate verbose HIR dumps. Use `dump!(...)` for noisy debug output
// that should only appear when `NEPL_DUMP_HIR` is set.
fn dump_enabled() -> bool {
    #[cfg(target_os = "none")]
    {
        false
    }
    #[cfg(not(target_os = "none"))]
    {
        std::env::var("NEPL_DUMP_HIR").is_ok()
    }
}

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

macro_rules! dump {
    ($($arg:tt)*) => {
        if dump_enabled() {
            typecheck_log!($($arg)*);
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScalarMatchKind {
    I32,
    U8,
    Bool,
    Char,
}

#[cfg(not(target_os = "none"))]
fn print_diagnostics_summary(diags: &alloc::vec::Vec<crate::diagnostic::Diagnostic>) {
    if diags.is_empty() {
        return;
    }
    // Print a short, readable summary of diagnostics (one line per diagnostic)
    typecheck_log!("Compiler diagnostics:");
    for d in diags.iter() {
        let sev = match d.severity {
            crate::diagnostic::Severity::Error => "error",
            crate::diagnostic::Severity::Warning => "warning",
        };
        // Display primary span as file_id:start-end for quick location.
        let span = &d.primary.span;
        typecheck_log!(
            "- {}: {} (span: {:?}:{:?}-{:?})",
            sev,
            d.message,
            span.file_id,
            span.start,
            span.end
        );
        for sec in d.secondary.iter() {
            typecheck_log!(
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

#[derive(Debug)]
struct CheckedFunction {
    function: HirFunction,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
struct EnumInfo {
    ty: TypeId,
    type_params: Vec<TypeId>,
    variants: Vec<EnumVariantInfo>,
}

#[derive(Debug, Clone)]
struct StructInfo {
    ty: TypeId,
    type_params: Vec<TypeId>,
    fields: Vec<TypeId>,
    field_names: Vec<String>,
}

#[derive(Debug, Clone)]
enum FieldIdx {
    Index(usize),
    Name(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldAccessorKind {
    Get,
    GetRef,
    Put,
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
                        .with_id(DiagnosticId::TypeWasiImportTargetMismatch),
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
                    Diagnostic::error("extern signature must be a function type", *span)
                        .with_id(DiagnosticId::TypeExternSignatureMustBeFunction),
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
                            .with_id(DiagnosticId::TypeItemNameConflict),
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
                            .with_id(DiagnosticId::TypeEnumTypeParamBoundsUnsupported),
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
                            .with_id(DiagnosticId::TypeItemNameConflict),
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
                            .with_id(DiagnosticId::TypeStructTypeParamBoundsUnsupported),
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
                                .with_id(DiagnosticId::TypeUnknownTraitCapability),
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
                            .with_id(DiagnosticId::TypeTraitMethodTypeParamsUnsupported),
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
                    Diagnostic::error("inherent impl is not supported yet", i.span)
                        .with_id(DiagnosticId::TypeInherentImplUnsupported),
                );
                continue;
            }
            let trait_name = i.trait_ref.as_ref().map(|tr| tr.name.name.clone());
            let mut trait_self_ty = None;
            if let Some(tn) = &trait_name {
                if !traits.contains_key(tn) {
                    diagnostics.push(
                        Diagnostic::error(format!("unknown trait '{}'", tn), i.span)
                            .with_id(DiagnosticId::TypeUnknownTrait),
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
                        .with_id(DiagnosticId::TypeTraitTypeParamsUnsupported),
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
                        .with_id(DiagnosticId::TypeImplTargetMustBeConcrete),
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
                        .with_id(DiagnosticId::TypeCopyImplTargetNotCopy),
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
                        .with_id(DiagnosticId::TypeDuplicateImplForTraitTarget),
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
                .with_id(DiagnosticId::TypeCopyImplRequiresClone),
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
                            .with_id(DiagnosticId::TypeItemNameConflict),
                    );
                    continue;
                }
                if enums.contains_key(&f.name.name) || structs.contains_key(&f.name.name) {
                    diagnostics.push(
                        Diagnostic::error("name already used by another item", f.name.span)
                            .with_id(DiagnosticId::TypeItemNameConflict),
                    );
                    continue;
                }
                if crate::log::is_verbose() {
                    typecheck_log!("typecheck: registering global func {}", f.name.name);
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
                            .with_id(DiagnosticId::TypeAmbiguousOverload)
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
                                .with_id(DiagnosticId::TypeNoShadowViolation),
                            );
                            diagnostics.push(
                                Diagnostic::error(
                                    "non-shadowable function declaration is here",
                                    conflict.span,
                                )
                                .with_id(DiagnosticId::TypeNoShadowViolation)
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
                            .with_id(DiagnosticId::TypeNoShadowViolation),
                        );
                        diagnostics.push(
                            Diagnostic::error("non-shadowable declaration is here", blocked.span)
                                .with_id(DiagnosticId::TypeNoShadowViolation)
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
                        .with_id(DiagnosticId::TypeNoShadowConflict),
                    );
                    continue;
                }
                env.remove_duplicate_func(&f.name.name, ty, &ctx);
                let symbol = mangle_function_symbol(&f.name.name, ty, &ctx);
                if crate::log::is_verbose() && f.name.name == "new" {
                    typecheck_log!(
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
                        .with_id(DiagnosticId::TypeFunctionSignatureMustBeFunction),
                );
            }
        }
    }

    for alias in &fn_aliases {
        if enums.contains_key(&alias.name.name) || structs.contains_key(&alias.name.name) {
            diagnostics.push(
                Diagnostic::error("name already used by another item", alias.name.span)
                    .with_id(DiagnosticId::TypeItemNameConflict),
            );
            continue;
        }
        let targets = env.lookup_all_callables(&alias.target.name);
        if targets.is_empty() {
            diagnostics.push(
                Diagnostic::error("alias target not found", alias.target.span)
                    .with_id(DiagnosticId::TypeAliasTargetNotFound),
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
                        .with_id(DiagnosticId::TypeItemNameConflict),
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
                            .with_id(DiagnosticId::TypeNoShadowViolation),
                        );
                        diagnostics.push(
                            Diagnostic::error(
                                "non-shadowable function declaration is here",
                                conflict.span,
                            )
                            .with_id(DiagnosticId::TypeNoShadowViolation)
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
                        .with_id(DiagnosticId::TypeNoShadowViolation),
                    );
                    diagnostics.push(
                        Diagnostic::error("non-shadowable declaration is here", blocked.span)
                            .with_id(DiagnosticId::TypeNoShadowViolation)
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
                    .with_id(DiagnosticId::TypeNoShadowConflict),
                );
                break;
            }
            env.remove_duplicate_func(&alias.name.name, ty, &ctx);
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
                                .with_id(DiagnosticId::TypeFunctionSignatureOverloadNotFound),
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
                                        .with_id(DiagnosticId::TypeTraitTypeParamsUnsupported),
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
                            .with_id(DiagnosticId::TypeInherentImplUnsupported),
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
                            .with_id(DiagnosticId::TypeUnknownTrait),
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
                    .with_id(DiagnosticId::TypeTraitTypeParamsUnsupported),
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
                        .with_id(DiagnosticId::TypeImplTargetMustBeConcrete),
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
                        Diagnostic::error("duplicate method in impl", m.name.span)
                            .with_id(DiagnosticId::TypeDuplicateImplMethod),
                    );
                    continue;
                }
                if !m.type_params.is_empty() {
                    diagnostics.push(
                        Diagnostic::error(
                            "impl methods cannot have type parameters yet",
                            m.name.span,
                        )
                        .with_id(DiagnosticId::TypeTraitMethodTypeParamsUnsupported),
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
                            .with_id(DiagnosticId::TypeImplMethodNotFoundInTrait),
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
                        .with_id(DiagnosticId::TypeImplMethodSignatureMismatch),
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
                        .with_id(DiagnosticId::TypeImplMissingTraitMethod),
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

    let resolved_entry = if let Some((name, entry_span)) = entry {
        let bindings = env.lookup_all_callables(&name);
        let mut func_symbols = Vec::new();
        for b in bindings {
            if let BindingKind::Func { symbol, .. } = &b.kind {
                func_symbols.push(symbol.clone());
            }
        }
        if func_symbols.len() == 1 {
            Some(func_symbols.remove(0))
        } else if top_level_llvmir_defines_entry(module, target, profile, name.as_str()) {
            None
        } else {
            diagnostics.push(
                Diagnostic::error("entry function is missing or ambiguous", entry_span)
                    .with_id(DiagnosticId::TypeEntryFunctionMissingOrAmbiguous),
            );
            None
        }
    } else {
        None
    };

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

fn top_level_llvmir_defines_entry(
    module: &crate::ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
    entry: &str,
) -> bool {
    if !matches!(target, CompileTarget::Llvm) {
        return false;
    }
    for idx in crate::target_precheck::active_stmt_indices(&module.root, target, profile) {
        if let Stmt::LlvmIr(block) = &module.root.items[idx] {
            for line in &block.lines {
                if crate::llvm_ir::parse_defined_function_name(line) == Some(entry) {
                    return true;
                }
            }
        }
    }
    false
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

impl<'a> BlockChecker<'a> {
    fn check_block(
        &mut self,
        block: &Block,
        base_depth: usize,
        new_scope: bool,
        expected_last_ty: Option<TypeId>,
    ) -> Option<(HirBlock, Option<TypeId>)> {
        let old_effect = self.current_effect;

        if new_scope {
            self.env.push_scope();
        }

        // Hoist let (non-mut) and nested fn signatures
        for stmt in block.items.iter() {
            if let Stmt::Expr(PrefixExpr { items, .. })
            | Stmt::ExprSemi(PrefixExpr { items, .. }, _) = stmt
            {
                if let Some(PrefixItem::Symbol(Symbol::Let {
                    name,
                    mutable: false,
                    no_shadow,
                })) = items.first()
                {
                    if let Some(blocked) = shadow_blocked_by_nonshadow(self.env, &name.name) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!("cannot shadow non-shadowable symbol '{}'", name.name),
                                name.span,
                            )
                            .with_id(DiagnosticId::TypeNoShadowViolation),
                        );
                        self.diagnostics.push(
                            Diagnostic::error("non-shadowable declaration is here", blocked.span)
                                .with_id(DiagnosticId::TypeNoShadowViolation)
                                .with_secondary_label(name.span, Some("shadow attempt".into())),
                        );
                        continue;
                    }
                    if *no_shadow && self.env.lookup_any(&name.name).is_some() {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "noshadow declaration '{}' conflicts with existing symbol",
                                    name.name
                                ),
                                name.span,
                            )
                            .with_id(DiagnosticId::TypeNoShadowConflict),
                        );
                        continue;
                    }
                    let ty = self.ctx.fresh_var(None);
                    emit_shadow_warning(
                        &mut self.diagnostics,
                        self.env,
                        &name.name,
                        name.span,
                        "let",
                    );
                    let _ = self.env.insert_local(Binding {
                        name: name.name.clone(),
                        ty,
                        mutable: false,
                        no_shadow: *no_shadow,
                        defined: false,
                        span: name.span,
                        kind: BindingKind::Var,
                    });
                    dump!("typecheck: hoisted binding {}", name.name);
                }
            } else if let Stmt::FnAlias(_) = stmt {
                // function alias is handled at top-level
            } else if let Stmt::FnDef(f) = stmt {
                if !f.type_params.is_empty() {
                    self.diagnostics.push(Diagnostic::error(
                        "nested generic functions are not supported yet",
                        f.name.span,
                    ));
                    continue;
                }
                let base_ty = type_from_expr(self.ctx, self.labels, &f.signature);
                let captures = self.collect_nested_fn_captures(f);
                let mut ty = base_ty;
                if let TypeKind::Function {
                    type_params,
                    params,
                    result,
                    effect,
                } = self.ctx.get(base_ty)
                {
                    if let Some(blocked) = shadow_blocked_by_nonshadow(self.env, &f.name.name) {
                        if is_callable_binding(blocked) {
                            if let Some(conflict) = find_nonshadow_same_signature_func(
                                self.env,
                                &f.name.name,
                                ty,
                                self.ctx,
                            ) {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "cannot shadow non-shadowable function '{}' with same signature",
                                            f.name.name
                                        ),
                                        f.name.span,
                                    )
                                    .with_id(DiagnosticId::TypeNoShadowViolation),
                                );
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "non-shadowable function declaration is here",
                                        conflict.span,
                                    )
                                    .with_id(DiagnosticId::TypeNoShadowViolation)
                                    .with_secondary_label(
                                        f.name.span,
                                        Some("shadow attempt".into()),
                                    ),
                                );
                                continue;
                            }
                            // 関数同名はオーバーロードとして扱う（異なるシグネチャは許可）。
                        } else {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    format!(
                                        "cannot shadow non-shadowable symbol '{}'",
                                        f.name.name
                                    ),
                                    f.name.span,
                                )
                                .with_id(DiagnosticId::TypeNoShadowViolation),
                            );
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "non-shadowable declaration is here",
                                    blocked.span,
                                )
                                .with_id(DiagnosticId::TypeNoShadowViolation)
                                .with_secondary_label(f.name.span, Some("shadow attempt".into())),
                            );
                            continue;
                        }
                    }
                    if f.no_shadow
                        && (self
                            .env
                            .lookup_all_any_defined(&f.name.name)
                            .iter()
                            .any(|b| !is_callable_binding(b))
                            || find_same_signature_func(self.env, &f.name.name, ty, self.ctx)
                                .is_some())
                    {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "noshadow declaration '{}' conflicts with existing symbol",
                                    f.name.name
                                ),
                                f.name.span,
                            )
                            .with_id(DiagnosticId::TypeNoShadowConflict),
                        );
                        continue;
                    }
                    if !captures.is_empty() {
                        let mut lifted_params =
                            captures.iter().map(|(_, t)| *t).collect::<Vec<_>>();
                        lifted_params.extend(params.iter().copied());
                        ty = self
                            .ctx
                            .function(type_params.clone(), lifted_params, result, effect);
                    }
                    let has_non_callable_conflict = self
                        .env
                        .lookup_all_any_defined(&f.name.name)
                        .iter()
                        .any(|b| !is_callable_binding(b));
                    if has_non_callable_conflict {
                        emit_shadow_warning(
                            &mut self.diagnostics,
                            self.env,
                            &f.name.name,
                            f.name.span,
                            "fn",
                        );
                    }
                    let _ = self.env.insert_local(Binding {
                        name: f.name.name.clone(),
                        ty,
                        mutable: false,
                        no_shadow: f.no_shadow,
                        defined: true,
                        span: f.name.span,
                        kind: BindingKind::Func {
                            def_id: DefId::from_span(f.name.span),
                            symbol: f.name.name.clone(),
                            effect,
                            arity: f.params.len(),
                            builtin: None,
                            field_accessor: detect_field_accessor_fn(f),
                            type_param_bounds: BTreeMap::new(),
                            captures,
                        },
                    });
                }
            }
        }

        let mut lines = Vec::new();
        let mut stack: Vec<StackEntry> = Vec::new();
        for _ in 0..base_depth {
            stack.push(StackEntry {
                ty: self.ctx.unit(),
                expr: HirExpr {
                    ty: self.ctx.unit(),
                    kind: HirExprKind::Unit,
                    span: block.span,
                },
                type_args: Vec::new(),
                assign: None,
                auto_call: true,
            });
        }

        // Find the last expression statement index (it determines the block result)
        let last_expr_idx = block
            .items
            .iter()
            .rposition(|s| matches!(s, Stmt::Expr(_) | Stmt::ExprSemi(_, _)));

        let mut pending_if: Option<bool> = None;
        for (idx, stmt) in block.items.iter().enumerate() {
            if let Stmt::Directive(d) = stmt {
                if let Some(allowed) = gate_allows(d, self.target, self.profile) {
                    pending_if = Some(allowed);
                    continue;
                }
            }
            let allowed = pending_if.unwrap_or(true);
            pending_if = None;
            if !allowed {
                continue;
            }

            // Drop stray unit between lines: [X, ()] -> [X]
            if stack.len() == base_depth + 1 {
                if matches!(self.ctx.get(stack.last().unwrap().ty), TypeKind::Unit) {
                    stack.pop();
                }
            }

            match stmt {
                Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
                    let expected_stmt_ty = if Some(idx) == last_expr_idx {
                        expected_last_ty
                    } else {
                        None
                    };
                    match self.check_prefix(expr, base_depth, &mut stack, expected_stmt_ty) {
                        Some((typed, dropped_from_prefix)) => {
                            let is_last_expr = Some(idx) == last_expr_idx;
                            let mut drop_result = !is_last_expr;
                            if matches!(stmt, Stmt::ExprSemi(_, _)) {
                                // `;` explicitly discards the statement value even
                                // when it appears on the last line of a block.
                                drop_result = true;
                            }

                            if dropped_from_prefix {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "expression left extra values on the stack",
                                        typed.span,
                                    )
                                    .with_id(DiagnosticId::TypeStackExtraValues),
                                );
                            }

                            // If there was an explicit semicolon token, require that the
                            // statement left exactly one value on the stack; otherwise
                            // emit a diagnostic and recover.
                            if let Stmt::ExprSemi(_, semi_span) = stmt {
                                if stack.len() != base_depth + 1 {
                                    let sp = semi_span.unwrap_or(typed.span);
                                    self.diagnostics.push(
                                        Diagnostic::error(
                                            "statement must leave exactly one value on the stack",
                                            sp,
                                        )
                                        .with_id(DiagnosticId::TypeStackExtraValues),
                                    );
                                    while stack.len() > base_depth {
                                        stack.pop();
                                    }
                                    drop_result = true;
                                }
                            }

                            if drop_result {
                                // Pop all values down to base_depth
                                while stack.len() > base_depth {
                                    let _ = stack.pop();
                                }
                            }

                            // Previously a fallback was here; lower-let is handled in `check_prefix`

                            lines.push(HirLine {
                                expr: typed,
                                drop_result,
                            });
                        }
                        None => {}
                    }
                }
                Stmt::Directive(_) => {}
                Stmt::FnAlias(_) => {}
                Stmt::FnDef(f) => {
                    // capture 型はホイスト時点では未確定になり得るため、
                    // 実チェック時に現在の環境から再計算する。
                    let captures = self.collect_nested_fn_captures(f);
                    let mut f_ty = type_from_expr(self.ctx, self.labels, &f.signature);
                    if let TypeKind::Function {
                        type_params,
                        params,
                        result,
                        effect,
                    } = self.ctx.get(f_ty)
                    {
                        if !captures.is_empty() {
                            let mut lifted_params =
                                captures.iter().map(|(_, t)| *t).collect::<Vec<_>>();
                            lifted_params.extend(params.iter().copied());
                            f_ty = self.ctx.function(
                                type_params.clone(),
                                lifted_params,
                                result,
                                effect,
                            );
                        }
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            "function signature must be a function type",
                            f.name.span,
                        ));
                        continue;
                    }
                    let mut nested_bounds = BTreeMap::new();
                    if let TypeKind::Function { type_params, .. } = self.ctx.get(f_ty) {
                        for (p_node, p_id) in f.type_params.iter().zip(type_params.iter()) {
                            self.labels.insert(p_node.name.name.clone(), *p_id);
                            if !p_node.bounds.is_empty() {
                                let mut bounds = Vec::new();
                                for b in &p_node.bounds {
                                    if let Some(info) = self.traits.get(&b.name.name) {
                                        if info.type_params.len() != b.args.len() {
                                            self.diagnostics.push(Diagnostic::error(
                                                format!(
                                                    "trait bound '{}' expects {} type arguments, found {}",
                                                    b.name.name,
                                                    info.type_params.len(),
                                                    b.args.len()
                                                ),
                                                b.name.span,
                                            ).with_id(DiagnosticId::TypeTraitTypeParamsUnsupported));
                                            continue;
                                        }
                                        let arg_tys: Vec<TypeId> = b
                                            .args
                                            .iter()
                                            .map(|arg| {
                                                type_from_expr(self.ctx, &mut self.labels, arg)
                                            })
                                            .collect();
                                        bounds.push(TraitBoundRef {
                                            name: format_trait_ref_name(
                                                &b.name.name,
                                                &arg_tys,
                                                self.ctx,
                                            ),
                                            trait_base_name: b.name.name.clone(),
                                            trait_args: arg_tys,
                                            trait_self_ty: info.self_ty,
                                        });
                                    }
                                }
                                if !bounds.is_empty() {
                                    nested_bounds.insert(*p_id, bounds);
                                }
                            }
                        }
                    }
                    let _ = self.env.update_local_function_binding(
                        self.ctx,
                        &f.name.name,
                        f.name.span,
                        f_ty,
                        captures.clone(),
                    );
                    match check_function(
                        f,
                        f_ty,
                        false,
                        self.target,
                        self.profile,
                        captures.as_slice(),
                        self.ctx,
                        self.env,
                        self.labels,
                        self.string_table,
                        self.enums,
                        self.structs,
                        self.instantiations,
                        nested_bounds,
                        self.traits,
                        self.impls,
                        self.generated_functions,
                        self.import_resolution,
                        self.source_map,
                    ) {
                        Ok(checked) => {
                            self.diagnostics.extend(checked.diagnostics);
                            self.generated_functions.push(checked.function);
                        }
                        Err(mut diags) => self.diagnostics.append(&mut diags),
                    }
                }
                Stmt::StructDef(_) => {}
                Stmt::EnumDef(_) => {}
                Stmt::Wasm(_) => {
                    self.diagnostics.push(Diagnostic::error(
                        "wasm block is only allowed as a function body",
                        block.span,
                    ));
                }
                Stmt::LlvmIr(_) => {
                    self.diagnostics.push(Diagnostic::error(
                        "llvm ir block is only allowed as a function body",
                        block.span,
                    ));
                }
                Stmt::Trait(_) | Stmt::Impl(_) => {}
            }
        }

        // Handle final stack depth. Prefer to be forgiving: if there are
        // extra values on the stack, drop them with a warning rather than
        // failing hard. This keeps `:`-blocks and `if` branch combinations
        // usable while preserving diagnostics for surprising code.
        let final_ty: TypeId;
        let value_ty: Option<TypeId>;
        if stack.len() == base_depth {
            let u = self.ctx.unit();
            final_ty = u;
            value_ty = Some(u);
        } else if stack.len() == base_depth + 1 {
            let t = stack.last().unwrap().ty;
            final_ty = t;
            value_ty = Some(t);
        } else if stack.len() > base_depth + 1 {
            // Too many values: report an error and pop extras for recovery.
            let extras = stack.len() - (base_depth + 1);
            for _ in 0..extras {
                // Pop and ignore the extra value(s).
                stack.pop();
            }
            self.diagnostics.push(Diagnostic::error(
                "block left extra values on the stack",
                block.span,
            ));
            if stack.len() == base_depth {
                let u = self.ctx.unit();
                final_ty = u;
                value_ty = Some(u);
            } else {
                let t = stack.last().unwrap().ty;
                final_ty = t;
                value_ty = Some(t);
            }
        } else {
            // Fewer than expected: this is a hard error.
            self.diagnostics.push(Diagnostic::error(
                "block leaves inconsistent stack state",
                block.span,
            ));
            final_ty = self.ctx.unit();
            value_ty = None;
        };

        if dump_enabled() {
            dump!(
                "NEPL_DUMP_HIR: block span={:?} lines={} final_ty={:?} value_ty={:?}",
                block.span,
                lines.len(),
                final_ty,
                value_ty
            );
            // Print env scopes and a compact preview of the HIR lines for diagnosis
            dump!("NEPL_DUMP_HIR: env scopes=\n{:?}", self.env.scopes);
            for (i, l) in lines.iter().enumerate() {
                dump!(
                    "NEPL_DUMP_HIR: line {} -> expr.kind = {:?}, ty={:?}, drop={}",
                    i,
                    l.expr.kind,
                    l.expr.ty,
                    l.drop_result
                );
            }
        }

        if new_scope {
            self.env.pop_scope();
        }

        self.current_effect = old_effect;

        Some((
            HirBlock {
                lines,
                ty: final_ty,
                span: block.span,
            },
            value_ty,
        ))
    }
}

// ---------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum AssignKind {
    Let,
    Set,
    AddrOf(bool),
    Deref,
}

#[derive(Debug, Clone)]
struct StackEntry {
    ty: TypeId,
    expr: HirExpr,
    type_args: Vec<TypeId>,
    assign: Option<AssignKind>,
    auto_call: bool,
}
