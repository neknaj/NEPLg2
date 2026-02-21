#![no_std]
extern crate alloc;
extern crate std;

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::*;
use crate::builtins::BuiltinKind;
use crate::compiler::{BuildProfile, CompileTarget};
use crate::diagnostic::Diagnostic;
use crate::hir::*;
use crate::span::Span;
use crate::types::{EnumVariantInfo, TypeCtx, TypeId, TypeKind};

// Helper to gate verbose HIR dumps. Use `dump!(...)` for noisy debug output
// that should only appear when `NEPL_DUMP_HIR` is set.
fn dump_enabled() -> bool {
    std::env::var("NEPL_DUMP_HIR").is_ok()
}

macro_rules! dump {
    ($($arg:tt)*) => {
        if dump_enabled() {
            std::eprintln!($($arg)*);
        }
    };
}

fn print_diagnostics_summary(diags: &alloc::vec::Vec<crate::diagnostic::Diagnostic>) {
    if diags.is_empty() {
        return;
    }
    // Print a short, readable summary of diagnostics (one line per diagnostic)
    std::eprintln!("Compiler diagnostics:");
    for d in diags.iter() {
        let sev = match d.severity {
            crate::diagnostic::Severity::Error => "error",
            crate::diagnostic::Severity::Warning => "warning",
        };
        // Display primary span as file_id:start-end for quick location.
        let span = &d.primary.span;
        std::eprintln!("- {}: {} (span: {:?}:{:?}-{:?})", sev, d.message, span.file_id, span.start, span.end);
        for sec in d.secondary.iter() {
            std::eprintln!("  note: {:?}:{:?}-{:?} {}", sec.span.file_id, sec.span.start, sec.span.end, sec.message.as_ref().unwrap_or(&alloc::string::String::new()));
        }
    }
}
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
struct TraitInfo {
    name: String,
    type_params: Vec<TypeId>,
    methods: BTreeMap<String, TypeId>,
    self_ty: TypeId,
    span: Span,
}

#[derive(Debug, Clone)]
struct ImplInfo {
    trait_name: Option<String>,
    target_ty: TypeId,
    methods: BTreeMap<String, (String, TypeId)>, // name -> (mangled_name, type)
}

#[derive(Debug, Clone)]
enum FieldIdx {
    Index(usize),
    Name(String),
}

fn collect_type_params(
    ctx: &mut TypeCtx,
    labels: &mut LabelEnv,
    params: &[TypeParam],
    traits: &BTreeMap<String, TraitInfo>,
    diags: &mut Vec<Diagnostic>,
) -> (Vec<TypeId>, Vec<Vec<String>>, BTreeMap<TypeId, Vec<String>>) {
    let mut tps = Vec::new();
    let mut bounds_vec = Vec::new();
    let mut bounds_map = BTreeMap::new();
    for p in params {
        let id = ctx.fresh_var(Some(p.name.name.clone()));
        labels.insert(p.name.name.clone(), id);
        let mut bounds = Vec::new();
        for b in &p.bounds {
            if !traits.contains_key(b) {
                diags.push(Diagnostic::error(
                    format!("unknown trait bound '{}'", b),
                    p.name.span,
                ));
            } else {
                bounds.push(b.clone());
            }
        }
        if !bounds.is_empty() {
            bounds_map.insert(id, bounds.clone());
        }
        bounds_vec.push(bounds);
        tps.push(id);
    }
    (tps, bounds_vec, bounds_map)
}

pub fn typecheck(
    module: &crate::ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> TypeCheckResult {
    let mut ctx = TypeCtx::new();
    let mut label_env = LabelEnv::new();
    let mut env = Env::new();
    let mut diagnostics = Vec::new();
    let mut strings = StringTable::new();
    let mut enums: BTreeMap<String, EnumInfo> = BTreeMap::new();
    let mut structs: BTreeMap<String, StructInfo> = BTreeMap::new();
    let mut traits: BTreeMap<String, TraitInfo> = BTreeMap::new();
    let mut impls: Vec<ImplInfo> = Vec::new();

    let mut entry = None;
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
            entry = Some(name.name.clone());
        } else if let Directive::Extern {
            module: m,
            name: n,
            func,
            signature,
            span,
        } = d
        {
            if matches!(target, CompileTarget::Wasm) && m == "wasi_snapshot_preview1" {
                diagnostics.push(Diagnostic::error(
                    "WASI import not allowed for wasm target (use #target wasi)",
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
                    mutable: false,
                    no_shadow: false,
                    defined: true,
                    moved: false,
                    span: *span,
                    kind: BindingKind::Func {
                        symbol: func.name.clone(),
                        effect,
                        arity: params.len(),
                        builtin: None,
                        type_param_bounds: Vec::new(),
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
                diagnostics.push(Diagnostic::error(
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
                if enums.contains_key(&e.name.name) || env.lookup(&e.name.name).is_some() {
                    continue;
                }
                if env.lookup(&e.name.name).is_some() || structs.contains_key(&e.name.name) {
                    diagnostics.push(Diagnostic::error(
                        "name already used by another item",
                        e.name.span,
                    ));
                    continue;
                }
                for p in &e.type_params {
                    if !p.bounds.is_empty() {
                        diagnostics.push(Diagnostic::error(
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
                let ty = ctx.register_named(
                    e.name.name.clone(),
                    TypeKind::Enum {
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
                for (i, v) in vars.iter().enumerate() {
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
                        moved: false,
                        span: e.name.span,
                        kind: BindingKind::Func {
                            symbol: v.name.clone(),
                            effect: Effect::Pure,
                            arity: params.len(),
                            builtin: None,
                            type_param_bounds: Vec::new(),
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
                        moved: false,
                        span: e.name.span,
                        kind: BindingKind::Func {
                            symbol: format!("{}::{}", e.name.name, v.name),
                            effect: Effect::Pure,
                            arity: params.len(),
                            builtin: None,
                            type_param_bounds: Vec::new(),
                            captures: Vec::new(),
                        },
                    });
                }
            }
            Stmt::StructDef(s) => {
                if structs.contains_key(&s.name.name) || env.lookup(&s.name.name).is_some() {
                    continue;
                }
                if env.lookup(&s.name.name).is_some() || enums.contains_key(&s.name.name) {
                    diagnostics.push(Diagnostic::error(
                        "name already used by another item",
                        s.name.span,
                    ));
                    continue;
                }
                for p in &s.type_params {
                    if !p.bounds.is_empty() {
                        diagnostics.push(Diagnostic::error(
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
                let ty = ctx.register_named(
                    s.name.name.clone(),
                    TypeKind::Struct {
                        name: s.name.name.clone(),
                        type_params: tps.clone(),
                        fields: fs.clone(),
                        field_names: f_names.clone(),
                    },
                );
                
                // Register constructor in environment
                let ret_ty = if tps.is_empty() {
                    ty
                } else {
                    ctx.apply(ty, tps.clone())
                };
                let constructor_ty = ctx.function(tps.clone(), fs.clone(), ret_ty, Effect::Pure);
                env.insert_global(Binding {
                    name: s.name.name.clone(),
                    ty: constructor_ty,
                    mutable: false,
                    no_shadow: false,
                    defined: true,
                    moved: false,
                    span: s.name.span,
                    kind: BindingKind::Func {
                        symbol: s.name.name.clone(),
                        effect: Effect::Pure,
                        arity: fs.len(),
                        builtin: None,
                        type_param_bounds: Vec::new(),
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
                let (tps, _bounds_vec, _bounds_map) =
                    collect_type_params(&mut ctx, &mut f_labels, &t.type_params, &traits, &mut diagnostics);
                if !t.type_params.is_empty() {
                    diagnostics.push(Diagnostic::error(
                        "trait type parameters are not supported yet",
                        t.name.span,
                    ));
                }
                let self_ty = ctx.fresh_var(Some(String::from("Self")));
                f_labels.insert(String::from("Self"), self_ty);
                let mut methods = BTreeMap::new();
                for m in &t.methods {
                    if !m.type_params.is_empty() {
                        diagnostics.push(Diagnostic::error(
                            "trait methods cannot have type parameters yet",
                            m.name.span,
                        ));
                        continue;
                    }
                    let sig = type_from_expr(&mut ctx, &mut f_labels, &m.signature);
                    methods.insert(m.name.name.clone(), sig);
                }
                traits.insert(
                    t.name.name.clone(),
                    TraitInfo {
                        name: t.name.name.clone(),
                        type_params: tps,
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
            if env.lookup(&vname).is_none() {
                env.insert_global(Binding {
                    name: vname.clone(),
                    ty: func_ty,
                    mutable: false,
                    no_shadow: false,
                    defined: true,
                    moved: false,
                    span: Span::dummy(),
                    kind: BindingKind::Func {
                        symbol: vname.clone(),
                        effect: Effect::Pure,
                        arity: params.len(),
                        builtin: None,
                        type_param_bounds: Vec::new(),
                        captures: Vec::new(),
                    },
                });
            }
        }
    }

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
            if i.trait_name.is_none() {
                diagnostics.push(Diagnostic::error(
                    "inherent impl is not supported yet",
                    i.span,
                ));
                continue;
            }
            if !i.type_params.is_empty() {
                diagnostics.push(Diagnostic::error(
                    "impl type parameters are not supported yet",
                    i.span,
                ));
                continue;
            }
            let mut f_labels = LabelEnv::new();
            let (tps, _bounds_vec, _bounds_map) =
                collect_type_params(&mut ctx, &mut f_labels, &i.type_params, &traits, &mut diagnostics);
            let target_ty = type_from_expr(&mut ctx, &mut f_labels, &i.target_ty);
            f_labels.insert(String::from("Self"), target_ty);
            let trait_name = i.trait_name.as_ref().map(|tn| tn.name.clone());
            if let Some(tn) = &trait_name {
                if !traits.contains_key(tn) {
                    diagnostics.push(Diagnostic::error(
                        format!("unknown trait '{}'", tn),
                        i.span,
                    ));
                    continue;
                }
            }
            if type_contains_unbound_var(&ctx, target_ty) {
                diagnostics.push(Diagnostic::error(
                    "impl target type must be concrete",
                    i.target_ty.span(),
                ));
                continue;
            }

            let mut methods = BTreeMap::new();
            for m in &i.methods {
                let sig = type_from_expr(&mut ctx, &mut f_labels, &m.signature);
                methods.insert(m.name.name.clone(), (m.name.name.clone(), sig));
            }
            impls.push(ImplInfo {
                trait_name,
                target_ty,
                methods,
            });
        }
    }
    for (name, info) in structs.iter() {
        let func_ty = ctx.function(
            info.type_params.clone(),
            info.fields.clone(),
            info.ty,
            Effect::Pure,
        );
        if env.lookup(name).is_none() {
            env.insert_global(Binding {
                name: name.clone(),
                ty: func_ty,
                mutable: false,
                no_shadow: false,
                defined: true,
                moved: false,
                span: Span::dummy(),
                kind: BindingKind::Func {
                    symbol: name.clone(),
                    effect: Effect::Pure,
                    arity: info.fields.len(),
                    builtin: None,
                    type_param_bounds: Vec::new(),
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
            let (mut tps, bounds_vec, _bounds_map) =
                collect_type_params(&mut ctx, &mut f_labels, &f.type_params, &traits, &mut diagnostics);

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
                params,
                result: _,
                effect,
            } = ctx.get(ty)
            {
                if env.lookup_value(&f.name.name).is_some() {
                    diagnostics.push(Diagnostic::error(
                        "name already used by another item",
                        f.name.span,
                    ));
                    continue;
                }
                if enums.contains_key(&f.name.name) || structs.contains_key(&f.name.name) {
                    diagnostics.push(Diagnostic::error(
                        "name already used by another item",
                        f.name.span,
                    ));
                    continue;
                }
                if crate::log::is_verbose() {
                    std::eprintln!("typecheck: registering global func {}", f.name.name);
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
                if let Some(blocked) = shadow_blocked_by_nonshadow(&env, &f.name.name) {
                    if is_callable_binding(blocked) {
                        // 関数同名はオーバーロードとして扱う（noshadow でも許可）。
                    } else {
                    diagnostics.push(Diagnostic::error(
                        format!(
                            "cannot shadow non-shadowable symbol '{}'",
                            f.name.name
                        ),
                        f.name.span,
                    ));
                    diagnostics.push(
                        Diagnostic::error("non-shadowable declaration is here", blocked.span)
                            .with_secondary_label(f.name.span, Some("shadow attempt".into())),
                    );
                    continue;
                    }
                }
                if f.no_shadow
                    && env
                        .lookup_all(&f.name.name)
                        .iter()
                        .any(|b| !is_callable_binding(b))
                {
                    diagnostics.push(Diagnostic::error(
                        format!(
                            "noshadow declaration '{}' conflicts with existing symbol",
                            f.name.name
                        ),
                        f.name.span,
                    ));
                    continue;
                }
                env.remove_duplicate_func(&f.name.name, ty, &ctx);
                let symbol = if type_contains_unbound_var(&ctx, ty) {
                    f.name.name.clone()
                } else {
                    mangle_function_symbol(&f.name.name, ty, &ctx)
                };
                env.insert_global(Binding {
                    name: f.name.name.clone(),
                    ty,
                    mutable: false,
                    no_shadow: f.no_shadow,
                    defined: true,
                    moved: false,
                    span: f.name.span,
                    kind: BindingKind::Func {
                        symbol,
                        effect,
                        arity: params.len(),
                        builtin: None,
                        type_param_bounds: bounds_vec.clone(),
                        captures: Vec::new(),
                    },
                });
            } else {
                diagnostics.push(Diagnostic::error(
                    "function signature must be a function type",
                    f.name.span,
                ));
            }
        }
    }

    for alias in &fn_aliases {
        if enums.contains_key(&alias.name.name) || structs.contains_key(&alias.name.name) {
            diagnostics.push(Diagnostic::error(
                "name already used by another item",
                alias.name.span,
            ));
            continue;
        }
        let targets = env.lookup_all(&alias.target.name);
        if targets.is_empty() {
            diagnostics.push(Diagnostic::error(
                "alias target not found",
                alias.target.span,
            ));
            continue;
        }
        let mut target_infos = Vec::new();
        for target in targets {
            let (symbol, effect, arity, builtin, bounds, captures) = match &target.kind {
                BindingKind::Func {
                    symbol,
                    effect,
                    arity,
                    builtin,
                    type_param_bounds,
                    captures,
                } => (
                    symbol.clone(),
                    *effect,
                    *arity,
                    *builtin,
                    type_param_bounds.clone(),
                    captures.clone(),
                ),
                _ => {
                    diagnostics.push(Diagnostic::error(
                        "alias target is not a function",
                        alias.target.span,
                    ));
                    continue;
                }
            };
            target_infos.push((target.ty, symbol, effect, arity, builtin, bounds, captures));
        }
        for (ty, symbol, effect, arity, builtin, bounds, captures) in target_infos {
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
                diagnostics.push(Diagnostic::error(
                    "name already used by another item",
                    alias.name.span,
                ));
                break;
            }
            if let Some(blocked) = shadow_blocked_by_nonshadow(&env, &alias.name.name) {
                if is_callable_binding(blocked) {
                    // 関数同名はオーバーロードとして扱う（noshadow でも許可）。
                } else {
                diagnostics.push(Diagnostic::error(
                    format!(
                        "cannot shadow non-shadowable symbol '{}'",
                        alias.name.name
                    ),
                    alias.name.span,
                ));
                diagnostics.push(
                    Diagnostic::error("non-shadowable declaration is here", blocked.span)
                        .with_secondary_label(alias.name.span, Some("shadow attempt".into())),
                );
                break;
                }
            }
            if alias.no_shadow
                && env
                    .lookup_all(&alias.name.name)
                    .iter()
                    .any(|b| !is_callable_binding(b))
            {
                diagnostics.push(Diagnostic::error(
                    format!(
                        "noshadow declaration '{}' conflicts with existing symbol",
                        alias.name.name
                    ),
                    alias.name.span,
                ));
                break;
            }
            env.remove_duplicate_func(&alias.name.name, ty, &ctx);
            env.insert_global(Binding {
                name: alias.name.name.clone(),
                ty,
                mutable: false,
                no_shadow: alias.no_shadow,
                defined: true,
                moved: false,
                span: alias.name.span,
                kind: BindingKind::Func {
                    symbol,
                    effect,
                    arity,
                    builtin,
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
                    for tp in &f.type_params {
                        let tv = ctx.fresh_var(Some(tp.name.name.clone()));
                        tmp_labels.insert(tp.name.name.clone(), tv);
                    }
                    let sig_ty = type_from_expr(&mut ctx, &mut tmp_labels, &f.signature);
                    let sig_key = function_signature_string(&ctx, sig_ty);
                    let mut matched: Option<TypeId> = None;
                    for binding in funcs.drain(..) {
                        if function_signature_string(&ctx, binding.ty) == sig_key {
                            matched = Some(binding.ty);
                            break;
                        }
                    }
                    match matched {
                        Some(ty) => ty,
                        None => {
                            diagnostics.push(Diagnostic::error(
                                "function signature does not match any overload",
                                f.name.span,
                            ));
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
                        type_param_bounds.insert(*p_id, p_node.bounds.clone());
                    }
                }
            }
            let mut nested_functions = Vec::new();
            match check_function(
                f,
                f_ty,
                entry.as_ref().map(|n| n == &f.name.name).unwrap_or(false),
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
            name: name.clone(),
            type_params: info.type_params.clone(),
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
            let trait_name = match &i.trait_name {
                Some(tn) => tn.name.clone(),
                None => {
                    diagnostics.push(Diagnostic::error(
                        "inherent impl is not supported yet",
                        i.span,
                    ));
                    continue;
                }
            };
            let trait_info = match traits.get(&trait_name) {
                Some(info) => info,
                None => {
                    diagnostics.push(Diagnostic::error(
                        format!("unknown trait '{}'", trait_name),
                        i.span,
                    ));
                    continue;
                }
            };
            if !i.type_params.is_empty() {
                diagnostics.push(Diagnostic::error(
                    "impl type parameters are not supported yet",
                    i.span,
                ));
                continue;
            }

            let mut impl_methods = Vec::new();
            let mut f_labels = LabelEnv::new();
            let (tps, _bounds_vec, _bounds_map) =
                collect_type_params(&mut ctx, &mut f_labels, &i.type_params, &traits, &mut diagnostics);
            let target_ty = type_from_expr(&mut ctx, &mut f_labels, &i.target_ty);
            if type_contains_unbound_var(&ctx, target_ty) {
                diagnostics.push(Diagnostic::error(
                    "impl target type must be concrete",
                    i.target_ty.span(),
                ));
                continue;
            }
            f_labels.insert(String::from("Self"), target_ty);
            let prev_self = label_env.insert(String::from("Self"), target_ty);

            let mut seen_methods = BTreeSet::new();
            for m in &i.methods {
                if !seen_methods.insert(m.name.name.clone()) {
                    diagnostics.push(Diagnostic::error(
                        "duplicate method in impl",
                        m.name.span,
                    ));
                    continue;
                }
                if !m.type_params.is_empty() {
                    diagnostics.push(Diagnostic::error(
                        "impl methods cannot have type parameters yet",
                        m.name.span,
                    ));
                    continue;
                }
                let trait_sig = match trait_info.methods.get(&m.name.name) {
                    Some(sig) => *sig,
                    None => {
                        diagnostics.push(Diagnostic::error(
                            format!("method '{}' not found in trait '{}'", m.name.name, trait_name),
                            m.name.span,
                        ));
                        continue;
                    }
                };
                let mut mapping = BTreeMap::new();
                mapping.insert(ctx.resolve_id(trait_info.self_ty), ctx.resolve_id(target_ty));
                let expected_sig = ctx.substitute(trait_sig, &mapping);
                let actual_sig = type_from_expr(&mut ctx, &mut f_labels, &m.signature);
                if !type_signature_matches(&ctx, expected_sig, actual_sig) {
                    diagnostics.push(Diagnostic::error(
                        "impl method signature does not match trait",
                        m.name.span,
                    ));
                    continue;
                }
                let mut nested_functions = Vec::new();
                let mut checked = match check_function(
                    m,
                    expected_sig,
                    false,
                    &[],
                    &mut ctx,
                    &mut env,
                    &mut label_env,
                    &mut strings,
                    &enums,
                    &structs,
                    &mut instantiations,
                    BTreeMap::new(),
                    &traits,
                    &impls,
                    &mut nested_functions,
                ) {
                    Ok(checked) => checked,
                    Err(mut diags) => {
                        diagnostics.append(&mut diags);
                        continue;
                    }
                };
                diagnostics.extend(checked.diagnostics);
                let mut func = checked.function;
                let mangled = mangle_impl_method(&trait_name, &m.name.name, target_ty, &ctx);
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
                    diagnostics.push(Diagnostic::error(
                        format!("missing method '{}' for trait '{}'", trait_method, trait_name),
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
                trait_name,
                type_args: tps,
                target_ty,
                methods: impl_methods,
                span: i.target_ty.span(),
            });
        }
    }

    let resolved_entry = if let Some(name) = entry {
        let bindings = env.lookup_all(&name);
        let mut func_symbols = Vec::new();
        for b in bindings {
            if let BindingKind::Func { symbol, .. } = &b.kind {
                func_symbols.push(symbol.clone());
            }
        }
        if func_symbols.len() == 1 {
            Some(func_symbols.remove(0))
        } else {
            diagnostics.push(Diagnostic::error(
                "entry function is missing or ambiguous",
                Span::dummy(),
            ));
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

// ---------------------------------------------------------------------
// Function checking
// ---------------------------------------------------------------------

fn check_function(
    f: &FnDef,
    func_ty: TypeId,
    is_entry: bool,
    captured_params: &[(String, TypeId)],
    ctx: &mut TypeCtx,
    env: &mut Env,
    labels: &mut LabelEnv,
    strings: &mut StringTable,
    enums: &BTreeMap<String, EnumInfo>,
    structs: &BTreeMap<String, StructInfo>,
    instantiations: &mut BTreeMap<String, Vec<Vec<TypeId>>>,
    type_param_bounds: BTreeMap<TypeId, Vec<String>>,
    traits: &BTreeMap<String, TraitInfo>,
    impls: &Vec<ImplInfo>,
    generated_functions: &mut Vec<HirFunction>,
) -> Result<CheckedFunction, Vec<Diagnostic>> {
    let mut diags = Vec::new();
    let (params_ty, result_ty, effect) = match ctx.get(func_ty) {
        TypeKind::Function {
            params,
            result,
            effect,
            ..
        } => (params, result, effect),
        _ => {
            diags.push(Diagnostic::error(
                "function signature must be a function type",
                f.name.span,
            ));
            return Err(diags);
        }
    };
    if params_ty.len() != captured_params.len() + f.params.len() {
        diags.push(Diagnostic::error(
            "parameter count mismatch with signature",
            f.name.span,
        ));
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
            moved: false,
            span: f.name.span,
            kind: BindingKind::Var,
        });
    }
    for (param, ty) in f.params.iter().zip(params_ty.iter().skip(captured_params.len())) {
        emit_shadow_warning(&mut diags, env, &param.name, param.span, "parameter");
        let _ = env.insert_local(Binding {
            name: param.name.clone(),
            ty: *ty,
            mutable: false,
            no_shadow: false,
            defined: true,
            moved: false,
            span: param.span,
            kind: BindingKind::Var,
        });
    }

    let (body, diag_out) = {
        let mut checker = BlockChecker {
            ctx,
            env,
            labels,
            string_table: strings,
            diagnostics: Vec::new(),
            current_effect: if is_entry { Effect::Impure } else { effect },
            enums,
            structs,
            instantiations,
            type_param_bounds: type_param_bounds.clone(),
            traits,
            impls,
            generated_functions,
        };

        let body_res = match &f.body {
            FnBody::Parsed(b) => match checker.check_block(b, 0, true) {
                Some((blk, _val)) => {
                    if checker.ctx.unify(blk.ty, result_ty).is_err() {
                        checker.diagnostics.push(Diagnostic::error(
                            "return type does not match signature",
                            f.name.span,
                        ));
                    }
                    HirBody::Block(blk)
                }
                None => {
                    return Err(checker.diagnostics);
                }
            },
            FnBody::Wasm(wb) => HirBody::Wasm(wb.clone()),
        };
        (body_res, checker.diagnostics)
    };

    env.pop_scope();
    let has_error = diag_out
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error));

    let out_name = env
        .lookup_func_symbol(&f.name.name, func_ty, ctx)
        .unwrap_or_else(|| {
            if type_contains_unbound_var(ctx, func_ty) {
                f.name.name.clone()
            } else {
                mangle_function_symbol(&f.name.name, func_ty, ctx)
            }
        });
    let function = HirFunction {
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
    if has_error {
        Err(diag_out)
    } else {
        Ok(CheckedFunction {
            function,
            diagnostics: diag_out,
        })
    }
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
    current_effect: Effect,
    enums: &'a BTreeMap<String, EnumInfo>,
    structs: &'a BTreeMap<String, StructInfo>,
    instantiations: &'a mut BTreeMap<String, Vec<Vec<TypeId>>>, // new
    type_param_bounds: BTreeMap<TypeId, Vec<String>>,
    traits: &'a BTreeMap<String, TraitInfo>,
    impls: &'a Vec<ImplInfo>,
    generated_functions: &'a mut Vec<HirFunction>,
}

impl<'a> BlockChecker<'a> {
    fn user_visible_arity(&self, func_expr: &HirExpr, total_param_len: usize) -> usize {
        if let HirExprKind::Var(name) = &func_expr.kind {
            let bindings = self.env.lookup_all_callables(name);
            if !bindings.is_empty() {
                let mut cap_len: Option<usize> = None;
                for b in bindings {
                    if let BindingKind::Func { captures, .. } = &b.kind {
                        match cap_len {
                            Some(prev) if prev != captures.len() => return total_param_len,
                            Some(_) => {}
                            None => cap_len = Some(captures.len()),
                        }
                    }
                }
                if let Some(c) = cap_len {
                    return total_param_len.saturating_sub(c);
                }
            }
        }
        total_param_len
    }

    fn collect_bound_names_from_prefix(expr: &PrefixExpr, out: &mut BTreeSet<String>) {
        for item in &expr.items {
            match item {
                PrefixItem::Symbol(Symbol::Let { name, .. }) => {
                    out.insert(name.name.clone());
                }
                PrefixItem::Block(b, _) => {
                    Self::collect_bound_names_from_block(b, out);
                }
                PrefixItem::Match(m, _) => {
                    for arm in &m.arms {
                        if let Some(b) = &arm.bind {
                            out.insert(b.name.clone());
                        }
                        Self::collect_bound_names_from_block(&arm.body, out);
                    }
                }
                _ => {}
            }
        }
    }

    fn collect_bound_names_from_block(block: &Block, out: &mut BTreeSet<String>) {
        for stmt in &block.items {
            match stmt {
                Stmt::Expr(e) | Stmt::ExprSemi(e, _) => {
                    Self::collect_bound_names_from_prefix(e, out);
                }
                Stmt::FnDef(f) => {
                    out.insert(f.name.name.clone());
                }
                _ => {}
            }
        }
    }

    fn collect_ref_names_from_prefix(expr: &PrefixExpr, out: &mut BTreeSet<String>) {
        for item in &expr.items {
            match item {
                PrefixItem::Symbol(Symbol::Ident(id, _, _)) => {
                    out.insert(id.name.clone());
                }
                PrefixItem::Block(b, _) => {
                    Self::collect_ref_names_from_block(b, out);
                }
                PrefixItem::Match(m, _) => {
                    Self::collect_ref_names_from_prefix(&m.scrutinee, out);
                    for arm in &m.arms {
                        Self::collect_ref_names_from_block(&arm.body, out);
                    }
                }
                PrefixItem::Tuple(items, _) => {
                    for it in items {
                        Self::collect_ref_names_from_prefix(it, out);
                    }
                }
                PrefixItem::Group(inner, _) => {
                    Self::collect_ref_names_from_prefix(inner, out);
                }
                _ => {}
            }
        }
    }

    fn collect_ref_names_from_block(block: &Block, out: &mut BTreeSet<String>) {
        for stmt in &block.items {
            match stmt {
                Stmt::Expr(e) | Stmt::ExprSemi(e, _) => {
                    Self::collect_ref_names_from_prefix(e, out);
                }
                Stmt::FnDef(_) => {}
                _ => {}
            }
        }
    }

    fn collect_nested_fn_captures(&self, f: &FnDef) -> Vec<(String, TypeId)> {
        let FnBody::Parsed(body) = &f.body else {
            return Vec::new();
        };
        let mut refs = BTreeSet::new();
        let mut bounds = BTreeSet::new();
        for p in &f.params {
            bounds.insert(p.name.clone());
        }
        Self::collect_bound_names_from_block(body, &mut bounds);
        Self::collect_ref_names_from_block(body, &mut refs);
        let mut captures = Vec::new();
        for name in refs {
            if bounds.contains(&name) || name == f.name.name {
                continue;
            }
            if let Some(b) = self.env.lookup_any(&name) {
                if matches!(b.kind, BindingKind::Var) {
                    captures.push((name, b.ty));
                }
            }
        }
        captures
    }

    fn find_outer_function_consumer(
        &self,
        stack: &[StackEntry],
        inner_pos: usize,
        min_func_pos: usize,
    ) -> Option<usize> {
        for j in (min_func_pos..inner_pos).rev() {
            if !stack[j].auto_call {
                continue;
            }
            let rty = self.ctx.resolve_id(stack[j].ty);
            let TypeKind::Function { params, .. } = self.ctx.get(rty) else {
                continue;
            };
            let total_arity = params.len();
            let arity = self.user_visible_arity(&stack[j].expr, total_arity);
            if stack.len() < j + 1 + arity {
                continue;
            }
            if inner_pos < j + 1 {
                continue;
            }
            let user_arg_idx = inner_pos - (j + 1);
            if user_arg_idx >= arity {
                continue;
            }
            let capture_len = total_arity.saturating_sub(arity);
            let arg_idx = capture_len + user_arg_idx;
            if arg_idx >= total_arity {
                continue;
            }
            let pty = self.ctx.resolve_id(params[arg_idx]);
            if matches!(self.ctx.get(pty), TypeKind::Function { .. }) {
                return Some(j);
            }
        }
        None
    }

    fn is_concrete_type(&self, ty: TypeId) -> bool {
        let resolved = self.ctx.resolve_id(ty);
        !matches!(self.ctx.get(resolved), TypeKind::Var(v) if v.binding.is_none())
    }

    fn type_param_has_bound(&self, ty: TypeId, trait_name: &str) -> bool {
        let resolved = self.ctx.resolve_id(ty);
        if let Some(bounds) = self.type_param_bounds.get(&resolved) {
            return bounds.iter().any(|b| b == trait_name);
        }

        // 型変数が他の型変数へ束縛された場合、resolve 後の TypeId が
        // 直接 type_param_bounds に存在しないことがあるため、正規化後 ID でも照合する。
        if self.type_param_bounds.iter().any(|(tp, bounds)| {
            self.ctx.resolve_id(*tp) == resolved && bounds.iter().any(|b| b == trait_name)
        }) {
            return true;
        }

        // `.T` の明示型引数が同一スコープの別 TypeId として現れる経路があるため、
        // 型変数ラベルが一致する場合も同じ境界として扱う。
        let label = match self.ctx.get(resolved) {
            TypeKind::Var(v) => v.label.clone(),
            _ => None,
        };
        let Some(label) = label else {
            return false;
        };
        self.type_param_bounds.iter().any(|(tp, bounds)| {
            let same_label = match self.ctx.get(self.ctx.resolve_id(*tp)) {
                TypeKind::Var(v) => v.label.as_deref() == Some(label.as_str()),
                _ => false,
            };
            same_label && bounds.iter().any(|b| b == trait_name)
        })
    }

    fn trait_bound_satisfied(&self, trait_name: &str, ty: TypeId) -> bool {
        if !self.is_concrete_type(ty) {
            return self.type_param_has_bound(ty, trait_name);
        }
        let ty_name = self.ctx.type_to_string(self.ctx.resolve_id(ty));
        self.impls.iter().any(|imp| {
            imp.trait_name.as_deref() == Some(trait_name)
                && self.ctx.type_to_string(self.ctx.resolve_id(imp.target_ty)) == ty_name
        })
    }

    fn resolve_field_access(
        &mut self,
        base_ty: TypeId,
        idx: FieldIdx,
        span: Span,
    ) -> Option<(TypeId, usize)> {
        let resolved_ty = self.ctx.resolve(base_ty);
        match self.ctx.get(resolved_ty) {
            TypeKind::Struct {
                fields,
                field_names,
                ..
            } => match idx {
                FieldIdx::Index(i) => {
                    if i < fields.len() {
                        Some((fields[i], i * 4))
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            format!("struct index out of bounds: {}", i),
                            span,
                        ));
                        None
                    }
                }
                FieldIdx::Name(name) => {
                    if let Some(i) = field_names.iter().position(|n| *n == name) {
                        Some((fields[i], i * 4))
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            format!("struct has no field {}", name),
                            span,
                        ));
                        None
                    }
                }
            },
            TypeKind::Tuple { items } => match idx {
                FieldIdx::Index(i) => {
                    if i < items.len() {
                        Some((items[i], i * 4))
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            format!("tuple index out of bounds: {}", i),
                            span,
                        ));
                        None
                    }
                }
                FieldIdx::Name(name) => {
                    if let Ok(i) = name.parse::<usize>() {
                        if i < items.len() {
                            Some((items[i], i * 4))
                        } else {
                            self.diagnostics.push(Diagnostic::error(
                                format!("tuple index out of bounds: {}", i),
                                span,
                            ));
                            None
                        }
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            format!("invalid tuple field access: {}", name),
                            span,
                        ));
                        None
                    }
                }
            },
            TypeKind::Apply { base, args } => {
                let base_ty = self.ctx.resolve(base);
                match self.ctx.get(base_ty) {
                    TypeKind::Struct {
                        type_params,
                        fields,
                        field_names,
                        ..
                    } => {
                        let mut mapping = BTreeMap::new();
                        for (tp, arg) in type_params.iter().zip(args.iter()) {
                            mapping.insert(*tp, *arg);
                        }
                        let substituted_fields = fields
                            .iter()
                            .map(|f| self.ctx.substitute(*f, &mapping))
                            .collect::<Vec<_>>();
                        match idx {
                            FieldIdx::Index(i) => {
                                if i < substituted_fields.len() {
                                    Some((substituted_fields[i], i * 4))
                                } else {
                                    self.diagnostics.push(Diagnostic::error(
                                        format!("generic struct index out of bounds: {}", i),
                                        span,
                                    ));
                                    None
                                }
                            }
                            FieldIdx::Name(name) => {
                                if let Some(i) = field_names.iter().position(|n| *n == name) {
                                    Some((substituted_fields[i], i * 4))
                                } else {
                                    self.diagnostics.push(Diagnostic::error(
                                        format!("generic struct has no field {}", name),
                                        span,
                                    ));
                                    None
                                }
                            }
                        }
                    }
                    _ => {
                        self.diagnostics
                            .push(Diagnostic::error("cannot access field on this type", span));
                        None
                    }
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "cannot access field on non-composite type",
                    span,
                ));
                None
            }
        }
    }

    fn check_block(
        &mut self,
        block: &Block,
        base_depth: usize,
        new_scope: bool,
    ) -> Option<(HirBlock, Option<TypeId>)> {
        let old_effect = self.current_effect;

        if new_scope {
            self.env.push_scope();
        }

        // Hoist let (non-mut) and nested fn signatures
        for (i, stmt) in block.items.iter().enumerate() {
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
                        self.diagnostics.push(Diagnostic::error(
                            format!("cannot shadow non-shadowable symbol '{}'", name.name),
                            name.span,
                        ));
                        self.diagnostics.push(
                            Diagnostic::error("non-shadowable declaration is here", blocked.span)
                                .with_secondary_label(name.span, Some("shadow attempt".into())),
                        );
                        continue;
                    }
                    if *no_shadow && self.env.lookup_any(&name.name).is_some() {
                        self.diagnostics.push(Diagnostic::error(
                            format!(
                                "noshadow declaration '{}' conflicts with existing symbol",
                                name.name
                            ),
                            name.span,
                        ));
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
                        moved: false,
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
                            // 関数同名はオーバーロードとして扱う（noshadow でも許可）。
                        } else {
                        self.diagnostics.push(Diagnostic::error(
                            format!("cannot shadow non-shadowable symbol '{}'", f.name.name),
                            f.name.span,
                        ));
                        self.diagnostics.push(
                            Diagnostic::error("non-shadowable declaration is here", blocked.span)
                                .with_secondary_label(f.name.span, Some("shadow attempt".into())),
                        );
                        continue;
                        }
                    }
                    if f.no_shadow
                        && self
                            .env
                            .lookup_all(&f.name.name)
                            .iter()
                            .any(|b| !is_callable_binding(b))
                    {
                        self.diagnostics.push(Diagnostic::error(
                            format!(
                                "noshadow declaration '{}' conflicts with existing symbol",
                                f.name.name
                            ),
                            f.name.span,
                        ));
                        continue;
                    }
                    if !captures.is_empty() {
                        let mut lifted_params = captures.iter().map(|(_, t)| *t).collect::<Vec<_>>();
                        lifted_params.extend(params.iter().copied());
                        ty = self
                            .ctx
                            .function(type_params.clone(), lifted_params, result, effect);
                    }
                    let has_non_callable_conflict = self
                        .env
                        .lookup_all(&f.name.name)
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
                        moved: false,
                        span: f.name.span,
                        kind: BindingKind::Func {
                            symbol: f.name.name.clone(),
                            effect,
                            arity: params.len(),
                            builtin: None,
                            type_param_bounds: Vec::new(),
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

        for (idx, stmt) in block.items.iter().enumerate() {
            // Drop stray unit between lines: [X, ()] -> [X]
            if stack.len() == base_depth + 1 {
                if matches!(self.ctx.get(stack.last().unwrap().ty), TypeKind::Unit) {
                    stack.pop();
                }
            }

            match stmt {
                Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
                    match self.check_prefix(expr, base_depth, &mut stack) {
                        Some((typed, dropped_from_prefix)) => {
                            let is_last_expr = Some(idx) == last_expr_idx;
                            let mut drop_result = !is_last_expr;
                            if matches!(stmt, Stmt::ExprSemi(_, _)) {
                                // `;` explicitly discards the statement value even
                                // when it appears on the last line of a block.
                                drop_result = true;
                            }

                            if dropped_from_prefix {
                                self.diagnostics.push(Diagnostic::error(
                                    "expression left extra values on the stack",
                                    typed.span,
                                ));
                            }

                            // If there was an explicit semicolon token, require that the
                            // statement left exactly one value on the stack; otherwise
                            // emit a diagnostic and recover.
                            if let Stmt::ExprSemi(_, semi_span) = stmt {
                                if stack.len() != base_depth + 1 {
                                    let sp = semi_span.unwrap_or(typed.span);
                                    self.diagnostics.push(Diagnostic::error(
                                        "statement must leave exactly one value on the stack",
                                        sp,
                                    ));
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
                    let (f_ty, captures) = {
                        let funcs: Vec<&Binding> = self.env.lookup_all_callables(&f.name.name);
                        if funcs.is_empty() {
                            continue;
                        }
                        if funcs.len() == 1 {
                            let caps = match &funcs[0].kind {
                                BindingKind::Func { captures, .. } => captures.clone(),
                                _ => Vec::new(),
                            };
                            (funcs[0].ty, caps)
                        } else {
                            let mut tmp_labels = LabelEnv::new();
                            for tp in &f.type_params {
                                let tv = self.ctx.fresh_var(Some(tp.name.name.clone()));
                                tmp_labels.insert(tp.name.name.clone(), tv);
                            }
                            let sig_ty = type_from_expr(self.ctx, &mut tmp_labels, &f.signature);
                            let sig_key = function_signature_string(self.ctx, sig_ty);
                            let mut matched: Option<TypeId> = None;
                            for binding in &funcs {
                                if function_signature_string(self.ctx, binding.ty) == sig_key {
                                    matched = Some(binding.ty);
                                    break;
                                }
                            }
                            match matched {
                                Some(ty) => {
                                    let mut caps = Vec::new();
                                    for b in &funcs {
                                        if b.ty == ty {
                                            if let BindingKind::Func { captures, .. } = &b.kind {
                                                caps = captures.clone();
                                            }
                                            break;
                                        }
                                    }
                                    (ty, caps)
                                }
                                None => {
                                    self.diagnostics.push(Diagnostic::error(
                                        "function signature does not match any overload",
                                        f.name.span,
                                    ));
                                    continue;
                                }
                            }
                        }
                    };
                    let mut nested_bounds = BTreeMap::new();
                    if let TypeKind::Function { type_params, .. } = self.ctx.get(f_ty) {
                        for (p_node, p_id) in f.type_params.iter().zip(type_params.iter()) {
                            self.labels.insert(p_node.name.name.clone(), *p_id);
                            if !p_node.bounds.is_empty() {
                                nested_bounds.insert(*p_id, p_node.bounds.clone());
                            }
                        }
                    }
                    match check_function(
                        f,
                        f_ty,
                        false,
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

        if std::env::var("NEPL_DUMP_HIR").is_ok() {
            dump!("NEPL_DUMP_HIR: block span={:?} lines={} final_ty={:?} value_ty={:?}", block.span, lines.len(), final_ty, value_ty);
            // Print env scopes and a compact preview of the HIR lines for diagnosis
            dump!("NEPL_DUMP_HIR: env scopes=\n{:?}", self.env.scopes);
            for (i, l) in lines.iter().enumerate() {
                dump!("NEPL_DUMP_HIR: line {} -> expr.kind = {:?}, ty={:?}, drop={}", i, l.expr.kind, l.expr.ty, l.drop_result);
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

    fn check_prefix(
        &mut self,
        expr: &PrefixExpr,
        base_depth: usize,
        stack: &mut Vec<StackEntry>,
    ) -> Option<(HirExpr, bool)> {
        // Track indices of functions on the stack to avoid linear scanning in reduce_calls.
        // This makes reduction O(1) amortized instead of O(N^2).
        let mut open_calls: Vec<usize> = Vec::new();
        // Initialize open_calls from existing stack (if any)
        for (i, entry) in stack.iter().enumerate() {
             let rty = self.ctx.resolve(entry.ty);
             if entry.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. }) {
                 open_calls.push(i);
             }
        }

        let mut dropped = false;
        let mut last_expr: Option<HirExpr> = None;
        let mut pipe_pending: Option<StackEntry> = None;
        // (target_type, stack_depth_when_annotation_appeared)
        let mut pending_ascription: Option<(TypeId, usize)> = None;

        // Try to apply a pending ascription when the next expression is complete.
        fn try_apply_pending_ascription(
            this: &mut BlockChecker,
            stack: &mut Vec<StackEntry>,
            pending: &mut Option<(TypeId, usize)>,
        ) {
            let Some((target_ty, base_len)) = *pending else {
                return;
            };
            // The next expression is complete exactly when the stack returns to base_len + 1
            if stack.len() == base_len + 1 {
                let top = stack.last().unwrap();
                // Do not apply to functions
                if !matches!(this.ctx.get(top.ty), TypeKind::Function { .. }) {
                    let sp = top.expr.span;
                    this.apply_ascription(stack.as_mut_slice(), target_ty, sp);
                    *pending = None;
                }
            }
        }
        for (idx, item) in expr.items.iter().enumerate() {
            // std::eprintln!("  Item: {:?}", item);
            let next_is_pipe = matches!(expr.items.get(idx + 1), Some(PrefixItem::Pipe(_)));
            match item {
                PrefixItem::Literal(lit, span) => {
                    let (ty, hir) = match lit {
                        Literal::Int(text) => {
                            let v = match parse_i32_literal(text) {
                                Some(v) => v,
                                None => {
                                    self.diagnostics.push(Diagnostic::error(
                                        "invalid integer literal",
                                        *span,
                                    ));
                                    0
                                }
                            };
                            (self.ctx.i32(), HirExprKind::LiteralI32(v))
                        }
                        Literal::Float(text) => {
                            let v = text.parse::<f32>().unwrap_or(0.0);
                            (self.ctx.f32(), HirExprKind::LiteralF32(v))
                        }
                        Literal::Bool(b) => (self.ctx.bool(), HirExprKind::LiteralBool(*b)),
                        Literal::Str(s) => {
                            let id = self.string_table.intern(s.clone());
                            (self.ctx.str(), HirExprKind::LiteralStr(id))
                        }
                        Literal::Unit => (self.ctx.unit(), HirExprKind::Unit),
                    };
                    stack.push(StackEntry {
                        ty,
                        expr: HirExpr {
                            ty,
                            kind: hir,
                            span: *span,
                        },
                        type_args: Vec::new(),
                        assign: None,
                            auto_call: true,
                    });
                    last_expr = Some(stack.last().unwrap().expr.clone());
                }
                PrefixItem::Symbol(sym) => match sym {
                    Symbol::Ident(id, type_args, forced_value) => {
                        if let Some(entry) = self.resolve_dotted_field_symbol(id, *forced_value) {
                            stack.push(entry);
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        } else if let Some(binding) = {
                            // In head position of a prefix expression, prefer callable symbols
                            // over value symbols when both names coexist in the same scope.
                            // This keeps `add add 1` semantics stable even if `add` is also a value.
                            let preferred_callable = if !*forced_value
                                && stack.is_empty()
                                && expr.items.get(idx + 1).is_some()
                            {
                                self.env.lookup_callable_any(&id.name)
                            } else {
                                None
                            };
                            preferred_callable.or_else(|| self.env.lookup(&id.name))
                        } {
                            let ty = binding.ty;
                            let auto_call = match binding.kind {
                                BindingKind::Func { .. } => !*forced_value,
                                _ => true,
                            };
                            let explicit_args = match binding.kind {
                                BindingKind::Func { .. } => {
                                    let mut args = Vec::new();
                                    for arg_expr in type_args {
                                        args.push(type_from_expr(self.ctx, self.labels, arg_expr));
                                    }
                                    args
                                }
                                _ => {
                                    if !type_args.is_empty() {
                                        self.diagnostics.push(Diagnostic::error(
                                            "type arguments are not allowed for variables",
                                            id.span,
                                        ));
                                    }
                                    Vec::new()
                                }
                            };
                            stack.push(StackEntry {
                                ty,
                                expr: HirExpr {
                                    ty,
                                    kind: HirExprKind::Var(id.name.clone()),
                                    span: id.span,
                                },
                                type_args: explicit_args,
                                assign: None,
                                auto_call,
                            });
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        } else {
                            let mut lookup_name = id.name.clone();
                            let mut bindings = self.env.lookup_all(&lookup_name);
                            if bindings.is_empty() {
                                if let Some((ns, member)) = parse_variant_name(&id.name) {
                                    if !self.enums.contains_key(ns) && !self.traits.contains_key(ns)
                                    {
                                        let alt = self.env.lookup_all(member);
                                        if !alt.is_empty() {
                                            lookup_name = member.to_string();
                                            bindings = alt;
                                        }
                                    }
                                }
                            }
                            if !bindings.is_empty() {
                                if let Some(binding) = self.env.lookup_value(&lookup_name) {
                                    if !type_args.is_empty() {
                                        self.diagnostics.push(Diagnostic::error(
                                            "type arguments are not allowed for variables",
                                            id.span,
                                        ));
                                    }
                                    let ty = binding.ty;
                                    stack.push(StackEntry {
                                        ty,
                                        expr: HirExpr {
                                            ty,
                                            kind: HirExprKind::Var(lookup_name.clone()),
                                            span: id.span,
                                        },
                                        type_args: Vec::new(),
                                        assign: None,
                                        auto_call: !*forced_value,
                                    });
                                    last_expr = Some(stack.last().unwrap().expr.clone());
                                } else {
                                    let mut effect = None;
                                    let mut arity = None;
                                    for b in &bindings {
                                        if let BindingKind::Func {
                                            effect: e,
                                            arity: a,
                                            ..
                                        } = b.kind
                                        {
                                            if effect.is_none() {
                                                effect = Some(e);
                                            } else if effect != Some(e) {
                                                self.diagnostics.push(Diagnostic::error(
                                                    "overloaded functions must have the same effect",
                                                    id.span,
                                                ));
                                            }
                                            if arity.is_none() {
                                                arity = Some(a);
                                            } else if arity != Some(a) {
                                                self.diagnostics.push(Diagnostic::error(
                                                    "overloaded functions must have the same arity",
                                                    id.span,
                                                ));
                                            }
                                        }
                                    }
                                    let arity = arity.unwrap_or(0);
                                    let effect = effect.unwrap_or(Effect::Pure);
                                    let mut explicit_args = Vec::new();
                                    if !type_args.is_empty() {
                                        for arg_expr in type_args {
                                            explicit_args
                                                .push(type_from_expr(self.ctx, self.labels, arg_expr));
                                        }
                                    }
                                    let mut params = Vec::new();
                                    for _ in 0..arity {
                                        params.push(self.ctx.fresh_var(None));
                                    }
                                    let result = self.ctx.fresh_var(None);
                                    let ty = self
                                        .ctx
                                        .function(Vec::new(), params, result, effect);
                                    stack.push(StackEntry {
                                        ty,
                                        expr: HirExpr {
                                            ty,
                                            kind: HirExprKind::Var(lookup_name.clone()),
                                            span: id.span,
                                        },
                                        type_args: explicit_args,
                                        assign: None,
                            auto_call: true,
                                    });
                                    last_expr = Some(stack.last().unwrap().expr.clone());
                                }
                            } else if let Some((trait_name, method_name)) = parse_variant_name(&id.name)
                            {
                                if let Some(trait_info) = self.traits.get(trait_name) {
                                    if !type_args.is_empty() {
                                        self.diagnostics.push(Diagnostic::error(
                                            "type arguments are not supported for trait methods yet",
                                            id.span,
                                        ));
                                        return None;
                                    }
                                    if let Some(sig) = trait_info.methods.get(method_name) {
                                        let (inst_ty, args) = self.ctx.instantiate(*sig);
                                        stack.push(StackEntry {
                                            ty: inst_ty,
                                            expr: HirExpr {
                                                ty: inst_ty,
                                                kind: HirExprKind::Var(id.name.clone()),
                                                span: id.span,
                                            },
                                            type_args: args,
                                            assign: None,
                                            auto_call: !*forced_value,
                                        });
                                        last_expr = Some(stack.last().unwrap().expr.clone());
                                    } else {
                                        self.diagnostics.push(Diagnostic::error(
                                            format!(
                                                "unknown method '{}' for trait '{}'",
                                                method_name, trait_name
                                            ),
                                            id.span,
                                        ));
                                        return None;
                                    }
                                } else {
                                    self.diagnostics
                                        .push(Diagnostic::error("undefined identifier", id.span));
                                }
                            } else {
                                self.diagnostics
                                    .push(Diagnostic::error("undefined identifier", id.span));
                            }
                        }
                    }
                    Symbol::Let {
                        name,
                        mutable,
                        no_shadow,
                    } => {
                        // Use current-scope lookup so `let` always creates a local binding
                        // (shadowing outer bindings) rather than reusing an outer binding.
                        let ty = if let Some(b) = self.env.lookup_current(&name.name) {
                            if b.no_shadow && b.span != name.span {
                                self.diagnostics.push(Diagnostic::error(
                                    format!("cannot shadow non-shadowable symbol '{}'", name.name),
                                    name.span,
                                ));
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "non-shadowable declaration is here",
                                        b.span,
                                    )
                                    .with_secondary_label(
                                        name.span,
                                        Some("shadow attempt".into()),
                                    ),
                                );
                                return None;
                            }
                            b.ty
                        } else {
                            if let Some(blocked) = shadow_blocked_by_nonshadow(self.env, &name.name)
                            {
                                self.diagnostics.push(Diagnostic::error(
                                    format!(
                                        "cannot shadow non-shadowable symbol '{}'",
                                        name.name
                                    ),
                                    name.span,
                                ));
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "non-shadowable declaration is here",
                                        blocked.span,
                                    )
                                    .with_secondary_label(
                                        name.span,
                                        Some("shadow attempt".into()),
                                    ),
                                );
                                return None;
                            }
                            if *no_shadow && self.env.lookup_any(&name.name).is_some() {
                                self.diagnostics.push(Diagnostic::error(
                                    format!(
                                        "noshadow declaration '{}' conflicts with existing symbol",
                                        name.name
                                    ),
                                    name.span,
                                ));
                                return None;
                            }
                            let t = self.ctx.fresh_var(None);
                            emit_shadow_warning(
                                &mut self.diagnostics,
                                self.env,
                                &name.name,
                                name.span,
                                if *mutable { "let mut" } else { "let" },
                            );
                            let _ = self.env.insert_local(Binding {
                                name: name.name.clone(),
                                ty: t,
                                mutable: *mutable,
                                no_shadow: *no_shadow,
                                defined: false,
                                moved: false,
                                span: name.span,
                                kind: BindingKind::Var,
                            });
                            dump!("typecheck: inserted local binding {}", name.name);
                            t
                        };
                        let func_ty =
                            self.ctx
                                .function(Vec::new(), vec![ty], self.ctx.unit(), Effect::Pure);
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var(name.name.clone()),
                                span: name.span,
                            },
                            type_args: Vec::new(),
                            assign: Some(AssignKind::Let),
                            auto_call: false,
                        });
                        // defer applying ascription until the expression is complete
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                    Symbol::Set { name } => {
                        if let Some(binding) = self
                            .env
                            .lookup_current(&name.name)
                            .or_else(|| self.env.lookup_value(&name.name))
                        {
                            if !binding.mutable {
                                self.diagnostics.push(Diagnostic::error(
                                    "cannot set immutable variable",
                                    name.span,
                                ));
                            }
                            let func_ty = self.ctx.function(
                                Vec::new(),
                                vec![binding.ty],
                                self.ctx.unit(),
                                Effect::Impure,
                            );
                            stack.push(StackEntry {
                                ty: func_ty,
                                expr: HirExpr {
                                    ty: func_ty,
                                    kind: HirExprKind::Var(name.name.clone()),
                                    span: name.span,
                                },
                                type_args: Vec::new(),
                                assign: Some(AssignKind::Set),
                            auto_call: true,
                            });
                            // defer applying ascription until the expression is complete
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        } else {
                            self.diagnostics
                                .push(Diagnostic::error("undefined variable", name.span));
                        }
                    }
                    Symbol::AddrOf(span) => {
                        if crate::log::is_verbose() {
                            std::eprintln!("check_prefix: pushing AddrOf to stack");
                        }
                        let a = self.ctx.fresh_var(None);
                        let ref_a = self.ctx.reference(a, false);
                        let func_ty = self.ctx.function(Vec::new(), vec![a], ref_a, Effect::Pure);
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var("&&addr_of".to_string()),
                                span: *span,
                            },
                            type_args: Vec::new(),
                            assign: Some(AssignKind::AddrOf),
                            auto_call: true,
                        });
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                    Symbol::Deref(span) => {
                        let a = self.ctx.fresh_var(None);
                        let ref_a = self.ctx.reference(a, false);
                        let func_ty = self.ctx.function(Vec::new(), vec![ref_a], a, Effect::Pure);
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var("&&deref".to_string()),
                                span: *span,
                            },
                            type_args: Vec::new(),
                            assign: Some(AssignKind::Deref),
                            auto_call: true,
                        });
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                    Symbol::If(sp) => {
                        let t_cond = self.ctx.bool();
                        let t_branch = self.ctx.fresh_var(None);
                        let func_ty = self.ctx.function(
                            Vec::new(),
                            vec![t_cond, t_branch, t_branch],
                            t_branch,
                            Effect::Pure,
                        );
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var("if".to_string()),
                                span: *sp,
                            },
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: true,
                        });
                        // defer applying ascription until the expression is complete
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                    Symbol::While(sp) => {
                        let t_cond = self.ctx.bool();
                        let func_ty = self.ctx.function(
                            Vec::new(),
                            vec![t_cond, self.ctx.unit()],
                            self.ctx.unit(),
                            Effect::Pure,
                        );
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var("while".to_string()),
                                span: *sp,
                            },
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: true,
                        });
                        // defer applying ascription until the expression is complete
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                },
                PrefixItem::Intrinsic(intrin, sp) => {
                    let mut type_args = Vec::new();
                    for t in &intrin.type_args {
                        type_args.push(type_from_expr(self.ctx, self.labels, t));
                    }
                    
                    let mut args = Vec::new();
                    for arg in &intrin.args {
                        let mut arg_stack = Vec::new();
                        if let Some((hexpr, _)) = self.check_prefix(arg, 0, &mut arg_stack) {
                            args.push(hexpr);
                        } else {
                             return None;
                        }
                    }
                    
                    let ty = if intrin.name == "size_of" || intrin.name == "align_of" {
                        self.ctx.i32()
                    } else if intrin.name == "load" {
                        if type_args.len() == 1 {
                             type_args[0]
                        } else {
                             self.ctx.unit()
                        }
                    } else if intrin.name == "store" {
                         self.ctx.unit()
                    } else if intrin.name == "callsite_span" {
                        if type_args.len() == 1 {
                            type_args[0]
                        } else {
                            self.diagnostics
                                .push(Diagnostic::error("callsite_span expects 1 type arg", *sp));
                            self.ctx.unit()
                        }
                    } else if intrin.name == "get_field" || intrin.name == "set_field" {
                        self.ctx.unit() // temporary, will continue below
                    } else if intrin.name == "unreachable" {
                         self.ctx.never()
                    } else if intrin.name == "i32_to_f32" {
                        self.ctx.f32()
                    } else if intrin.name == "i32_to_u8" {
                        self.ctx.u8()
                    } else if intrin.name == "f32_to_i32" {
                        self.ctx.i32()
                    } else if intrin.name == "u8_to_i32" {
                        self.ctx.i32()
                    } else if intrin.name == "reinterpret_i32_f32" {
                        self.ctx.f32()
                    } else if intrin.name == "reinterpret_f32_i32" {
                        self.ctx.i32()
                    } else if intrin.name == "get_field" {
                        self.ctx.fresh_var(None)
                    } else if intrin.name == "set_field" {
                        self.ctx.unit()
                    } else {
                        self.diagnostics.push(Diagnostic::error("unknown intrinsic", *sp));
                        self.ctx.unit()
                    };

                    if intrin.name == "get_field" {
                            let obj = args[0].clone();
                            let idx = &args[1];
                            let res = match &idx.kind {
                                HirExprKind::LiteralI32(val) => {
                                    self.resolve_field_access(obj.ty, FieldIdx::Index(*val as usize), *sp)
                                }
                                HirExprKind::LiteralStr(sid) => {
                                    let name = self.string_table.get(*sid).unwrap().clone();
                                    self.resolve_field_access(obj.ty, FieldIdx::Name(name), *sp)
                                }
                                _ => None,
                            };
                            if let Some((f_ty, offset)) = res {
                                 // Unify our determined ty (fresh var) with the actual field type
                                 let _ = self.ctx.unify(ty, f_ty);

                                 // Lower to load(add(obj, offset))
                                 let addr_expr = if offset == 0 {
                                     obj
                                 } else {
                                     HirExpr {
                                         ty: self.ctx.i32(),
                                         kind: HirExprKind::Intrinsic {
                                             name: "add".to_string(),
                                             type_args: vec![self.ctx.i32()],
                                             args: vec![
                                                 obj,
                                                 HirExpr {
                                                     ty: self.ctx.i32(),
                                                     kind: HirExprKind::LiteralI32(offset as i32),
                                                     span: idx.span,
                                                 }
                                             ],
                                         },
                                         span: *sp,
                                     }
                                 };
                                 let hexpr = HirExpr {
                                     ty: f_ty,
                                     kind: HirExprKind::Intrinsic {
                                         name: "load".to_string(),
                                         type_args: vec![f_ty],
                                         args: vec![addr_expr],
                                     },
                                     span: *sp,
                                 };
                                 stack.push(StackEntry {
                                     ty: f_ty,
                                     expr: hexpr.clone(),
                                     type_args: Vec::new(),
                                     assign: None,
                            auto_call: true,
                                 });
                                 last_expr = Some(hexpr);
                                 continue;
                            }
                            // which pushes HirExprKind::Intrinsic and uses the fresh variable 'ty'.
                    } else if intrin.name == "set_field" {
                            let obj = args[0].clone();
                            let idx = &args[1];
                            let val = args[2].clone();
                            let res = match &idx.kind {
                                HirExprKind::LiteralI32(v) => {
                                    self.resolve_field_access(obj.ty, FieldIdx::Index(*v as usize), *sp)
                                }
                                HirExprKind::LiteralStr(sid) => {
                                    let name = self.string_table.get(*sid).unwrap().clone();
                                    self.resolve_field_access(obj.ty, FieldIdx::Name(name), *sp)
                                }
                                _ => None,
                            };
                            if let Some((f_ty, offset)) = res {
                                // Unify value type with field type
                                if let Err(_) = self.ctx.unify(val.ty, f_ty) {
                                     self.diagnostics.push(Diagnostic::error(format!("type mismatch in set_field: expected {}, found {}", self.ctx.type_to_string(f_ty), self.ctx.type_to_string(val.ty)), *sp));
                                }

                                // Lower to store(add(obj, offset), val)
                                let addr_expr = if offset == 0 {
                                    obj
                                } else {
                                    HirExpr {
                                        ty: self.ctx.i32(),
                                        kind: HirExprKind::Intrinsic {
                                            name: "add".to_string(),
                                            type_args: vec![self.ctx.i32()],
                                            args: vec![
                                                obj,
                                                HirExpr {
                                                    ty: self.ctx.i32(),
                                                    kind: HirExprKind::LiteralI32(offset as i32),
                                                    span: idx.span,
                                                }
                                            ],
                                        },
                                        span: *sp,
                                    }
                                };
                                let hexpr = HirExpr {
                                    ty: self.ctx.unit(),
                                    kind: HirExprKind::Intrinsic {
                                        name: "store".to_string(),
                                        type_args: vec![f_ty],
                                        args: vec![addr_expr, val],
                                    },
                                    span: *sp,
                                };
                                stack.push(StackEntry {
                                    ty: self.ctx.unit(),
                                    expr: hexpr.clone(),
                                    type_args: Vec::new(),
                                    assign: None,
                            auto_call: true,
                                });
                                last_expr = Some(hexpr);
                                continue;
                            }
                    }

                    
                    // Validate intrinsic argument types for known cast/bitcast intrinsics
                    if intrin.name == "i32_to_f32"
                        || intrin.name == "reinterpret_i32_f32"
                        || intrin.name == "i32_to_u8"
                    {
                        if args.len() != 1 {
                            self.diagnostics.push(Diagnostic::error(
                                "intrinsic expects 1 argument",
                                *sp,
                            ));
                        } else if let Err(_) = self.ctx.unify(args[0].ty, self.ctx.i32()) {
                            self.diagnostics.push(Diagnostic::error(
                                "intrinsic argument type mismatch (expected i32)",
                                *sp,
                            ));
                        }
                    } else if intrin.name == "f32_to_i32" || intrin.name == "reinterpret_f32_i32"
                    {
                        if args.len() != 1 {
                            self.diagnostics.push(Diagnostic::error(
                                "intrinsic expects 1 argument",
                                *sp,
                            ));
                        } else if let Err(_) = self.ctx.unify(args[0].ty, self.ctx.f32()) {
                            self.diagnostics.push(Diagnostic::error(
                                "intrinsic argument type mismatch (expected f32)",
                                *sp,
                            ));
                        }
                    } else if intrin.name == "u8_to_i32" {
                        if args.len() != 1 {
                            self.diagnostics.push(Diagnostic::error(
                                "intrinsic expects 1 argument",
                                *sp,
                            ));
                        } else if let Err(_) = self.ctx.unify(args[0].ty, self.ctx.u8()) {
                            self.diagnostics.push(Diagnostic::error(
                                "intrinsic argument type mismatch (expected u8)",
                                *sp,
                            ));
                        }
                    }

                    stack.push(StackEntry {
                        ty,
                        expr: HirExpr {
                            ty,
                            kind: HirExprKind::Intrinsic {
                                name: intrin.name.clone(),
                                type_args,
                                args,
                            },
                            span: *sp,
                        },
                        type_args: Vec::new(),
                        assign: None,
                            auto_call: true,
                    });
                    last_expr = Some(stack.last().unwrap().expr.clone());
                }
                PrefixItem::TypeAnnotation(ty_expr, _span) => {
                    let ty = type_from_expr(self.ctx, self.labels, ty_expr);
                    // record target type and current stack depth; do NOT treat as an expression
                    pending_ascription = Some((ty, stack.len()));
                }
                PrefixItem::Match(mexpr, _sp) => {
                    if let Some((hexpr, ty)) = self.check_match_expr(mexpr) {
                        stack.push(StackEntry {
                            ty,
                            expr: hexpr,
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: true,
                        });
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                }
                PrefixItem::Tuple(items, sp) => {
                    let mut elems = Vec::new();
                    let mut elem_tys = Vec::new();
                    for elem in items {
                        let mut elem_stack = Vec::new();
                        if let Some((hexpr, _)) = self.check_prefix(elem, 0, &mut elem_stack) {
                            elem_tys.push(hexpr.ty);
                            elems.push(hexpr);
                        } else {
                            return None;
                        }
                    }
                    let ty = self.ctx.tuple(elem_tys);
                    stack.push(StackEntry {
                        ty,
                        expr: HirExpr {
                            ty,
                            kind: HirExprKind::TupleConstruct { items: elems },
                            span: *sp,
                        },
                        type_args: Vec::new(),
                        assign: None,
                            auto_call: true,
                    });
                    last_expr = Some(stack.last().unwrap().expr.clone());
                }
                PrefixItem::Group(inner, _sp) => {
                    let mut group_stack = Vec::new();
                    if let Some((hexpr, _)) = self.check_prefix(inner, 0, &mut group_stack) {
                        stack.push(StackEntry {
                            ty: hexpr.ty,
                            expr: hexpr,
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: true,
                        });
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    } else {
                        return None;
                    }
                }
                PrefixItem::Block(b, sp) => {
                    // Treat blocks uniformly; parser now desugars `if:`/`if <cond>:`
                    // layout forms into ordinary prefix items, so the checker
                    // should not special-case `if` here.
                    let (blk, val_ty) = self.check_block(b, 0, true)?;
                    if let Some(ty) = val_ty {
                        stack.push(StackEntry {
                            ty,
                            expr: HirExpr {
                                ty,
                                kind: HirExprKind::Block(blk),
                                span: *sp,
                            },
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: false,
                        });
                        // defer applying ascription until the expression is complete
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    } else {
                        last_expr = Some(HirExpr {
                            ty: self.ctx.unit(),
                            kind: HirExprKind::Block(blk),
                            span: *sp,
                        });
                    }
                }
                PrefixItem::Pipe(sp) => {
                    if pipe_pending.is_some() {
                        self.diagnostics.push(Diagnostic::error(
                            "pipe already pending; consecutive |> not allowed",
                            *sp,
                        ));
                        continue;
                    }
                    if stack.len() == base_depth {
                        self.diagnostics
                            .push(Diagnostic::error("pipe requires a value on the stack", *sp));
                        continue;
                    }
                    pipe_pending = stack.pop();
                    last_expr = pipe_pending.as_ref().map(|se| se.expr.clone());
                }
            }

            if !matches!(item, PrefixItem::Pipe(_) | PrefixItem::TypeAnnotation(_, _)) {
                if let Some(val) = pipe_pending.take() {
                    // The last pushed element should be a callable (function type)
                    if let Some(top) = stack.last() {
                        if top.auto_call
                            && matches!(self.ctx.get(top.ty), TypeKind::Function { .. })
                        {
                            // pipe では「関数を積んだ直後に引数を注入」するため、
                            // 通常の末尾関数追跡だけでは open_calls に載らない。
                            let func_idx = stack.len() - 1;
                            if !open_calls.iter().any(|&i| i == func_idx) {
                                open_calls.push(func_idx);
                            }
                            stack.push(val);
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        } else {
                            self.diagnostics.push(Diagnostic::error(
                                "pipe target must be a callable expression",
                                expr.span,
                            ));
                            stack.push(val);
                        }
                    } else {
                        self.diagnostics
                            .push(Diagnostic::error("pipe target missing", expr.span));
                        stack.push(val);
                    }
                }
            }

            // Maintain open_calls stack
            open_calls.retain(|&i| i < stack.len());
            if let Some(top) = stack.last() {
                let idx = stack.len() - 1;
                let rty = self.ctx.resolve(top.ty);
                if top.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. }) {
                    if open_calls.last() != Some(&idx) {
                        open_calls.push(idx);
                    }
                }
            }

            // Try applying pending ascription before call reduction.
            // If the next token is `|>`, defer ascription until pipe injection
            // and subsequent call reduction are completed.
            if !next_is_pipe {
                try_apply_pending_ascription(self, stack, &mut pending_ascription);
            }

            let mut pending_base = pending_ascription.map(|(_, base)| base);
            let mut pipe_guard = false;
            if next_is_pipe {
                if let Some(assign_pos) = stack.iter().rposition(|e| e.assign.is_some()) {
                    let guard_pos = assign_pos + 1;
                    pending_base = Some(pending_base.map_or(guard_pos, |base| base.max(guard_pos)));
                    pipe_guard = true;
                }
            }
            if let Some(base_len) = pending_base {
                self.reduce_calls_guarded(stack, &mut open_calls, base_len, pending_ascription);
            } else {
                self.reduce_calls(stack, &mut open_calls, pending_ascription);
            }
                        // std::eprintln!("  Stack after reduce: {:?}", stack.iter().map(|e| self.ctx.type_to_string(e.ty)).collect::<Vec<_>>());

            // Try applying pending ascription after call reduction.
            if !next_is_pipe {
                try_apply_pending_ascription(self, stack, &mut pending_ascription);
            }

            if pending_base.is_some() && pending_ascription.is_none() && !pipe_guard {
                self.reduce_calls(stack, &mut open_calls, pending_ascription);
            }
        }

        if pipe_pending.is_some() {
            self.diagnostics
                .push(Diagnostic::error("pipe has no target", expr.span));
        }

        let leading_let = matches!(
            expr.items.first(),
            Some(PrefixItem::Symbol(Symbol::Let { .. }))
        );
        // Validate final stack depth. `let` is special-cased because its RHS
        // expression remains on stack until we lower to `HirExprKind::Let`.
        if leading_let {
            if stack.len() > base_depth + 2 {
                let extras = stack.len() - (base_depth + 2);
                for _ in 0..extras {
                    stack.pop();
                }
                dropped = true;
            }
        } else if stack.len() > base_depth + 1 {
            let extras = stack.len() - (base_depth + 1);
            for _ in 0..extras {
                stack.pop();
            }
            dropped = true;
        }

        let result_expr = if leading_let && stack.len() >= base_depth + 2 {
            stack[base_depth + 1].expr.clone()
        } else if stack.len() == base_depth + 1 {
            stack.last().unwrap().expr.clone()
        } else if let Some(ref e) = last_expr {
            e.clone()
        } else {
            HirExpr {
                ty: self.ctx.unit(),
                kind: HirExprKind::Unit,
                span: expr.span,
            }
        };

        // If this prefix began with a `let` symbol but reduction did not
        // produce a `Let` HIR node (e.g. for layout/colon forms), lower it
        // here: update the hoisted binding, mark it defined, and return a
        // `Let` expression so downstream codegen sees a stable binding.
        if let Some(PrefixItem::Symbol(Symbol::Let { name, mutable, .. })) = expr.items.first() {
            if !matches!(result_expr.kind, HirExprKind::Let { .. }) {
                // If RHS remains on stack (auto_call disabled for `let`), use it as
                // the binding value directly.
                let value_expr = if stack.len() >= base_depth + 2 {
                    stack[base_depth + 1].expr.clone()
                } else {
                    match &result_expr.kind {
                    HirExprKind::Var(n) if n == &name.name => {
                        if let Some(le) = last_expr.clone() { le } else { result_expr.clone() }
                    }
                    HirExprKind::Block(blk) => {
                        // Detect `if:` layout: block with exactly 3 lines and the
                        // original prefix contained an `if` symbol. In that case
                        // synthesize an `If` node from the three lines.
                        if blk.lines.len() == 3 && expr.items.iter().any(|it| matches!(it, PrefixItem::Symbol(Symbol::If(_)))) {
                            let cond = blk.lines[0].expr.clone();
                            let then_branch = blk.lines[1].expr.clone();
                            let else_branch = blk.lines[2].expr.clone();
                            HirExpr {
                                ty: then_branch.ty,
                                kind: HirExprKind::If {
                                    cond: Box::new(cond),
                                    then_branch: Box::new(then_branch),
                                    else_branch: Box::new(else_branch),
                                },
                                span: result_expr.span,
                            }
                        } else {
                            result_expr.clone()
                        }
                    }
                    _ => result_expr.clone(),
                }
                };

                if let Some(b) = self.env.lookup_mut(&name.name) {
                    b.defined = true;
                    b.ty = value_expr.ty;
                }
                let let_expr = HirExpr {
                    ty: self.ctx.unit(),
                    kind: HirExprKind::Let {
                        name: name.name.clone(),
                        mutable: *mutable,
                        value: Box::new(value_expr),
                    },
                    span: expr.span,
                };
                while stack.len() > base_depth {
                    let _ = stack.pop();
                }
                stack.push(StackEntry {
                    ty: self.ctx.unit(),
                    expr: let_expr.clone(),
                    type_args: Vec::new(),
                    assign: None,
                    auto_call: true,
                });
                return Some((let_expr, dropped));
            }
        }

        Some((result_expr, dropped))
    }

    fn apply_ascription(&mut self, stack: &mut [StackEntry], target: TypeId, span: Span) {
        if let Some(top) = stack.last_mut() {
            if let Err(_) = self.ctx.unify(top.ty, target) {
                self.diagnostics
                    .push(Diagnostic::error("type annotation mismatch", span));
            } else {
                top.ty = target;
                top.expr.ty = target;
            }
        }
    }

    fn reduce_calls(
        &mut self,
        stack: &mut Vec<StackEntry>,
        _open_calls: &mut Vec<usize>,
        expected: Option<(TypeId, usize)>,
    ) {
        let max_iterations = 1000; // Safety limit to prevent infinite loops
        let mut iterations = 0;
        loop {
            iterations += 1;
            if iterations > max_iterations {
                self.diagnostics.push(Diagnostic::error(
                    "reduce_calls exceeded maximum iterations (possible infinite loop)",
                    Span::dummy(),
                ));
                break;
            }
            dump!("reduce_calls: stack=[{}]", stack.iter().map(|e| match &e.expr.kind { HirExprKind::Var(n) => n.clone(), _ => "<expr>".to_string() }).collect::<Vec<_>>().join(","));

            let mut func_pos = match stack.iter().rposition(|e| {
                let rty = self.ctx.resolve(e.ty);
                e.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. })
            }) {
                Some(p) => p,
                None => break,
            };
            if let Some(outer) = self.find_outer_function_consumer(stack, func_pos, 0) {
                func_pos = outer;
            }

            let (inst_ty, fresh_args) = if !stack[func_pos].type_args.is_empty() {
                (stack[func_pos].ty, stack[func_pos].type_args.clone())
            } else {
                self.ctx.instantiate(stack[func_pos].ty)
            };
            let func_ty = self.ctx.get(inst_ty);
            let (params, result, effect) = match func_ty {
                TypeKind::Function {
                    params,
                    result,
                    effect,
                    ..
                } => (params, result, effect),
                _ => {
                    continue;
                }
            };
            let needed_args = self.user_visible_arity(&stack[func_pos].expr, params.len());
            let consume_unit_sugar = needed_args == 0
                && stack
                    .get(func_pos + 1)
                    .map(|e| matches!(e.expr.kind, HirExprKind::Unit))
                    .unwrap_or(false);
            let args_to_take = needed_args + if consume_unit_sugar { 1 } else { 0 };
            if stack.len() < func_pos + 1 + args_to_take {
                break;
            };
            let expected_ret = expected.and_then(|(target, base_len)| {
                let new_len = stack.len().saturating_sub(args_to_take);
                if new_len == base_len + 1 {
                    Some(target)
                } else {
                    None
                }
            });
            let mut args = Vec::new();
            for _ in 0..args_to_take {
                args.push(stack.remove(func_pos + 1));
            }

            let mut func_entry = stack.remove(func_pos);
            func_entry.ty = inst_ty;
            func_entry.expr.ty = inst_ty;
                if crate::log::is_verbose() {
                    std::eprintln!("    Reducing: {} at pos {} with {} args, assign={:?}", self.ctx.type_to_string(inst_ty), func_pos, params.len(), func_entry.assign);
                }
            let applied = self.apply_function(
                func_entry,
                params,
                result,
                effect,
                args,
                fresh_args,
                expected_ret,
            );

            if let Some(val) = applied {
                stack.insert(func_pos, val);
            } else {
                break;
            }
        }
    }

    fn resolve_dotted_field_symbol(&mut self, id: &Ident, forced_value: bool) -> Option<StackEntry> {
        if !id.name.contains('.') || id.name.contains("::") {
            return None;
        }

        let mut parts = id.name.split('.');
        let base_name = parts.next()?;
        let base_binding = self.env.lookup_value(base_name)?;
        if !matches!(base_binding.kind, BindingKind::Var) {
            return None;
        }

        let mut current = HirExpr {
            ty: base_binding.ty,
            kind: HirExprKind::Var(base_name.to_string()),
            span: id.span,
        };
        let mut current_ty = base_binding.ty;

        for field_name in parts {
            let (field_ty, offset) = self.resolve_field_access(
                current_ty,
                FieldIdx::Name(field_name.to_string()),
                id.span,
            )?;
            let addr_expr = if offset == 0 {
                current
            } else {
                HirExpr {
                    ty: self.ctx.i32(),
                    kind: HirExprKind::Intrinsic {
                        name: "add".to_string(),
                        type_args: vec![self.ctx.i32()],
                        args: vec![
                            current,
                            HirExpr {
                                ty: self.ctx.i32(),
                                kind: HirExprKind::LiteralI32(offset as i32),
                                span: id.span,
                            },
                        ],
                    },
                    span: id.span,
                }
            };
            current = HirExpr {
                ty: field_ty,
                kind: HirExprKind::Intrinsic {
                    name: "load".to_string(),
                    type_args: vec![field_ty],
                    args: vec![addr_expr],
                },
                span: id.span,
            };
            current_ty = field_ty;
        }

        Some(StackEntry {
            ty: current_ty,
            expr: current,
            type_args: Vec::new(),
            assign: None,
            auto_call: !forced_value,
        })
    }

    fn reduce_calls_guarded(
        &mut self,
        stack: &mut Vec<StackEntry>,
        _open_calls: &mut Vec<usize>,
        min_func_pos: usize,
        expected: Option<(TypeId, usize)>,
    ) {
        let max_iterations = 1000; // Safety limit to prevent infinite loops
        let mut iterations = 0;
        loop {
            iterations += 1;
            if iterations > max_iterations {
                self.diagnostics.push(Diagnostic::error(
                    "reduce_calls_guarded exceeded maximum iterations (possible infinite loop)",
                    Span::dummy(),
                ));
                break;
            }
            dump!("reduce_calls_guarded: stack=[{}]", stack.iter().map(|e| match &e.expr.kind { HirExprKind::Var(n) => n.clone(), _ => "<expr>".to_string() }).collect::<Vec<_>>().join(","));

            let mut func_pos: Option<usize> = None;
            for i in (min_func_pos..stack.len()).rev() {
                let rty = self.ctx.resolve(stack[i].ty);
                if stack[i].auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. }) {
                    func_pos = Some(i);
                    break;
                }
            }
            let Some(mut func_pos) = func_pos else {
                break;
            };
            if let Some(outer) = self.find_outer_function_consumer(stack, func_pos, min_func_pos) {
                func_pos = outer;
            }

            let (inst_ty, fresh_args) = if !stack[func_pos].type_args.is_empty() {
                (stack[func_pos].ty, stack[func_pos].type_args.clone())
            } else {
                self.ctx.instantiate(stack[func_pos].ty)
            };
            let func_ty = self.ctx.get(inst_ty);
            let (params, result, effect) = match func_ty {
                TypeKind::Function {
                    params,
                    result,
                    effect,
                    ..
                } => (params, result, effect),
                _ => {
                    continue;
                }
            };
            let needed_args = self.user_visible_arity(&stack[func_pos].expr, params.len());
            let consume_unit_sugar = needed_args == 0
                && stack
                    .get(func_pos + 1)
                    .map(|e| matches!(e.expr.kind, HirExprKind::Unit))
                    .unwrap_or(false);
            let args_to_take = needed_args + if consume_unit_sugar { 1 } else { 0 };
            if stack.len() < func_pos + 1 + args_to_take {
                break;
            }
            let expected_ret = expected.and_then(|(target, base_len)| {
                let new_len = stack.len().saturating_sub(args_to_take);
                if new_len == base_len + 1 {
                    Some(target)
                } else {
                    None
                }
            });
            let mut args = Vec::new();
            for _ in 0..args_to_take {
                args.push(stack.remove(func_pos + 1));
            }

            let mut func_entry = stack.remove(func_pos);
            func_entry.ty = inst_ty;
            func_entry.expr.ty = inst_ty;
                if crate::log::is_verbose() {
                    std::eprintln!("    Reducing (guarded): {} at pos {} with {} args, assign={:?}", self.ctx.type_to_string(inst_ty), func_pos, params.len(), func_entry.assign);
                }
            let applied = self.apply_function(
                func_entry,
                params,
                result,
                effect,
                args,
                fresh_args,
                expected_ret,
            );

            if let Some(val) = applied {
                stack.insert(func_pos, val);
            } else {
                break;
            }
        }
    }

    fn check_match_expr(&mut self, m: &MatchExpr) -> Option<(HirExpr, TypeId)> {
        // evaluate scrutinee
        let mut tmp_stack = Vec::new();
        if let Some((scrut_expr, _)) = self.check_prefix(&m.scrutinee, 0, &mut tmp_stack) {
            let scrut_ty = scrut_expr.ty;
            let resolved_ty = self.ctx.resolve(scrut_ty);
            let variants = match self.ctx.get(resolved_ty) {
                TypeKind::Enum { variants, .. } => Some(variants.clone()),
                TypeKind::Bool => Some(alloc::vec![
                    EnumVariantInfo { name: "true".to_string(), payload: None },
                    EnumVariantInfo { name: "false".to_string(), payload: None },
                ]),
                TypeKind::Apply { base, args } => {
                    let base_ty = self.ctx.resolve(base);
                    match self.ctx.get(base_ty) {
                        TypeKind::Enum { type_params, variants, .. } => {
                            if type_params.len() == args.len() {
                                let mut mapping = alloc::collections::BTreeMap::new();
                                for (tp, arg) in type_params.iter().zip(args.iter()) {
                                    mapping.insert(*tp, *arg);
                                }
                                let mut new_vars = Vec::new();
                                for v in variants {
                                    new_vars.push(EnumVariantInfo {
                                        name: v.name.clone(),
                                        payload: v.payload.map(|p| self.ctx.substitute(p, &mapping)),
                                    });
                                }
                                Some(new_vars)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                }
                _ => None,
            };
            if variants.is_none() {
                self.diagnostics
                    .push(Diagnostic::error("match scrutinee must be an enum", m.span));
                return None;
            }
            let variants = variants.unwrap();
            let mut seen = alloc::collections::BTreeSet::new();
            let mut arms_hir = Vec::new();
            let mut result_ty: Option<TypeId> = None;
            for arm in &m.arms {
                let arm_var_name = if let Some(pos) = arm.variant.name.find("::") {
                    &arm.variant.name[pos + 2..]
                } else {
                    &arm.variant.name
                };
                if !seen.insert(arm_var_name.to_string()) {
                    self.diagnostics
                        .push(Diagnostic::error("duplicate match arm", arm.variant.span));
                    continue;
                }
                let var_info = variants.iter().find(|v| v.name == arm_var_name);
                if var_info.is_none() {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("unknown enum variant '{}' in match", arm.variant.name),
                    arm.variant.span,
                ));
                    continue;
                }
                let var_info = var_info.unwrap();
                self.env.push_scope();
                if let Some(bind) = &arm.bind {
                    if let Some(pty) = var_info.payload {
                        emit_shadow_warning(
                            &mut self.diagnostics,
                            self.env,
                            &bind.name,
                            bind.span,
                            "match binding",
                        );
                        let _ = self.env.insert_local(Binding {
                            name: bind.name.clone(),
                            ty: pty,
                            mutable: false,
                            no_shadow: false,
                            defined: true,
                            moved: false,
                            span: bind.span,
                            kind: BindingKind::Var,
                        });
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            "variant has no payload to bind",
                            bind.span,
                        ));
                    }
                }
                let (blk, val_ty) = self.check_block(&arm.body, 0, false)?;
                self.env.pop_scope();
                let body_ty = val_ty.unwrap_or(self.ctx.unit());
                if let Some(t) = result_ty {
                    if let Err(_) = self.ctx.unify(t, body_ty) {
                        self.diagnostics.push(Diagnostic::error(
                            alloc::format!(
                                "match arms have incompatible types: {} and {}",
                                self.ctx.type_to_string(t),
                                self.ctx.type_to_string(body_ty)
                            ),
                            arm.span,
                        ));
                    }
                } else {
                    result_ty = Some(body_ty);
                }
                arms_hir.push(HirMatchArm {
                    variant: arm.variant.name.clone(),
                    bind_local: arm.bind.as_ref().map(|b| b.name.clone()),
                    body: HirExpr {
                        ty: body_ty,
                        kind: HirExprKind::Block(blk),
                        span: arm.span,
                    },
                });
            }
            // exhaustiveness
            for v in variants {
                if !seen.contains(&v.name) {
                    self.diagnostics
                        .push(Diagnostic::error("non-exhaustive match", m.span));
                    break;
                }
            }
            let rty = result_ty.unwrap_or(self.ctx.unit());
            return Some((
                HirExpr {
                    ty: rty,
                    kind: HirExprKind::Match {
                        scrutinee: Box::new(scrut_expr),
                        arms: arms_hir,
                    },
                    span: m.span,
                },
                rty,
            ));
        }
        None
    }

    fn split_if_then_else_block_ast(b: &Block) -> Option<(Block, Block)> {
        // Find top-level `else` marker line inside the block
        let mut else_idx: Option<usize> = None;
        for (i, stmt) in b.items.iter().enumerate() {
            if let Stmt::Expr(e) = stmt {
                if let Some(PrefixItem::Symbol(Symbol::Ident(id, _, _))) = e.items.first() {
                    if id.name == "else" {
                        else_idx = Some(i);
                        break;
                    }
                }
            }
        }
        let else_idx = else_idx?;

        if else_idx + 1 != b.items.len() {
            return None;
        }

        let mut then_items = b.items[..else_idx].to_vec();
        let else_stmt = b.items[else_idx].clone();

        let then_block = if then_items.len() == 1 {
            if let Stmt::Expr(e) = then_items.remove(0) {
                if let Some(PrefixItem::Symbol(Symbol::Ident(id, _, _))) = e.items.first() {
                    if id.name == "then" {
                        // drop leading marker and convert remaining items into block
                        let mut items = e.items;
                        if !items.is_empty() {
                            items.remove(0);
                        }
                        if items.len() == 1 {
                            if let PrefixItem::Block(bb, _) = items.remove(0) {
                                bb
                            } else {
                                Block {
                                    items: alloc::vec![Stmt::Expr(PrefixExpr {
                                        items,
                                        trailing_semis: 0,
                                        trailing_semi_span: None,
                                        span: e.span
                                    })],
                                    span: e.span,
                                }
                            }
                        } else {
                            Block {
                                items: alloc::vec![Stmt::Expr(PrefixExpr {
                                    items,
                                    trailing_semis: 0,
                                    trailing_semi_span: None,
                                    span: e.span
                                })],
                                span: e.span,
                            }
                        }
                    } else {
                        Block {
                            items: alloc::vec![Stmt::Expr(e)],
                            span: b.span,
                        }
                    }
                } else {
                    Block {
                        items: alloc::vec![Stmt::Expr(e)],
                        span: b.span,
                    }
                }
            } else {
                Block {
                    items: then_items,
                    span: b.span,
                }
            }
        } else {
            Block {
                items: then_items,
                span: b.span,
            }
        };

        let else_block = match else_stmt {
            Stmt::Expr(e) => {
                let mut items = e.items;
                if !items.is_empty() {
                    items.remove(0);
                }
                if items.len() == 1 {
                    if let PrefixItem::Block(bb, _) = items.remove(0) {
                        bb
                    } else {
                        Block {
                            items: alloc::vec![Stmt::Expr(PrefixExpr {
                                items,
                                trailing_semis: 0,
                                trailing_semi_span: None,
                                span: e.span
                            })],
                            span: e.span,
                        }
                    }
                } else {
                    Block {
                        items: alloc::vec![Stmt::Expr(PrefixExpr {
                            items,
                            trailing_semis: 0,
                            trailing_semi_span: None,
                            span: e.span
                        })],
                        span: e.span,
                    }
                }
            }
            _ => return None,
        };

        Some((then_block, else_block))
    }

    fn apply_function(
        &mut self,
        func: StackEntry,
        params: Vec<TypeId>,
        result: TypeId,
        effect: Effect,
        mut args: Vec<StackEntry>,
        type_args: Vec<TypeId>,
        expected_ret: Option<TypeId>,
    ) -> Option<StackEntry> {
        if params.is_empty()
            && args.len() == 1
            && matches!(args[0].expr.kind, HirExprKind::Unit)
        {
            args.clear();
        }

        if matches!(self.current_effect, Effect::Pure)
            && matches!(effect, Effect::Impure)
        {
            self.diagnostics.push(Diagnostic::error(
                "pure context cannot call impure function",
                func.expr.span,
            ));
            return None;
        }

        // Assignment operators
        if let Some(assign) = func.assign {
            if args.len() != 1 {
                self.diagnostics.push(Diagnostic::error(
                    "assignment expects one argument",
                    func.expr.span,
                ));
                return None;
            }
            // Handle field store first since it doesn't need variable lookup
            if let AssignKind::Store(addr) = assign {
                if !params.is_empty() {
                    if let Err(_) = self.ctx.unify(params[0], args[0].ty) {
                        self.diagnostics.push(Diagnostic::error(
                            "type mismatch in field assignment",
                            func.expr.span,
                        ));
                    }
                }
                return Some(StackEntry {
                    ty: self.ctx.unit(),
                    expr: HirExpr {
                        ty: self.ctx.unit(),
                        kind: HirExprKind::Intrinsic {
                            name: "store".to_string(),
                            type_args: vec![args[0].ty],
                            args: vec![addr, args[0].expr.clone()],
                        },
                        span: func.expr.span,
                    },
                    type_args: Vec::new(),
                    assign: None,
                            auto_call: true,
                });
            } else if matches!(assign, AssignKind::AddrOf) {
                if args.len() != 1 { return None; }
                if crate::log::is_verbose() {
                    std::eprintln!("apply_function: Reducing AddrOf, inner={:?}", args[0].expr.kind);
                }
                let inner_ty = args[0].ty;
                let res_ty = self.ctx.reference(inner_ty, false);
                return Some(StackEntry {
                    ty: res_ty,
                    expr: HirExpr {
                        ty: res_ty,
                        kind: HirExprKind::AddrOf(Box::new(args[0].expr.clone())),
                        span: func.expr.span,
                    },
                    type_args: Vec::new(),
                    assign: None,
                            auto_call: true,
                });
            } else if matches!(assign, AssignKind::Deref) {
                if args.len() != 1 { return None; }
                let arg_ty = self.ctx.resolve(args[0].ty);
                let inner_ty = match self.ctx.get(arg_ty) {
                    TypeKind::Reference(inner, _) => inner,
                    _ => {
                        self.diagnostics.push(Diagnostic::error(
                            format!("cannot dereference non-reference type: {}", self.ctx.type_to_string(arg_ty)),
                            args[0].expr.span,
                        ));
                        self.ctx.never()
                    }
                };
                return Some(StackEntry {
                    ty: inner_ty,
                    expr: HirExpr {
                        ty: inner_ty,
                        kind: HirExprKind::Deref(Box::new(args[0].expr.clone())),
                        span: func.expr.span,
                    },
                    type_args: Vec::new(),
                    assign: None,
                            auto_call: true,
                });
            }

            let name = match &func.expr.kind {
                HirExprKind::Var(n) => n.clone(),
                _ => "_".to_string(),
            };
            // For assignments we must find hoisted (possibly undefined)
            // bindings as well, so use a mutable lookup that returns
            // bindings regardless of `defined` state.
            if let Some(b) = self.env.lookup_mut(&name) {
                let b_ty = b.ty;
                let b_mut = b.mutable;
                let b_defined = b.defined;
                if let Err(_) = self.ctx.unify(b_ty, args[0].ty) {
                    self.diagnostics.push(Diagnostic::error(
                        "type mismatch in assignment",
                        func.expr.span,
                    ));
                }
                match assign {
                    AssignKind::Let => {
                        b.defined = true;
                        b.ty = b_ty;
                        dump!("typecheck: marking binding defined {}", name);
                        return Some(StackEntry {
                            ty: self.ctx.unit(),
                            expr: HirExpr {
                                ty: self.ctx.unit(),
                                kind: HirExprKind::Let {
                                    name: name.clone(),
                                    mutable: b_mut,
                                    value: Box::new(args[0].expr.clone()),
                                },
                                span: func.expr.span,
                            },
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: true,
                        });
                    }
                    AssignKind::Set => {
                        if !b_defined {
                            self.diagnostics.push(Diagnostic::error(
                                "cannot set undefined variable",
                                func.expr.span,
                            ));
                        }
                        if !b_mut {
                            self.diagnostics
                                .push(Diagnostic::error("variable is not mutable", func.expr.span));
                        }
                        return Some(StackEntry {
                            ty: self.ctx.unit(),
                            expr: HirExpr {
                                ty: self.ctx.unit(),
                                kind: HirExprKind::Set {
                                    name: name.clone(),
                                    value: Box::new(args[0].expr.clone()),
                                },
                                span: func.expr.span,
                            },
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: true,
                        });
                    }
                    _ => unreachable!(),
                }
            } else {
                self.diagnostics.push(Diagnostic::error(
                    format!("undefined variable for assignment: {}", name),
                    func.expr.span,
                ));
                return None;
            }
        }

        // Special-cased symbols (if / while)
        match &func.expr.kind {
            HirExprKind::Var(name) if name == "if" => {
                if args.len() != 3 {
                    self.diagnostics.push(Diagnostic::error(
                        "if expects three arguments",
                        func.expr.span,
                    ));
                    return None;
                }
                if self.ctx.unify(args[0].ty, self.ctx.bool()).is_err() {
                    self.diagnostics.push(Diagnostic::error(
                        "if condition must be bool",
                        args[0].expr.span,
                    ));
                }
                let branch_ty = self.ctx.unify(args[1].ty, args[2].ty).unwrap_or(args[1].ty);
                return Some(StackEntry {
                    ty: branch_ty,
                    expr: HirExpr {
                        ty: branch_ty,
                        kind: HirExprKind::If {
                            cond: Box::new(args[0].expr.clone()),
                            then_branch: Box::new(args[1].expr.clone()),
                            else_branch: Box::new(args[2].expr.clone()),
                        },
                        span: func.expr.span,
                    },
                    type_args: Vec::new(),
                    assign: None,
                            auto_call: true,
                });
            }
            HirExprKind::Var(name) if name == "while" => {
                if args.len() != 2 {
                    self.diagnostics.push(Diagnostic::error(
                        "while expects two arguments",
                        func.expr.span,
                    ));
                    return None;
                }
                if self.ctx.unify(args[0].ty, self.ctx.bool()).is_err() {
                    self.diagnostics.push(Diagnostic::error(
                        "while condition must be bool",
                        args[0].expr.span,
                    ));
                }
                if self.ctx.unify(args[1].ty, self.ctx.unit()).is_err() {
                    self.diagnostics.push(Diagnostic::error(
                        "while body must be unit",
                        args[1].expr.span,
                    ));
                }
                return Some(StackEntry {
                    ty: self.ctx.unit(),
                    expr: HirExpr {
                        ty: self.ctx.unit(),
                        kind: HirExprKind::While {
                            cond: Box::new(args[0].expr.clone()),
                            body: Box::new(args[1].expr.clone()),
                        },
                        span: func.expr.span,
                    },
                    type_args: Vec::new(),
                    assign: None,
                            auto_call: true,
                });
            }
            HirExprKind::Var(name) if name == "let" || name == "set" => {
                // handled elsewhere
            }
            _ => {}
        }
        
        // Specialized inlining for get/put
        if let HirExprKind::Var(name) = &func.expr.kind {
            if (name == "get" || name == "put") && args.len() >= 2 {
                let obj = args[0].expr.clone();
                let idx = &args[1].expr;
                let field_idx = match &idx.kind {
                    HirExprKind::LiteralI32(v) => Some(FieldIdx::Index(*v as usize)),
                    HirExprKind::LiteralStr(sid) => {
                        let name = self.string_table.get(*sid).unwrap().clone();
                        Some(FieldIdx::Name(name))
                    }
                    _ => None,
                };
                if let Some(f_idx) = field_idx {
                    if let Some((f_ty, offset)) = self.resolve_field_access(obj.ty, f_idx, func.expr.span) {
                        if name == "get" && args.len() == 2 {
                            let addr_expr = if offset == 0 {
                                obj
                            } else {
                                HirExpr {
                                    ty: self.ctx.i32(),
                                    kind: HirExprKind::Intrinsic {
                                        name: "add".to_string(),
                                        type_args: vec![self.ctx.i32()],
                                        args: vec![
                                            obj,
                                            HirExpr {
                                                ty: self.ctx.i32(),
                                                kind: HirExprKind::LiteralI32(offset as i32),
                                                span: idx.span,
                                            }
                                        ],
                                    },
                                    span: func.expr.span,
                                }
                            };
                            return Some(StackEntry {
                                ty: f_ty,
                                expr: HirExpr {
                                    ty: f_ty,
                                    kind: HirExprKind::Intrinsic {
                                        name: "load".to_string(),
                                        type_args: vec![f_ty],
                                        args: vec![addr_expr],
                                    },
                                    span: func.expr.span,
                                },
                                type_args: Vec::new(),
                                assign: None,
                            auto_call: true,
                            });
                        } else if name == "put" && args.len() == 3 {
                            let val = args[2].expr.clone();
                            let _ = self.ctx.unify(val.ty, f_ty);
                            let addr_expr = if offset == 0 {
                                obj
                            } else {
                                HirExpr {
                                    ty: self.ctx.i32(),
                                    kind: HirExprKind::Intrinsic {
                                        name: "add".to_string(),
                                        type_args: vec![self.ctx.i32()],
                                        args: vec![
                                            obj,
                                            HirExpr {
                                                ty: self.ctx.i32(),
                                                kind: HirExprKind::LiteralI32(offset as i32),
                                                span: idx.span,
                                            }
                                        ],
                                    },
                                    span: func.expr.span,
                                }
                            };
                            return Some(StackEntry {
                                ty: self.ctx.unit(),
                                expr: HirExpr {
                                    ty: self.ctx.unit(),
                                    kind: HirExprKind::Intrinsic {
                                        name: "store".to_string(),
                                        type_args: vec![f_ty],
                                        args: vec![addr_expr, val],
                                    },
                                    span: func.expr.span,
                                },
                                type_args: Vec::new(),
                                assign: None,
                            auto_call: true,
                            });
                        }
                    }
                }
            }
        }

        // General call or let/set
        if let HirExprKind::Var(name) = &func.expr.kind {
            let bindings = self.env.lookup_all_callables(name);
            if !bindings.is_empty() {
                {
                    let explicit_type_args = type_args.clone();
                    let use_expected = expected_ret.is_some() && bindings.len() > 1;
                    let mut candidates: Vec<&Binding> = Vec::new();
                    let mut mismatch_count = false;
                    for binding in &bindings {
                        let capture_len = match &binding.kind {
                            BindingKind::Func { captures, .. } => captures.len(),
                            _ => 0,
                        };
                        let mut tmp_ctx = self.ctx.clone();
                        let inst_ty = if !explicit_type_args.is_empty() {
                            let func_data = if let TypeKind::Function {
                                type_params,
                                params,
                                result,
                                effect,
                            } = tmp_ctx.get(binding.ty)
                            {
                                Some((type_params, params, result, effect))
                            } else {
                                None
                            };
                            let Some((type_params, params, result, effect)) = func_data else {
                                continue;
                            };
                            if type_params.len() != explicit_type_args.len() {
                                mismatch_count = true;
                                continue;
                            }
                            let mut mapping = BTreeMap::new();
                            for (p, a) in type_params.iter().zip(explicit_type_args.iter()) {
                                mapping.insert(tmp_ctx.resolve_id(*p), tmp_ctx.resolve_id(*a));
                            }
                            let substituted_params = params
                                .iter()
                                .map(|p| tmp_ctx.substitute(*p, &mapping))
                                .collect::<Vec<_>>();
                            let substituted_result = tmp_ctx.substitute(result, &mapping);
                            tmp_ctx.function(Vec::new(), substituted_params, substituted_result, effect)
                        } else {
                            let (inst_ty, _args) = tmp_ctx.instantiate(binding.ty);
                            inst_ty
                        };

                        let func_ty = tmp_ctx.get(inst_ty);
                        let (c_params, c_result, _c_effect) = match func_ty {
                            TypeKind::Function {
                                params,
                                result,
                                effect,
                                ..
                            } => (params, result, effect),
                            _ => continue,
                        };
                        if c_params.len() < capture_len {
                            continue;
                        }
                        let user_params = &c_params[capture_len..];
                        if user_params.len() != args.len() {
                            continue;
                        }
                        let mut ok = true;
                        for (arg, pty) in args.iter().zip(user_params.iter()) {
                            if tmp_ctx.unify(arg.ty, *pty).is_err() {
                                ok = false;
                                break;
                            }
                        }
                        if ok && use_expected {
                            if let Some(expected) = expected_ret {
                                if tmp_ctx.unify(c_result, expected).is_err() {
                                    ok = false;
                                }
                            }
                        }
                        if ok {
                            candidates.push(binding);
                        }
                    }

                    if candidates.is_empty() {
                        if mismatch_count {
                            self.diagnostics.push(Diagnostic::error(
                                "type arguments do not match any overload",
                                func.expr.span,
                            ));
                        } else {
                            self.diagnostics.push(Diagnostic::error(
                                "no matching overload found",
                                func.expr.span,
                            ));
                        }
                        return None;
                    }
                    if candidates.len() > 1 {
                        self.diagnostics.push(Diagnostic::error(
                            "ambiguous overload",
                            func.expr.span,
                        ));
                        return None;
                    }

                    let binding = candidates[0];
                    let (inst_ty, mut resolved_args) = if !explicit_type_args.is_empty() {
                        let func_data = if let TypeKind::Function {
                            type_params,
                            params,
                            result,
                            effect,
                        } = self.ctx.get(binding.ty)
                        {
                            Some((type_params.clone(), params.clone(), result, effect))
                        } else {
                            None
                        };
                        let Some((type_params, params, result, effect)) = func_data else {
                            return None;
                        };
                        if type_params.len() != explicit_type_args.len() {
                            self.diagnostics.push(Diagnostic::error(
                                "type arguments do not match overload",
                                func.expr.span,
                            ));
                            return None;
                        }
                        let mut mapping = BTreeMap::new();
                        for (p, a) in type_params.iter().zip(explicit_type_args.iter()) {
                            mapping.insert(self.ctx.resolve_id(*p), self.ctx.resolve_id(*a));
                        }
                        let substituted_params = params
                            .iter()
                            .map(|p| self.ctx.substitute(*p, &mapping))
                            .collect::<Vec<_>>();
                        let substituted_result = self.ctx.substitute(result, &mapping);
                        (
                            self.ctx
                                .function(Vec::new(), substituted_params, substituted_result, effect),
                            explicit_type_args.clone(),
                        )
                    } else {
                        self.ctx.instantiate(binding.ty)
                    };

                    let (c_params, c_result, c_effect) = match self.ctx.get(inst_ty) {
                        TypeKind::Function {
                            params,
                            result,
                            effect,
                            ..
                        } => (params, result, effect),
                        _ => return None,
                    };
                    let captures = match &binding.kind {
                        BindingKind::Func { captures, .. } => captures.clone(),
                        _ => Vec::new(),
                    };
                    if c_params.len() < captures.len() {
                        self.diagnostics.push(Diagnostic::error(
                            "internal error: capture arity mismatch",
                            func.expr.span,
                        ));
                        return None;
                    }
                    let user_params = &c_params[captures.len()..];
                    if user_params.len() != args.len() {
                        self.diagnostics.push(Diagnostic::error(
                            "argument count mismatch",
                            func.expr.span,
                        ));
                        return None;
                    }
                    for (arg, param_ty) in args.iter().zip(user_params.iter()) {
                        if self.ctx.unify(arg.ty, *param_ty).is_err() {
                            self.diagnostics.push(Diagnostic::error(
                                "argument type mismatch",
                                arg.expr.span,
                            ));
                        }
                    }
                    if matches!(self.current_effect, Effect::Pure) && matches!(c_effect, Effect::Impure)
                    {
                        self.diagnostics.push(Diagnostic::error(
                            "pure context cannot call impure function",
                            func.expr.span,
                        ));
                        return None;
                    }

                    let raw_type_args = resolved_args.clone();
                    resolved_args = resolved_args
                        .into_iter()
                        .map(|t| self.ctx.resolve_id(t))
                        .collect();

                    if let BindingKind::Func {
                        type_param_bounds,
                        ..
                    } = &binding.kind
                    {
                        if !type_param_bounds.is_empty()
                            && type_param_bounds.len() == resolved_args.len()
                        {
                            for (bounds, (raw_arg, resolved_arg)) in type_param_bounds
                                .iter()
                                .zip(raw_type_args.iter().zip(resolved_args.iter()))
                            {
                                for b in bounds {
                                    if !self.trait_bound_satisfied(b, *raw_arg)
                                        && !self.trait_bound_satisfied(b, *resolved_arg)
                                    {
                                        self.diagnostics.push(Diagnostic::error(
                                            format!("type does not satisfy trait bound '{}'", b),
                                            func.expr.span,
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    // Enum constructors
                    if let Some((enm, var)) = parse_variant_name(name) {
                        if let Some(info) = self.enums.get(enm) {
                            if let Some(_vinfo) = info.variants.iter().find(|v| v.name == var) {
                                if c_params.len() == 1 && args.len() != 1 {
                                    self.diagnostics.push(Diagnostic::error(
                                        "constructor expects one argument",
                                        func.expr.span,
                                    ));
                                    return None;
                                }
                                if c_params.is_empty() && !args.is_empty() {
                                    self.diagnostics.push(Diagnostic::error(
                                        "constructor takes no arguments",
                                        func.expr.span,
                                    ));
                                    return None;
                                }
                                let payload_expr = if c_params.len() == 1 {
                                    if let Some(a0) = args.first() {
                                        Some(Box::new(a0.expr.clone()))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                let applied_ty = if resolved_args.is_empty() {
                                    info.ty
                                } else {
                                    self.ctx.apply(info.ty, resolved_args.clone())
                                };
                                return Some(StackEntry {
                                    ty: applied_ty,
                                    expr: HirExpr {
                                        ty: applied_ty,
                                        kind: HirExprKind::EnumConstruct {
                                            name: enm.to_string(),
                                            variant: var.to_string(),
                                            type_args: resolved_args.clone(),
                                            payload: payload_expr,
                                        },
                                        span: func.expr.span,
                                    },
                                    type_args: Vec::new(),
                                    assign: None,
                            auto_call: true,
                                });
                            }
                        }
                    }
                    if let Some(s) = self.structs.get(name) {
                        if args.len() != c_params.len() {
                            self.diagnostics.push(Diagnostic::error(
                                "struct constructor arity mismatch",
                                func.expr.span,
                            ));
                            return None;
                        }
                        let applied_ty = if resolved_args.is_empty() {
                            s.ty
                        } else {
                            self.ctx.apply(s.ty, resolved_args.clone())
                        };
                        return Some(StackEntry {
                            ty: applied_ty,
                            expr: HirExpr {
                                ty: applied_ty,
                                kind: HirExprKind::StructConstruct {
                                    name: name.clone(),
                                    type_args: resolved_args.clone(),
                                    fields: args.into_iter().map(|a| a.expr).collect(),
                                },
                                span: func.expr.span,
                            },
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: true,
                        });
                    }

                    let (callee_name, builtin) = match &binding.kind {
                        BindingKind::Func { symbol, builtin, .. } => (symbol.clone(), builtin),
                        _ => (name.clone(), &None),
                    };
                    let mut final_args: Vec<HirExpr> = Vec::new();
                    for (cap_name, cap_ty) in captures.iter() {
                        final_args.push(HirExpr {
                            ty: *cap_ty,
                            kind: HirExprKind::Var(cap_name.clone()),
                            span: func.expr.span,
                        });
                    }
                    final_args.extend(args.iter().map(|a| a.expr.clone()));
                    let mut trait_callee: Option<FuncRef> = None;
                    if let Some((trait_name, method_name)) = parse_variant_name(name) {
                        if let Some(trait_info) = self.traits.get(trait_name) {
                            if trait_info.methods.get(method_name).is_some() {
                                if let Some(first) = args.first() {
                                    trait_callee = Some(FuncRef::Trait {
                                        trait_name: trait_name.to_string(),
                                        method: method_name.to_string(),
                                        self_ty: self.ctx.resolve_id(first.ty),
                                    });
                                }
                            }
                        }
                    }
                    let callee = if builtin.is_some() {
                        FuncRef::Builtin(callee_name.clone())
                    } else if let Some(tc) = trait_callee {
                        tc
                    } else {
                        if !resolved_args.is_empty()
                            && resolved_args
                                .iter()
                                .all(|t| !type_contains_unbound_var(self.ctx, *t))
                        {
                            self.instantiations
                                .entry(callee_name.clone())
                                .or_insert_with(Vec::new)
                                .push(resolved_args.clone());
                        }
                        FuncRef::User(callee_name.clone(), resolved_args.clone())
                    };
                    let resolved_result = self.ctx.resolve_id(c_result);
                    return Some(StackEntry {
                        ty: resolved_result,
                        expr: HirExpr {
                            ty: resolved_result,
                            kind: HirExprKind::Call {
                                callee,
                                args: final_args,
                            },
                            span: func.expr.span,
                        },
                        type_args: Vec::new(),
                        assign: None,
                            auto_call: true,
                    });
                }
            }
        }


        if let HirExprKind::Var(name) = &func.expr.kind {
            if self.env.lookup_all(name).is_empty() {
                if let Some((trait_name, method_name)) = parse_variant_name(name) {
                    if let Some(trait_info) = self.traits.get(trait_name) {
                        if trait_info.methods.get(method_name).is_some() {
                            if args.is_empty() {
                                self.diagnostics.push(Diagnostic::error(
                                    "trait method call requires receiver argument",
                                    func.expr.span,
                                ));
                                return None;
                            }
                            let self_ty = self.ctx.resolve_id(args[0].ty);
                            if !self.trait_bound_satisfied(trait_name, self_ty) {
                                self.diagnostics.push(Diagnostic::error(
                                    format!(
                                        "type does not satisfy trait bound '{}'",
                                        trait_name
                                    ),
                                    func.expr.span,
                                ));
                                return None;
                            }
                            if matches!(self.current_effect, Effect::Pure)
                                && matches!(effect, Effect::Impure)
                            {
                                self.diagnostics.push(Diagnostic::error(
                                    "pure context cannot call impure function",
                                    func.expr.span,
                                ));
                                return None;
                            }
                            let resolved_result = self.ctx.resolve_id(result);
                            return Some(StackEntry {
                                ty: resolved_result,
                                expr: HirExpr {
                                    ty: resolved_result,
                                    kind: HirExprKind::Call {
                                        callee: FuncRef::Trait {
                                            trait_name: trait_name.to_string(),
                                            method: method_name.to_string(),
                                            self_ty,
                                        },
                                        args: args.into_iter().map(|a| a.expr).collect(),
                                    },
                                    span: func.expr.span,
                                },
                                type_args: Vec::new(),
                                assign: None,
                                auto_call: true,
                            });
                        }
                    }
                }
            } else if self.env.lookup_value(name).is_some() {
                if !matches!(self.ctx.get(func.ty), TypeKind::Function { .. }) {
                    self.diagnostics.push(Diagnostic::error(
                        "variable is not callable",
                        func.expr.span,
                    ));
                    return None;
                }
            }
        }

        // Fallback: function value call (`call_indirect` in wasm backend)
        let resolved_params: Vec<TypeId> = args
            .iter()
            .map(|a| self.ctx.resolve_id(a.ty))
            .collect();
        let mut resolved_result = self.ctx.resolve_id(result);
        if let Some(expected) = expected_ret {
            if self.ctx.unify(resolved_result, expected).is_ok() {
                resolved_result = self.ctx.resolve_id(expected);
            }
        }
        Some(StackEntry {
            ty: resolved_result,
            expr: HirExpr {
                ty: resolved_result,
                kind: HirExprKind::CallIndirect {
                    callee: Box::new(func.expr.clone()),
                    params: resolved_params,
                    result: resolved_result,
                    args: args.into_iter().map(|a| a.expr).collect(),
                },
                span: func.expr.span,
            },
            type_args: Vec::new(),
            assign: None,
                            auto_call: true,
        })
    }
}

// ---------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Binding {
    name: String,
    ty: TypeId,
    mutable: bool,
    no_shadow: bool,
    defined: bool,
    moved: bool,
    span: Span,
    kind: BindingKind,
}

#[derive(Debug, Clone)]
enum BindingKind {
    Var,
    Func {
        symbol: String,
        effect: Effect,
        arity: usize,
        builtin: Option<BuiltinKind>,
        type_param_bounds: Vec<Vec<String>>,
        captures: Vec<(String, TypeId)>,
    },
}

#[derive(Debug)]
struct Env {
    scopes: Vec<Vec<Binding>>,
}

impl Env {
    fn new() -> Self {
        Self {
            scopes: vec![Vec::new()],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_global(&mut self, binding: Binding) {
        if let Some(scope) = self.scopes.first_mut() {
            scope.push(binding);
        }
    }

    fn remove_duplicate_func(&mut self, name: &str, ty: TypeId, ctx: &TypeCtx) {
        let target = function_signature_string(ctx, ty);
        if let Some(scope) = self.scopes.first_mut() {
            scope.retain(|b| {
                if b.name != name {
                    return true;
                }
                if let BindingKind::Func { .. } = b.kind {
                    let existing = function_signature_string(ctx, b.ty);
                    existing != target
                } else {
                    true
                }
            });
        }
    }

    fn insert_local(&mut self, binding: Binding) -> Result<(), ()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.iter().any(|b| b.name == binding.name) {
                let has_var = scope
                    .iter()
                    .any(|b| b.name == binding.name && matches!(b.kind, BindingKind::Var));
                let new_is_var = matches!(binding.kind, BindingKind::Var);
                if has_var || new_is_var {
                    return Err(());
                }
            }
            scope.push(binding);
        }
        Ok(())
    }

    fn lookup_current(&self, name: &str) -> Option<&Binding> {
        self.scopes
            .last()
            .and_then(|scope| scope.iter().rev().find(|b| b.name == name))
    }

    fn lookup_current_mut(&mut self, name: &str) -> Option<&mut Binding> {
        self.scopes
            .last_mut()
            .and_then(|scope| scope.iter_mut().rev().find(|b| b.name == name))
    }

    fn lookup(&self, name: &str) -> Option<&Binding> {
        // When resolving identifiers for reading, skip hoisted bindings
        // that are not yet defined. This prevents the RHS of a hoisted
        // `let` from accidentally seeing the placeholder binding.
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope.iter().rev().find(|b| b.name == name && b.defined) {
                return Some(b);
            }
        }
        None
    }

    fn lookup_all(&self, name: &str) -> Vec<&Binding> {
        for scope in self.scopes.iter().rev() {
            let mut items: Vec<&Binding> = scope
                .iter()
                .filter(|b| b.name == name && b.defined)
                .collect();
            if !items.is_empty() {
                return items;
            }
        }
        Vec::new()
    }

    fn lookup_value(&self, name: &str) -> Option<&Binding> {
        self.lookup_all(name)
            .into_iter()
            .find(|b| matches!(b.kind, BindingKind::Var))
    }

    fn lookup_all_callables(&self, name: &str) -> Vec<&Binding> {
        self.lookup_all(name)
            .into_iter()
            .filter(|b| matches!(b.kind, BindingKind::Func { .. }))
            .collect()
    }

    fn lookup_callable_any(&self, name: &str) -> Option<&Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope
                .iter()
                .rev()
                .find(|b| b.name == name && b.defined && matches!(b.kind, BindingKind::Func { .. }))
            {
                return Some(b);
            }
        }
        None
    }

    fn lookup_callable(&self, name: &str) -> Option<&Binding> {
        self.lookup_all_callables(name).into_iter().next()
    }

    /// 同名候補から型シグネチャ一致の関数シンボルを返す。
    ///
    /// typecheck 本体と HIR 生成で関数名決定ロジックを共有し、
    /// hoist した symbol と最終的な HIR 名の不整合を防ぐ。
    fn lookup_func_symbol(&self, name: &str, ty: TypeId, ctx: &TypeCtx) -> Option<String> {
        let target_sig = function_signature_string(ctx, ty);
        for binding in self.lookup_all_callables(name) {
            if let BindingKind::Func { symbol, .. } = &binding.kind {
                if function_signature_string(ctx, binding.ty) == target_sig {
                    return Some(symbol.clone());
                }
            }
        }
        None
    }

    fn lookup_outer_defined(&self, name: &str) -> Option<&Binding> {
        if self.scopes.len() <= 1 {
            return None;
        }
        for scope in self.scopes[..self.scopes.len() - 1].iter().rev() {
            if let Some(binding) = scope.iter().rev().find(|b| b.name == name && b.defined) {
                return Some(binding);
            }
        }
        None
    }

    fn lookup_any(&self, name: &str) -> Option<&Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope.iter().rev().find(|b| b.name == name) {
                return Some(b);
            }
        }
        None
    }

    fn lookup_mut(&mut self, name: &str) -> Option<&mut Binding> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(pos) = scope.iter().rposition(|b| b.name == name) {
                return scope.get_mut(pos);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------

fn is_important_shadow_symbol(name: &str) -> bool {
    matches!(
        name,
        "print"
            | "println"
            | "print_i32"
            | "println_i32"
            | "add"
            | "sub"
            | "mul"
            | "div"
            | "eq"
            | "lt"
            | "le"
            | "gt"
            | "ge"
    )
}

fn emit_shadow_warning(
    diagnostics: &mut Vec<Diagnostic>,
    env: &Env,
    name: &str,
    span: Span,
    kind: &str,
) {
    if let Some(shadowed) = env.lookup_outer_defined(name) {
        if !is_important_shadow_symbol(name) {
            return;
        }
        let message = format!(
            "important symbol '{}' is shadowed by local {}",
            name, kind
        );
        let mut diag = Diagnostic::warning(message, span);
        diag = diag.with_secondary_label(
            shadowed.span,
            Some(String::from("shadowed definition is here")),
        );
        diagnostics.push(diag);
    } else if is_important_shadow_symbol(name) {
        diagnostics.push(Diagnostic::warning(
            format!(
                "definition '{}' may shadow important stdlib symbol ({})",
                name, kind
            ),
            span,
        ));
    }
}

fn shadow_blocked_by_nonshadow<'a>(env: &'a Env, name: &str) -> Option<&'a Binding> {
    env.lookup_any(name)
        .and_then(|b| if b.no_shadow && b.defined { Some(b) } else { None })
}

fn is_callable_binding(binding: &Binding) -> bool {
    matches!(binding.kind, BindingKind::Func { .. })
}

fn find_same_signature_func<'a>(
    env: &'a Env,
    name: &str,
    ty: TypeId,
    ctx: &TypeCtx,
) -> Option<&'a Binding> {
    let target_sig = function_signature_string(ctx, ty);
    env.lookup_all_callables(name).into_iter().find(|b| {
        matches!(b.kind, BindingKind::Func { .. })
            && function_signature_string(ctx, b.ty) == target_sig
    })
}

type LabelEnv = BTreeMap<String, TypeId>;

#[derive(Debug)]
struct StringTable {
    map: BTreeMap<String, u32>,
    items: Vec<String>,
}

impl StringTable {
    fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            items: Vec::new(),
        }
    }

    fn intern(&mut self, s: String) -> u32 {
        if let Some(id) = self.map.get(&s) {
            *id
        } else {
            let id = self.items.len() as u32;
            self.items.push(s.clone());
            self.map.insert(s, id);
            id
        }
    }

    fn get(&self, id: u32) -> Option<&String> {
        self.items.get(id as usize)
    }

    fn into_vec(self) -> Vec<String> {
        self.items
    }
}

fn type_from_expr(ctx: &mut TypeCtx, labels: &mut LabelEnv, t: &TypeExpr) -> TypeId {
    match t {
        TypeExpr::Unit => ctx.unit(),
        TypeExpr::I32 => ctx.i32(),
        TypeExpr::U8 => ctx.u8(),
        TypeExpr::F32 => ctx.f32(),
        TypeExpr::Bool => ctx.bool(),
        TypeExpr::Str => ctx.str(),
        TypeExpr::Never => ctx.never(),
        TypeExpr::Named(name) => {
            match name.as_str() {
                "i32" => ctx.i32(),
                "u8" => ctx.u8(),
                "f32" => ctx.f32(),
                "bool" => ctx.bool(),
                "str" => ctx.str(),
                "never" => ctx.never(),
                _ => {
                    if let Some(id) = labels.get(name) {
                        return *id;
                    }
                    if let Some(id) = ctx.lookup_named(name) {
                        id
                    } else {
                        ctx.register_named(name.clone(), TypeKind::Named(name.clone()))
                    }
                }
            }
        }
        TypeExpr::Apply(base, args) => {
            let b = type_from_expr(ctx, labels, base);
            let mut arg_tys = Vec::new();
            for a in args {
                arg_tys.push(type_from_expr(ctx, labels, a));
            }
            ctx.apply(b, arg_tys)
        }
        TypeExpr::Label(label) => {
            if let Some(name) = label {
                if let Some(existing) = labels.get(name) {
                    *existing
                } else {
                    let id = ctx.fresh_var(Some(name.clone()));
                    labels.insert(name.clone(), id);
                    id
                }
            } else {
                ctx.fresh_var(None)
            }
        }
        TypeExpr::Function {
            params,
            result,
            effect,
        } => {
            let mut p = Vec::new();
            for ty in params {
                p.push(type_from_expr(ctx, labels, ty));
            }
            let r = type_from_expr(ctx, labels, result);
            ctx.function(Vec::new(), p, r, *effect)
        }
        TypeExpr::Tuple(items) => {
            let mut elems = Vec::new();
            for ty in items {
                elems.push(type_from_expr(ctx, labels, ty));
            }
            ctx.tuple(elems)
        }
        TypeExpr::Boxed(inner) => {
            let i = type_from_expr(ctx, labels, inner);
            ctx.box_ty(i)
        }
        TypeExpr::Reference(inner, is_mut) => {
            let i = type_from_expr(ctx, labels, inner);
            ctx.reference(i, *is_mut)
        }
    }
}

fn func_arity(ctx: &TypeCtx, ty: TypeId) -> usize {
    match ctx.get(ty) {
        TypeKind::Function { params, .. } => params.len(),
        _ => 0,
    }
}

fn parse_variant_name(name: &str) -> Option<(&str, &str)> {
    let mut parts = name.splitn(2, "::");
    let a = parts.next()?;
    let b = parts.next()?;
    Some((a, b))
}

fn mangle_function_symbol(base: &str, func_ty: TypeId, ctx: &TypeCtx) -> String {
    let mut s = String::new();
    s.push_str(base);
    if let TypeKind::Function {
        params,
        result,
        effect,
        ..
    } = ctx.get(func_ty)
    {
        s.push_str("__");
        if params.is_empty() {
            s.push_str("unit");
        } else {
            for (i, p) in params.iter().enumerate() {
                if i > 0 {
                    s.push('_');
                }
                s.push_str(&ctx.type_to_string(*p));
            }
        }
        s.push_str("__");
        s.push_str(&ctx.type_to_string(result));
        match effect {
            Effect::Pure => s.push_str("__pure"),
            Effect::Impure => s.push_str("__imp"),
        }
    }
    s
}

fn mangle_impl_method(trait_name: &str, method: &str, target_ty: TypeId, ctx: &TypeCtx) -> String {
    let mut name = String::new();
    name.push_str(trait_name);
    name.push_str("::");
    name.push_str(method);
    name.push_str("__");
    name.push_str(&ctx.type_to_string(target_ty));
    name
}

fn function_signature_string(ctx: &TypeCtx, ty: TypeId) -> String {
    let resolved = ctx.resolve_id(ty);
    match ctx.get(resolved) {
        TypeKind::Function {
            params,
            result,
            effect,
            ..
        } => {
            let mut s = String::from("func");
            s.push_str("__");
            if params.is_empty() {
                s.push_str("unit");
            } else {
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&ctx.type_to_string(*p));
                }
            }
            s.push_str("__");
            s.push_str(&ctx.type_to_string(result));
            match effect {
                Effect::Pure => s.push_str("__pure"),
                Effect::Impure => s.push_str("__imp"),
            }
            s
        }
        _ => ctx.type_to_string(resolved),
    }
}

fn type_signature_matches(ctx: &TypeCtx, a: TypeId, b: TypeId) -> bool {
    function_signature_string(ctx, a) == function_signature_string(ctx, b)
}

fn type_contains_unbound_var(ctx: &TypeCtx, ty: TypeId) -> bool {
    let ty = ctx.resolve_id(ty);
    match ctx.get(ty) {
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Named(_) => false,
        TypeKind::Var(tv) => tv.binding.is_none(),
        TypeKind::Enum { type_params, .. } | TypeKind::Struct { type_params, .. } => {
            !type_params.is_empty()
        }
        TypeKind::Function {
            type_params,
            params,
            result,
            ..
        } => {
            if !type_params.is_empty() {
                return true;
            }
            params
                .iter()
                .any(|p| type_contains_unbound_var(ctx, *p))
                || type_contains_unbound_var(ctx, result)
        }
        TypeKind::Tuple { items } => items.iter().any(|t| type_contains_unbound_var(ctx, *t)),
        TypeKind::Apply { base: _, args } => {
            args.iter().any(|t| type_contains_unbound_var(ctx, *t))
        }
        TypeKind::Box(inner) => type_contains_unbound_var(ctx, inner),
        TypeKind::Reference(inner, _) => type_contains_unbound_var(ctx, inner),
    }
}

fn parse_i32_literal(text: &str) -> Option<i32> {
    let (neg, digits) = if let Some(rest) = text.strip_prefix('-') {
        (true, rest)
    } else {
        (false, text)
    };
    let (radix, digits) = if let Some(rest) = digits.strip_prefix("0x") {
        (16, rest)
    } else if let Some(rest) = digits.strip_prefix("0X") {
        (16, rest)
    } else {
        (10, digits)
    };
    if digits.is_empty() {
        return None;
    }
    let unsigned = i128::from_str_radix(digits, radix).ok()?;
    let signed = if neg { -unsigned } else { unsigned };
    Some(signed as i32)
}

fn target_allows(target: &str, active: CompileTarget) -> bool {
    match target {
        "wasm" => true,
        "wasi" => matches!(active, CompileTarget::Wasi),
        _ => false,
    }
}

fn profile_allows(profile: &str, active: BuildProfile) -> bool {
    match profile {
        "debug" => matches!(active, BuildProfile::Debug),
        "release" => matches!(active, BuildProfile::Release),
        _ => false,
    }
}

fn gate_allows(
    d: &Directive,
    target: CompileTarget,
    active_profile: BuildProfile,
) -> Option<bool> {
    match d {
        Directive::IfTarget { target: gate, .. } => Some(target_allows(gate.as_str(), target)),
        Directive::IfProfile { profile, .. } => {
            Some(profile_allows(profile.as_str(), active_profile))
        }
        _ => None,
    }
}

#[derive(Clone, PartialEq, Debug)]
enum AssignKind {
    Let,
    Set,
    Store(HirExpr),
    AddrOf,
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
