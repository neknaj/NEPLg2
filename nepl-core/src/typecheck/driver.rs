use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::*;
use crate::compiler::{BuildProfile, CompileTarget};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::{ResolveDiagnosticCode, TypeDiagnosticCode};
use crate::hir::*;
use crate::resolve::{DefId, ImportResolution};
use crate::source_map::{CompilerMemoryType, SourceMap};
use crate::span::Span;
use crate::types::{
    EnumVariantInfo, NominalStableTypeIdentity, NominalStableTypeKind, TypeCtx, TypeId, TypeKind,
};

use super::binding_rules::{
    detect_field_accessor_fn, find_invalid_same_file_overload, find_same_signature_func_in_file,
    find_visible_nonshadow_same_signature_func, find_visible_same_signature_func,
    is_callable_binding, shadow_blocked_by_nonshadow,
};
use super::check_function;
use super::compiler_memory_type::compiler_memory_type_definition_allowed;
use super::copy_capability::{
    mark_owner_backed_aggregate_constructor_policies, target_is_compiler_owner_token,
};
use super::diagnostics::{resolve_error, resolve_warning, type_error};
use super::driver_entry::resolve_entry_function;
use super::driver_span::{span_key, top_level_definition_span};
use super::env::{Binding, BindingKind, Env};
use super::extern_import::ExternImportModule;
use super::model::{EnumInfo, RestrictedStructConstructor, StructConstructorPolicy, StructInfo};
use super::public_signature::{build_typed_public_signature_table, TypedPublicSignatureTable};
use super::public_surface::{build_typed_public_surface_table, TypedPublicSurfaceTable};
use super::signature::{
    contains_same_type, function_signature_string, mangle_function_symbol,
    mangle_function_symbol_for_def, mangle_impl_method, push_unique_type, same_function_signature,
    type_contains_unbound_var,
};
use super::struct_shape::StructConstructorShape;
use super::syntax_helpers::gate_allows;
use super::traits::{
    collect_type_params, insert_substitution_mapping, BoundEnv, ImplInfo, ImplKind,
    TraitApplication, TraitBound, TraitCapability, TraitId, TraitInfo, TraitSemantics, TypeParamId,
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

fn type_patterns_overlap(ctx: &TypeCtx, lhs: TypeId, rhs: TypeId) -> bool {
    ctx.type_pattern_matches(lhs, rhs) || ctx.type_pattern_matches(rhs, lhs)
}

fn nominal_stable_identity_for_definition(
    ctx: &TypeCtx,
    source_map: Option<&SourceMap>,
    span: Span,
    kind: NominalStableTypeKind,
    name: &str,
    arity: usize,
    type_kind: &TypeKind,
) -> Option<NominalStableTypeIdentity> {
    let source_path = source_map?
        .path(span.file_id)
        .map(|path| path.as_str().to_string())?;
    let definition_hash = ctx.nominal_definition_hash(type_kind)?;
    Some(NominalStableTypeIdentity::new(
        kind,
        source_path,
        name.to_string(),
        arity,
        definition_hash,
    ))
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
    pub public_signatures: TypedPublicSignatureTable,
    pub public_surface: TypedPublicSurfaceTable,
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
    let mut pending_drop_copy_checks: Vec<(TypeId, Span)> = Vec::new();
    let mut rejected_impl_spans: BTreeSet<(u32, u32, u32)> = BTreeSet::new();
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
        let key = span_key(sp);
        if !seen_directive_spans.insert(key) {
            return;
        }
        if let Directive::Entry { name } = d {
            entry = Some((name.name.clone(), name.span));
        } else if let Directive::Extern {
            vis,
            module: m,
            name: n,
            func,
            signature,
            span,
        } = d
        {
            if ExternImportModule::from_module_name(m)
                .is_some_and(|module| !module.is_allowed_for_target(target))
            {
                diagnostics.push(type_error(
                    TypeDiagnosticCode::ExternWasiTargetMismatch,
                    "WASI import is only allowed for #target wasi",
                    *span,
                ));
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
                    visibility: *vis,
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
                        type_param_bounds: BoundEnv::new(),
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
                diagnostics.push(type_error(
                    TypeDiagnosticCode::ExternSignatureNotFunction,
                    "extern signature must be a function type",
                    *span,
                ));
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
    let mut seen_declaration_item_spans = BTreeSet::new();
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
        if let Some(span) = top_level_definition_span(item) {
            if !seen_declaration_item_spans.insert(span_key(span)) {
                continue;
            }
        }
        match item {
            Stmt::EnumDef(e) => {
                if enums.contains_key(&e.name.name)
                    || structs.contains_key(&e.name.name)
                    || env.lookup_any_defined(&e.name.name).is_some()
                {
                    diagnostics.push(resolve_error(
                        ResolveDiagnosticCode::ItemNameConflict,
                        "name already used by another item",
                        e.name.span,
                    ));
                    continue;
                }
                for p in &e.type_params {
                    if !p.bounds.is_empty() {
                        diagnostics.push(type_error(
                            TypeDiagnosticCode::EnumTypeParamBoundsUnsupported,
                            "enum type parameter bounds are not supported yet",
                            p.name.span,
                        ));
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
                let enum_kind = TypeKind::Enum {
                    name: e.name.name.clone(),
                    type_params: tps.clone(),
                    variants: vars.clone(),
                };
                let ty = match nominal_stable_identity_for_definition(
                    &ctx,
                    source_map,
                    e.name.span,
                    NominalStableTypeKind::Enum,
                    &e.name.name,
                    tps.len(),
                    &enum_kind,
                ) {
                    Some(identity) => ctx.register_named_with_stable_identity(
                        e.name.name.clone(),
                        enum_kind,
                        identity,
                    ),
                    None => ctx.register_named(e.name.name.clone(), enum_kind),
                };
                label_env.insert(e.name.name.clone(), ty);
                enums.insert(
                    e.name.name.clone(),
                    EnumInfo {
                        ty,
                        visibility: e.vis,
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
                        visibility: e.vis,
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
                            type_param_bounds: BoundEnv::new(),
                            captures: Vec::new(),
                        },
                    });

                    // Qualified name (e.g. "Option::Some")
                    env.insert_global(Binding {
                        name: format!("{}::{}", e.name.name, v.name),
                        ty: func_ty,
                        visibility: e.vis,
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
                            type_param_bounds: BoundEnv::new(),
                            captures: Vec::new(),
                        },
                    });
                }
            }
            Stmt::StructDef(s) => {
                if structs.contains_key(&s.name.name)
                    || enums.contains_key(&s.name.name)
                    || env.lookup_any_defined(&s.name.name).is_some()
                {
                    diagnostics.push(resolve_error(
                        ResolveDiagnosticCode::ItemNameConflict,
                        "name already used by another item",
                        s.name.span,
                    ));
                    continue;
                }
                for p in &s.type_params {
                    if !p.bounds.is_empty() {
                        diagnostics.push(type_error(
                            TypeDiagnosticCode::StructTypeParamBoundsUnsupported,
                            "struct type parameter bounds are not supported yet",
                            p.name.span,
                        ));
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
                let struct_kind = TypeKind::Struct {
                    name: s.name.name.clone(),
                    type_params: tps.clone(),
                    fields: fs.clone(),
                    field_names: f_names.clone(),
                };
                let ty = match nominal_stable_identity_for_definition(
                    &ctx,
                    source_map,
                    s.name.span,
                    NominalStableTypeKind::Struct,
                    &s.name.name,
                    tps.len(),
                    &struct_kind,
                ) {
                    Some(identity) => ctx.register_named_with_stable_identity(
                        s.name.name.clone(),
                        struct_kind,
                        identity,
                    ),
                    None => ctx.register_named(s.name.name.clone(), struct_kind),
                };
                let compiler_memory_type = compiler_memory_type_definition_allowed(
                    s, &fs, &f_names, &tps, &ctx, source_map,
                );
                if let Some(memory_type) = compiler_memory_type {
                    ctx.mark_compiler_memory_type(ty, memory_type);
                }

                let constructor_shape = StructConstructorShape::classify(&ctx, &fs, &f_names);
                let ret_ty = if tps.is_empty() {
                    ty
                } else {
                    ctx.apply(ty, tps.clone())
                };
                let constructor_params = constructor_shape.constructor_params(&fs);
                let constructor_ty =
                    ctx.function(tps.clone(), constructor_params, ret_ty, Effect::Pure);
                env.insert_global(Binding {
                    name: s.name.name.clone(),
                    ty: constructor_ty,
                    visibility: s.vis,
                    mutable: false,
                    no_shadow: false,
                    defined: true,
                    span: s.name.span,
                    kind: BindingKind::Func {
                        def_id: DefId::from_span(s.name.span),
                        symbol: s.name.name.clone(),
                        effect: Effect::Pure,
                        arity: constructor_shape.constructor_arity(fs.len()),
                        builtin: None,
                        field_accessor: None,
                        type_param_bounds: BoundEnv::new(),
                        captures: Vec::new(),
                    },
                });

                label_env.insert(s.name.name.clone(), ty);
                structs.insert(
                    s.name.name.clone(),
                    StructInfo {
                        ty,
                        visibility: s.vis,
                        type_params: tps,
                        fields: fs,
                        field_names: f_names,
                        constructor_shape,
                        constructor_policy: struct_constructor_policy(compiler_memory_type),
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
                            diagnostics.push(type_error(
                                TypeDiagnosticCode::TraitCapabilityUnknown,
                                format!("unknown trait capability '{}'", name.trim()),
                                t.name.span,
                            ));
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
                        diagnostics.push(type_error(
                            TypeDiagnosticCode::TraitMethodTypeParamsUnsupported,
                            "trait methods cannot have type parameters yet",
                            m.name.span,
                        ));
                        continue;
                    }
                    let sig = type_from_expr(&mut ctx, &mut f_labels, &m.signature);
                    methods.insert(m.name.name.clone(), sig);
                }
                if capabilities.contains(&TraitCapability::Drop) && !methods.contains_key("drop") {
                    diagnostics.push(type_error(
                        TypeDiagnosticCode::TraitDropMethodMissing,
                        "drop capability trait must define a method named 'drop'",
                        t.name.span,
                    ));
                }
                traits.insert(
                    t.name.name.clone(),
                    TraitInfo {
                        doc: t.doc.clone(),
                        visibility: t.vis,
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

    mark_owner_backed_aggregate_constructor_policies(&ctx, &mut structs);

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
                    visibility: info.visibility,
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
                        type_param_bounds: BoundEnv::new(),
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
    let mut seen_impl_collection_spans = BTreeSet::new();
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
        if let Some(span) = top_level_definition_span(item) {
            if !seen_impl_collection_spans.insert(span_key(span)) {
                continue;
            }
        }
        if let Stmt::Impl(i) = item {
            let trait_ref = match i.trait_ref.as_ref() {
                Some(trait_ref) => trait_ref,
                None => {
                    let impl_kind = ImplKind::Inherent;
                    match impl_kind {
                        ImplKind::Inherent => {
                            diagnostics.push(type_error(
                                TypeDiagnosticCode::ImplInherentUnsupported,
                                "inherent impl is not supported yet",
                                i.span,
                            ));
                            rejected_impl_spans.insert(span_key(i.span));
                            continue;
                        }
                        ImplKind::Trait { .. } => continue,
                    }
                }
            };
            let trait_name = trait_ref.name.name.clone();
            let trait_info = match traits.get(&trait_name) {
                Some(info) => info,
                None => {
                    diagnostics.push(type_error(
                        TypeDiagnosticCode::TraitUnknown,
                        format!("unknown trait '{}'", trait_name),
                        i.span,
                    ));
                    rejected_impl_spans.insert(span_key(i.span));
                    continue;
                }
            };
            let trait_self_ty = trait_info.self_ty;
            let mut f_labels = LabelEnv::new();
            let (impl_type_params, _bounds_vec, impl_bounds_map) = collect_type_params(
                &mut ctx,
                &mut f_labels,
                &i.type_params,
                &traits,
                &mut diagnostics,
            );
            let target_ty = type_from_expr(&mut ctx, &mut f_labels, &i.target_ty);
            if trait_info.type_params.len() != trait_ref.args.len() {
                diagnostics.push(type_error(
                    TypeDiagnosticCode::TraitTypeParamsUnsupported,
                    format!(
                        "trait '{}' expects {} type arguments, found {}",
                        trait_ref.name.name,
                        trait_info.type_params.len(),
                        trait_ref.args.len()
                    ),
                    trait_ref.name.span,
                ));
                rejected_impl_spans.insert(span_key(i.span));
                continue;
            }
            let trait_args: Vec<TypeId> = trait_ref
                .args
                .iter()
                .map(|arg| type_from_expr(&mut ctx, &mut f_labels, arg))
                .collect();
            let trait_application = TraitApplication {
                trait_id: TraitId::from_name(&trait_name),
                args: trait_args,
            };
            f_labels.insert(String::from("Self"), target_ty);
            let generic_impl_target = type_contains_unbound_var(&ctx, target_ty);
            if generic_impl_target
                && !trait_semantics.has_copy_capability(Some(trait_self_ty))
                && !trait_semantics.has_clone_capability(Some(trait_self_ty))
                && !trait_semantics.has_drop_capability(Some(trait_self_ty))
            {
                diagnostics.push(type_error(
                    TypeDiagnosticCode::ImplTargetNotConcrete,
                    "impl target type must be concrete",
                    i.target_ty.span(),
                ));
                rejected_impl_spans.insert(span_key(i.span));
                continue;
            }
            if trait_semantics.has_copy_capability(Some(trait_self_ty)) {
                if target_is_compiler_owner_token(&ctx, target_ty)
                    || !ctx.is_copy_impl_eligible(target_ty)
                {
                    diagnostics.push(type_error(
                        TypeDiagnosticCode::CopyImplTargetNotCopy,
                        "copy impl target type is not copyable",
                        i.target_ty.span(),
                    ));
                    push_unique_type(&ctx, &mut rejected_copy_targets, target_ty);
                    rejected_impl_spans.insert(span_key(i.span));
                    continue;
                }
                pending_copy_clone_checks.push((target_ty, i.span));
            }
            if trait_semantics.has_drop_capability(Some(trait_self_ty)) {
                pending_drop_copy_checks.push((target_ty, i.span));
            }
            if impls.iter().any(|imp| {
                imp.matches_same_trait_impl(&ctx, &trait_application, trait_self_ty)
                    && (ctx.type_pattern_matches(imp.target_ty, target_ty)
                        || ctx.type_pattern_matches(target_ty, imp.target_ty))
            }) {
                diagnostics.push(type_error(
                    TypeDiagnosticCode::ImplDuplicateForTraitTarget,
                    "duplicate impl for same trait and target type",
                    i.span,
                ));
                rejected_impl_spans.insert(span_key(i.span));
                continue;
            }

            impls.push(ImplInfo {
                type_params: impl_type_params,
                type_param_bounds: impl_bounds_map,
                kind: ImplKind::Trait {
                    application: trait_application,
                    self_ty: trait_self_ty,
                },
                target_ty,
            });
        }
    }
    for (target_ty, span) in pending_copy_clone_checks {
        let has_clone_impl = impls.iter().any(|imp| {
            trait_semantics.has_clone_capability(imp.trait_self_ty())
                && (ctx.type_pattern_matches(imp.target_ty, target_ty)
                    || ctx.type_pattern_matches(target_ty, imp.target_ty))
        });
        if !has_clone_impl {
            diagnostics.push(type_error(
                TypeDiagnosticCode::CopyImplRequiresClone,
                "copy impl requires clone impl for the same target type",
                span,
            ));
            push_unique_type(&ctx, &mut rejected_copy_targets, target_ty);
            rejected_impl_spans.insert(span_key(span));
        }
    }
    impls.retain(|imp| {
        if !trait_semantics.has_copy_capability(imp.trait_self_ty()) {
            return true;
        }
        !contains_same_type(&ctx, &rejected_copy_targets, imp.target_ty)
    });
    let mut rejected_drop_targets: Vec<TypeId> = Vec::new();
    for (target_ty, span) in pending_drop_copy_checks {
        let overlaps_copy_impl = impls.iter().any(|imp| {
            trait_semantics.has_copy_capability(imp.trait_self_ty())
                && type_patterns_overlap(&ctx, imp.target_ty, target_ty)
        });
        if ctx.is_copy(target_ty) || overlaps_copy_impl {
            diagnostics.push(type_error(
                TypeDiagnosticCode::DropImplTargetCopy,
                "drop impl target type is copyable",
                span,
            ));
            push_unique_type(&ctx, &mut rejected_drop_targets, target_ty);
            rejected_impl_spans.insert(span_key(span));
        }
    }
    impls.retain(|imp| {
        if !trait_semantics.has_drop_capability(imp.trait_self_ty()) {
            return true;
        }
        !contains_same_type(&ctx, &rejected_drop_targets, imp.target_ty)
    });
    for imp in impls.iter() {
        if trait_semantics.has_clone_capability(imp.trait_self_ty()) {
            ctx.register_clone_impl_target(imp.target_ty);
        }
        if trait_semantics.has_copy_capability(imp.trait_self_ty()) {
            ctx.register_copy_impl_target(imp.target_ty);
        }
        if trait_semantics.has_drop_capability(imp.trait_self_ty()) {
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
                visibility: info.visibility,
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
                    type_param_bounds: BoundEnv::new(),
                    captures: Vec::new(),
                },
            });
        }
    }

    let mut pending_if: Option<bool> = None;
    let mut seen_callable_item_spans = BTreeSet::new();
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
        if let Some(span) = top_level_definition_span(item) {
            if !seen_callable_item_spans.insert(span_key(span)) {
                continue;
            }
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
                    diagnostics.push(resolve_error(
                        ResolveDiagnosticCode::ItemNameConflict,
                        "name already used by another item",
                        f.name.span,
                    ));
                    continue;
                }
                if enums.contains_key(&f.name.name) || structs.contains_key(&f.name.name) {
                    diagnostics.push(resolve_error(
                        ResolveDiagnosticCode::ItemNameConflict,
                        "name already used by another item",
                        f.name.span,
                    ));
                    continue;
                }
                if crate::log::is_verbose() {
                    driver_log!("typecheck: registering global func {}", f.name.name);
                }
                if let Some(prev) = find_same_signature_func_in_file(
                    &env,
                    &f.name.name,
                    ty,
                    &bounds_map,
                    f.name.span,
                    &ctx,
                ) {
                    diagnostics.push(
                        resolve_warning(
                            ResolveDiagnosticCode::ShadowSameSignatureCallable,
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
                        type_error(
                            TypeDiagnosticCode::OverloadAmbiguous,
                            "ambiguous overload",
                            f.name.span,
                        )
                        .with_secondary_label(
                            prev.span,
                            Some("conflicting overload is defined here".into()),
                        ),
                    );
                    continue;
                }
                if let Some(blocked) = shadow_blocked_by_nonshadow(&env, &f.name.name) {
                    if is_callable_binding(blocked) {
                        if let Some(conflict) = find_visible_nonshadow_same_signature_func(
                            &env,
                            &import_resolution,
                            &f.name.name,
                            ty,
                            &bounds_map,
                            f.name.span,
                            &ctx,
                        ) {
                            diagnostics.push(
                                resolve_error(
                                    ResolveDiagnosticCode::ShadowNoShadowViolation,
                                    format!(
                                        "cannot shadow non-shadowable function '{}' with same signature",
                                        f.name.name
                                    ),
                                    f.name.span,
                                ),
                            );
                            diagnostics.push(
                                resolve_error(
                                    ResolveDiagnosticCode::ShadowNoShadowViolation,
                                    "non-shadowable function declaration is here",
                                    conflict.span,
                                )
                                .with_secondary_label(f.name.span, Some("shadow attempt".into())),
                            );
                            continue;
                        }
                        // 関数同名はオーバーロードとして扱う（異なるシグネチャは許可）。
                    } else {
                        diagnostics.push(resolve_error(
                            ResolveDiagnosticCode::ShadowNoShadowViolation,
                            format!("cannot shadow non-shadowable symbol '{}'", f.name.name),
                            f.name.span,
                        ));
                        diagnostics.push(
                            resolve_error(
                                ResolveDiagnosticCode::ShadowNoShadowViolation,
                                "non-shadowable declaration is here",
                                blocked.span,
                            )
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
                        || find_visible_same_signature_func(
                            &env,
                            &import_resolution,
                            &f.name.name,
                            ty,
                            &bounds_map,
                            f.name.span,
                            &ctx,
                        )
                        .is_some())
                {
                    diagnostics.push(resolve_error(
                        ResolveDiagnosticCode::ShadowNoShadowConflict,
                        format!(
                            "noshadow declaration '{}' conflicts with existing symbol",
                            f.name.name
                        ),
                        f.name.span,
                    ));
                    continue;
                }
                env.remove_duplicate_func_in_file(&f.name.name, ty, &bounds_map, f.name.span, &ctx);
                let has_cross_file_duplicate =
                    env.qualify_same_signature_func_symbols(&f.name.name, ty, &ctx);
                let symbol = if has_cross_file_duplicate {
                    mangle_function_symbol_for_def(&f.name.name, ty, &ctx, f.name.span)
                } else {
                    mangle_function_symbol(&f.name.name, ty, &ctx)
                };
                if crate::log::is_verbose() {
                    driver_log!(
                        "typecheck: registering global func {} sig={}",
                        f.name.name,
                        function_signature_string(&ctx, ty)
                    );
                }
                env.insert_global(Binding {
                    name: f.name.name.clone(),
                    ty,
                    visibility: f.vis,
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
                diagnostics.push(type_error(
                    TypeDiagnosticCode::FunctionSignatureNotFunction,
                    "function signature must be a function type",
                    f.name.span,
                ));
            }
        }
    }

    for alias in &fn_aliases {
        if enums.contains_key(&alias.name.name) || structs.contains_key(&alias.name.name) {
            diagnostics.push(resolve_error(
                ResolveDiagnosticCode::ItemNameConflict,
                "name already used by another item",
                alias.name.span,
            ));
            continue;
        }
        let targets = env.lookup_all_callables(&alias.target.name);
        if targets.is_empty() {
            diagnostics.push(resolve_error(
                ResolveDiagnosticCode::AliasTargetNotFound,
                "alias target not found",
                alias.target.span,
            ));
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
            if let Some(prev) = find_same_signature_func_in_file(
                &env,
                &alias.name.name,
                ty,
                &bounds,
                alias.name.span,
                &ctx,
            ) {
                diagnostics.push(
                    resolve_warning(
                        ResolveDiagnosticCode::ShadowSameSignatureCallable,
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
                diagnostics.push(resolve_error(
                    ResolveDiagnosticCode::ItemNameConflict,
                    "name already used by another item",
                    alias.name.span,
                ));
                break;
            }
            if let Some(blocked) = shadow_blocked_by_nonshadow(&env, &alias.name.name) {
                if is_callable_binding(blocked) {
                    if let Some(conflict) = find_visible_nonshadow_same_signature_func(
                        &env,
                        &import_resolution,
                        &alias.name.name,
                        ty,
                        &bounds,
                        alias.name.span,
                        &ctx,
                    ) {
                        diagnostics.push(
                            resolve_error(
                                ResolveDiagnosticCode::ShadowNoShadowViolation,
                                format!(
                                    "cannot shadow non-shadowable function alias '{}' with same signature",
                                    alias.name.name
                                ),
                                alias.name.span,
                            ),
                        );
                        diagnostics.push(
                            resolve_error(
                                ResolveDiagnosticCode::ShadowNoShadowViolation,
                                "non-shadowable function declaration is here",
                                conflict.span,
                            )
                            .with_secondary_label(alias.name.span, Some("shadow attempt".into())),
                        );
                        break;
                    }
                    // 関数同名はオーバーロードとして扱う（異なるシグネチャは許可）。
                } else {
                    diagnostics.push(resolve_error(
                        ResolveDiagnosticCode::ShadowNoShadowViolation,
                        format!("cannot shadow non-shadowable symbol '{}'", alias.name.name),
                        alias.name.span,
                    ));
                    diagnostics.push(
                        resolve_error(
                            ResolveDiagnosticCode::ShadowNoShadowViolation,
                            "non-shadowable declaration is here",
                            blocked.span,
                        )
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
                    || find_visible_same_signature_func(
                        &env,
                        &import_resolution,
                        &alias.name.name,
                        ty,
                        &bounds,
                        alias.name.span,
                        &ctx,
                    )
                    .is_some())
            {
                diagnostics.push(resolve_error(
                    ResolveDiagnosticCode::ShadowNoShadowConflict,
                    format!(
                        "noshadow declaration '{}' conflicts with existing symbol",
                        alias.name.name
                    ),
                    alias.name.span,
                ));
                break;
            }
            env.remove_duplicate_func_in_file(&alias.name.name, ty, &bounds, alias.name.span, &ctx);
            env.insert_global(Binding {
                name: alias.name.name.clone(),
                ty,
                visibility: alias.vis,
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
    let mut seen_function_body_spans = BTreeSet::new();
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
        if let Some(span) = top_level_definition_span(item) {
            if !seen_function_body_spans.insert(span_key(span)) {
                continue;
            }
        }
        if let Stmt::FnDef(f) = item {
            let f_ty = {
                if let Some(binding) = env.lookup_callable_defined_at(&f.name.name, f.name.span) {
                    binding.ty
                } else {
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
                                diagnostics.push(type_error(
                                    TypeDiagnosticCode::FunctionSignatureOverloadNotFound,
                                    "function signature does not match any overload",
                                    f.name.span,
                                ));
                                continue;
                            }
                        }
                    }
                }
            };
            let mut type_param_bounds = BoundEnv::new();
            if let TypeKind::Function { type_params, .. } = ctx.get(f_ty) {
                for (p_node, p_id) in f.type_params.iter().zip(type_params.iter()) {
                    label_env.insert(p_node.name.name.clone(), *p_id);
                    if !p_node.bounds.is_empty() {
                        let mut bounds = Vec::new();
                        for b in &p_node.bounds {
                            if let Some(info) = traits.get(&b.name.name) {
                                if info.type_params.len() != b.args.len() {
                                    diagnostics.push(type_error(
                                        TypeDiagnosticCode::TraitTypeParamsUnsupported,
                                        format!(
                                            "trait bound '{}' expects {} type arguments, found {}",
                                            b.name.name,
                                            info.type_params.len(),
                                            b.args.len()
                                        ),
                                        b.name.span,
                                    ));
                                    continue;
                                }
                                let arg_tys: Vec<TypeId> = b
                                    .args
                                    .iter()
                                    .map(|arg| type_from_expr(&mut ctx, &mut label_env, arg))
                                    .collect();
                                bounds.push(TraitBound {
                                    application: TraitApplication {
                                        trait_id: TraitId::from_name(&b.name.name),
                                        args: arg_tys,
                                    },
                                    trait_self_ty: info.self_ty,
                                });
                            }
                        }
                        if !bounds.is_empty() {
                            type_param_bounds.insert(TypeParamId::new(*p_id), bounds);
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
    let mut seen_final_impl_spans = BTreeSet::new();
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
        if let Some(span) = top_level_definition_span(item) {
            if !seen_final_impl_spans.insert(span_key(span)) {
                continue;
            }
        }
        if let Stmt::Impl(i) = item {
            let impl_key = span_key(i.span);
            if rejected_impl_spans.contains(&impl_key) {
                continue;
            }
            let trait_ref = match &i.trait_ref {
                Some(tr) => tr,
                None => {
                    diagnostics.push(type_error(
                        TypeDiagnosticCode::ImplInherentUnsupported,
                        "inherent impl is not supported yet",
                        i.span,
                    ));
                    continue;
                }
            };
            let trait_name = trait_ref.name.name.clone();
            let trait_info = match traits.get(&trait_name) {
                Some(info) => info,
                None => {
                    diagnostics.push(type_error(
                        TypeDiagnosticCode::TraitUnknown,
                        format!("unknown trait '{}'", trait_name),
                        i.span,
                    ));
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
                diagnostics.push(type_error(
                    TypeDiagnosticCode::TraitTypeParamsUnsupported,
                    format!(
                        "trait '{}' expects {} type arguments, found {}",
                        trait_name,
                        trait_info.type_params.len(),
                        trait_ref.args.len()
                    ),
                    trait_ref.name.span,
                ));
                continue;
            }
            let trait_args: Vec<TypeId> = trait_ref
                .args
                .iter()
                .map(|arg| type_from_expr(&mut ctx, &mut f_labels, arg))
                .collect();
            let trait_application = TraitApplication {
                trait_id: TraitId::from_name(&trait_name),
                args: trait_args,
            };
            if type_contains_unbound_var(&ctx, target_ty)
                && !trait_semantics.has_copy_capability(Some(trait_info.self_ty))
                && !trait_semantics.has_clone_capability(Some(trait_info.self_ty))
                && !trait_semantics.has_drop_capability(Some(trait_info.self_ty))
            {
                diagnostics.push(type_error(
                    TypeDiagnosticCode::ImplTargetNotConcrete,
                    "impl target type must be concrete",
                    i.target_ty.span(),
                ));
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
                    diagnostics.push(type_error(
                        TypeDiagnosticCode::ImplDuplicateMethod,
                        "duplicate method in impl",
                        m.name.span,
                    ));
                    continue;
                }
                if !m.type_params.is_empty() {
                    diagnostics.push(type_error(
                        TypeDiagnosticCode::TraitMethodTypeParamsUnsupported,
                        "impl methods cannot have type parameters yet",
                        m.name.span,
                    ));
                    continue;
                }
                let trait_sig = match trait_info.methods.get(&m.name.name) {
                    Some(sig) => *sig,
                    None => {
                        diagnostics.push(type_error(
                            TypeDiagnosticCode::ImplMethodNotInTrait,
                            format!(
                                "method '{}' not found in trait '{}'",
                                m.name.name, trait_name
                            ),
                            m.name.span,
                        ));
                        continue;
                    }
                };
                let mut mapping = BTreeMap::new();
                mapping.insert(
                    ctx.resolve_id(trait_info.self_ty),
                    ctx.resolve_id(target_ty),
                );
                for (tp, arg) in trait_info
                    .type_params
                    .iter()
                    .zip(trait_application.args.iter())
                {
                    insert_substitution_mapping(&ctx, &mut mapping, *tp, *arg);
                }
                let expected_sig = ctx.substitute(trait_sig, &mapping);
                let actual_sig = type_from_expr(&mut ctx, &mut f_labels, &m.signature);
                if !ctx.same_type(expected_sig, actual_sig) {
                    diagnostics.push(type_error(
                        TypeDiagnosticCode::ImplMethodSignatureMismatch,
                        "impl method signature does not match trait",
                        m.name.span,
                    ));
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
                let trait_display_name = trait_application.display_name(&ctx);
                let mangled =
                    mangle_impl_method(&trait_display_name, &m.name.name, target_ty, &ctx);
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
                    diagnostics.push(type_error(
                        TypeDiagnosticCode::ImplMissingTraitMethod,
                        format!(
                            "missing method '{}' for trait '{}'",
                            trait_method, trait_name
                        ),
                        i.span,
                    ));
                }
            }

            if let Some(prev) = prev_self {
                label_env.insert(String::from("Self"), prev);
            } else {
                label_env.remove("Self");
            }

            final_impls.push(HirImpl {
                doc: i.doc.clone(),
                trait_application: HirTraitApplication::new(
                    String::from(trait_application.trait_id.as_str()),
                    trait_application.args.clone(),
                ),
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

    let public_signatures = if has_error {
        TypedPublicSignatureTable::default()
    } else {
        build_typed_public_signature_table(&ctx, &env, &structs, &enums, &traits, &impls)
    };
    let public_surface = if has_error {
        TypedPublicSurfaceTable::default()
    } else {
        build_typed_public_surface_table(&ctx, source_map, &env, &structs, &enums, &traits, &impls)
    };

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
        public_signatures,
        public_surface,
    }
}

fn struct_constructor_policy(
    compiler_memory_type: Option<CompilerMemoryType>,
) -> StructConstructorPolicy {
    let Some(memory_type) = compiler_memory_type else {
        return StructConstructorPolicy::Public;
    };
    match memory_type {
        CompilerMemoryType::RawPointer => {
            StructConstructorPolicy::RawMemoryBoundaryOnly(RestrictedStructConstructor::RawPointer)
        }
        CompilerMemoryType::OwnerToken => {
            StructConstructorPolicy::RawMemoryBoundaryOnly(RestrictedStructConstructor::OwnerToken)
        }
    }
}
