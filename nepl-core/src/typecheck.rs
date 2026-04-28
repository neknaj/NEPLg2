extern crate alloc;
#[cfg(not(target_os = "none"))]
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
use crate::diagnostic_ids::DiagnosticId;
use crate::effects::{
    intrinsic_effect, intrinsic_is_raw_memory_effect, raw_body_direct_callees,
    raw_body_memory_operations, raw_callee_is_raw_memory_effect,
};
use crate::hir::*;
use crate::layout::{composite_field_offset_bytes, is_aggregate_storage_type};
use crate::resolve::{DefId, ImportResolution};
use crate::source_map::SourceMap;
use crate::span::Span;
use crate::types::{EnumVariantInfo, TypeCtx, TypeId, TypeKind};

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
struct TraitInfo {
    doc: Option<String>,
    type_params: Vec<TypeId>,
    capabilities: Vec<TraitCapability>,
    methods: BTreeMap<String, TypeId>,
    self_ty: TypeId,
    span: Span,
}

#[derive(Debug, Clone, Default)]
struct TraitSemantics {
    copy_traits: Vec<(String, TypeId)>,
    clone_traits: Vec<(String, TypeId)>,
    drop_traits: Vec<(String, TypeId)>,
}

impl TraitSemantics {
    fn detect(traits: &BTreeMap<String, TraitInfo>) -> Self {
        let mut copy_traits: Vec<(String, TypeId)> = Vec::new();
        let mut clone_traits: Vec<(String, TypeId)> = Vec::new();
        let mut drop_traits: Vec<(String, TypeId)> = Vec::new();

        for (name, info) in traits {
            for cap in info.capabilities.iter().copied() {
                match cap {
                    TraitCapability::Copy => {
                        if !copy_traits.iter().any(|(_, id)| *id == info.self_ty) {
                            copy_traits.push((name.clone(), info.self_ty));
                        }
                    }
                    TraitCapability::Clone => {
                        if !clone_traits.iter().any(|(_, id)| *id == info.self_ty) {
                            clone_traits.push((name.clone(), info.self_ty));
                        }
                    }
                    TraitCapability::Drop => {
                        if !drop_traits.iter().any(|(_, id)| *id == info.self_ty) {
                            drop_traits.push((name.clone(), info.self_ty));
                        }
                    }
                }
            }
        }

        Self {
            copy_traits,
            clone_traits,
            drop_traits,
        }
    }

    fn has_any_copy_capability(&self) -> bool {
        !self.copy_traits.is_empty()
    }

    fn has_copy_capability(&self, trait_id: Option<TypeId>) -> bool {
        match trait_id {
            Some(actual) => self.copy_traits.iter().any(|(_, id)| *id == actual),
            None => false,
        }
    }

    fn has_clone_capability(&self, trait_id: Option<TypeId>) -> bool {
        match trait_id {
            Some(actual) => self.clone_traits.iter().any(|(_, id)| *id == actual),
            None => false,
        }
    }

    fn has_drop_capability(&self, trait_id: Option<TypeId>) -> bool {
        match trait_id {
            Some(actual) => self.drop_traits.iter().any(|(_, id)| *id == actual),
            None => false,
        }
    }
}

#[derive(Debug, Clone)]
struct ImplInfo {
    trait_name: Option<String>,
    trait_base_name: Option<String>,
    trait_args: Vec<TypeId>,
    trait_self_ty: Option<TypeId>,
    target_ty: TypeId,
}

#[derive(Debug, Clone)]
struct TraitBoundRef {
    name: String,
    trait_base_name: String,
    trait_args: Vec<TypeId>,
    trait_self_ty: TypeId,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraitCapability {
    Copy,
    Clone,
    Drop,
}

fn collect_type_params(
    ctx: &mut TypeCtx,
    labels: &mut LabelEnv,
    params: &[TypeParam],
    traits: &BTreeMap<String, TraitInfo>,
    diags: &mut Vec<Diagnostic>,
) -> (
    Vec<TypeId>,
    Vec<Vec<TraitBoundRef>>,
    BTreeMap<TypeId, Vec<TraitBoundRef>>,
) {
    let mut tps = Vec::new();
    let mut bounds_vec = Vec::new();
    let mut bounds_map = BTreeMap::new();
    for p in params {
        let id = ctx.fresh_var(Some(p.name.name.clone()));
        labels.insert(p.name.name.clone(), id);
        let mut bounds = Vec::new();
        let mut copy_cap = false;
        let mut clone_cap = false;
        let mut drop_cap = false;
        for b in &p.bounds {
            if let Some(info) = traits.get(&b.name.name) {
                if info.type_params.len() != b.args.len() {
                    diags.push(
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
                    .map(|arg| type_from_expr(ctx, labels, arg))
                    .collect();
                bounds.push(TraitBoundRef {
                    name: format_trait_ref_name(&b.name.name, &arg_tys, ctx),
                    trait_base_name: b.name.name.clone(),
                    trait_args: arg_tys,
                    trait_self_ty: info.self_ty,
                });
                for cap in info.capabilities.iter().copied() {
                    match cap {
                        TraitCapability::Copy => copy_cap = true,
                        TraitCapability::Clone => clone_cap = true,
                        TraitCapability::Drop => drop_cap = true,
                    }
                }
            } else {
                diags.push(
                    Diagnostic::error(
                        format!("unknown trait bound '{}'", b.name.name),
                        p.name.span,
                    )
                    .with_id(DiagnosticId::TypeUnknownTraitBound),
                );
            }
        }
        ctx.set_var_capabilities(id, copy_cap, clone_cap, drop_cap);
        if !bounds.is_empty() {
            bounds_map.insert(id, bounds.clone());
        }
        bounds_vec.push(bounds);
        tps.push(id);
    }
    (tps, bounds_vec, bounds_map)
}

fn format_trait_ref_name(base: &str, args: &[TypeId], ctx: &TypeCtx) -> String {
    if args.is_empty() {
        return base.to_string();
    }
    let mut name = String::from(base);
    name.push('<');
    for (idx, arg) in args.iter().enumerate() {
        if idx > 0 {
            name.push(',');
        }
        name.push_str(&ctx.type_to_string(*arg));
    }
    name.push('>');
    name
}

fn trait_application_matches(
    ctx: &TypeCtx,
    base_name: &str,
    args: &[TypeId],
    other_base_name: &str,
    other_args: &[TypeId],
) -> bool {
    if base_name != other_base_name || args.len() != other_args.len() {
        return false;
    }
    args.iter().zip(other_args.iter()).all(|(lhs, rhs)| {
        let lhs = ctx.resolve_id(*lhs);
        let rhs = ctx.resolve_id(*rhs);
        ctx.type_pattern_matches(lhs, rhs) || ctx.type_pattern_matches(rhs, lhs)
    })
}

fn type_param_has_trait_bound(
    ctx: &TypeCtx,
    type_param_bounds: &BTreeMap<TypeId, Vec<TraitBoundRef>>,
    ty: TypeId,
    trait_name: &str,
) -> bool {
    let matches_bound = |b: &TraitBoundRef| {
        if b.name == trait_name {
            return true;
        }
        if let Some((base, args)) = parse_trait_ref_name(trait_name, ctx) {
            return trait_application_matches(ctx, &base, &args, &b.trait_base_name, &b.trait_args);
        }
        false
    };
    let resolved = ctx.resolve_id(ty);
    if let Some(bounds) = type_param_bounds.get(&resolved) {
        return bounds.iter().any(matches_bound);
    }
    if type_param_bounds
        .iter()
        .any(|(tp, bounds)| ctx.resolve_id(*tp) == resolved && bounds.iter().any(matches_bound))
    {
        return true;
    }
    let label = match ctx.get(resolved) {
        TypeKind::Var(v) => v.label.clone(),
        _ => None,
    };
    let Some(label) = label else {
        return false;
    };
    type_param_bounds.iter().any(|(tp, bounds)| {
        let same_label = match ctx.get(ctx.resolve_id(*tp)) {
            TypeKind::Var(v) => v.label.as_deref() == Some(label.as_str()),
            _ => false,
        };
        same_label && bounds.iter().any(matches_bound)
    })
}

fn parse_trait_ref_name(name: &str, ctx: &TypeCtx) -> Option<(String, Vec<TypeId>)> {
    let lt = name.find('<')?;
    let gt = name.rfind('>')?;
    if gt <= lt {
        return None;
    }
    let base = name[..lt].to_string();
    let inner = &name[lt + 1..gt];
    if inner.trim().is_empty() {
        return Some((base, Vec::new()));
    }
    let mut args = Vec::new();
    for part in inner.split(',') {
        let ty_name = part.trim();
        let ty = match ty_name {
            "i32" => Some(ctx.i32()),
            "u8" => Some(ctx.u8()),
            "f32" => Some(ctx.f32()),
            "bool" => Some(ctx.bool()),
            "char" => Some(ctx.char()),
            "str" => Some(ctx.str()),
            _ => None,
        }?;
        args.push(ty);
    }
    Some((base, args))
}

fn merge_inferred_instantiation(
    ctx: &TypeCtx,
    current: Option<TypeId>,
    candidate: Option<TypeId>,
) -> Option<TypeId> {
    match (current, candidate) {
        (None, other) => other,
        (some, None) => some,
        (Some(a), Some(b)) if ctx.same_type(a, b) => Some(ctx.resolve_id(a)),
        _ => None,
    }
}

fn infer_type_param_from_instantiated_pair(
    ctx: &TypeCtx,
    original: TypeId,
    instantiated: TypeId,
    target_tp: TypeId,
    target_label: Option<&str>,
) -> Option<TypeId> {
    let original = ctx.resolve_id(original);
    let instantiated = ctx.resolve_id(instantiated);
    if original == ctx.resolve_id(target_tp) {
        return Some(instantiated);
    }

    let original_is_same_label = match ctx.get(original) {
        TypeKind::Var(v) => target_label
            .map(|label| v.label.as_deref() == Some(label))
            .unwrap_or(false),
        _ => false,
    };
    if original_is_same_label {
        return Some(instantiated);
    }

    match (ctx.get(original), ctx.get(instantiated)) {
        (
            TypeKind::Function {
                params: params_a,
                result: result_a,
                ..
            },
            TypeKind::Function {
                params: params_b,
                result: result_b,
                ..
            },
        ) if params_a.len() == params_b.len() => {
            let mut found = None;
            for (pa, pb) in params_a.iter().zip(params_b.iter()) {
                found = merge_inferred_instantiation(
                    ctx,
                    found,
                    infer_type_param_from_instantiated_pair(ctx, *pa, *pb, target_tp, target_label),
                );
            }
            merge_inferred_instantiation(
                ctx,
                found,
                infer_type_param_from_instantiated_pair(
                    ctx,
                    result_a,
                    result_b,
                    target_tp,
                    target_label,
                ),
            )
        }
        (
            TypeKind::Enum {
                type_params: args_a,
                ..
            },
            TypeKind::Enum {
                type_params: args_b,
                ..
            },
        )
        | (
            TypeKind::Struct {
                type_params: args_a,
                ..
            },
            TypeKind::Struct {
                type_params: args_b,
                ..
            },
        )
        | (TypeKind::Apply { args: args_a, .. }, TypeKind::Apply { args: args_b, .. })
            if args_a.len() == args_b.len() =>
        {
            let mut found = None;
            for (aa, ab) in args_a.iter().zip(args_b.iter()) {
                found = merge_inferred_instantiation(
                    ctx,
                    found,
                    infer_type_param_from_instantiated_pair(ctx, *aa, *ab, target_tp, target_label),
                );
            }
            found
        }
        (TypeKind::Tuple { items: items_a }, TypeKind::Tuple { items: items_b })
            if items_a.len() == items_b.len() =>
        {
            let mut found = None;
            for (ia, ib) in items_a.iter().zip(items_b.iter()) {
                found = merge_inferred_instantiation(
                    ctx,
                    found,
                    infer_type_param_from_instantiated_pair(ctx, *ia, *ib, target_tp, target_label),
                );
            }
            found
        }
        (TypeKind::Box(inner_a), TypeKind::Box(inner_b))
        | (TypeKind::Reference(inner_a, _), TypeKind::Reference(inner_b, _)) => {
            infer_type_param_from_instantiated_pair(ctx, inner_a, inner_b, target_tp, target_label)
        }
        _ => None,
    }
}

fn infer_instantiated_type_arg(
    ctx: &TypeCtx,
    original_fn_ty: TypeId,
    instantiated_fn_ty: TypeId,
    target_tp: TypeId,
) -> Option<TypeId> {
    let target_label = match ctx.get(ctx.resolve_id(target_tp)) {
        TypeKind::Var(v) => v.label.clone(),
        _ => None,
    };
    infer_type_param_from_instantiated_pair(
        ctx,
        original_fn_ty,
        instantiated_fn_ty,
        target_tp,
        target_label.as_deref(),
    )
    .map(|ty| ctx.resolve_id(ty))
}

fn insert_substitution_mapping(
    ctx: &TypeCtx,
    mapping: &mut BTreeMap<TypeId, TypeId>,
    param: TypeId,
    arg: TypeId,
) {
    mapping.insert(param, arg);
    let resolved_param = ctx.resolve_id(param);
    if resolved_param != param {
        mapping.insert(resolved_param, arg);
    }
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
    fn lookup_qualified_bindings(&self, id: &Ident) -> Option<(String, Vec<Binding>)> {
        let (ns, member) = parse_variant_name(&id.name)?;
        if self.enums.contains_key(ns) || self.traits.contains_key(ns) {
            return None;
        }
        let target_files = self
            .import_resolution
            .qualified_targets_for_alias(id.span.file_id.0, ns)?;
        let bindings = self
            .env
            .lookup_all_any_defined(member)
            .into_iter()
            .filter(|b| target_files.contains(&b.span.file_id.0))
            .cloned()
            .collect::<Vec<_>>();
        Some((member.to_string(), bindings))
    }

    fn unqualified_lookup_names(&self, id: &Ident) -> Vec<String> {
        self.import_resolution
            .unqualified_lookup_names(id.span.file_id.0, &id.name)
    }

    fn binding_is_visible_unqualified(&self, id: &Ident, binding: &Binding) -> bool {
        self.import_resolution.binding_is_visible_unqualified(
            id.span.file_id.0,
            &id.name,
            binding.span.file_id.0,
            &binding.name,
        )
    }

    fn lookup_all_unqualified_any_defined(&self, id: &Ident) -> Vec<&Binding> {
        let names = self.unqualified_lookup_names(id);
        for scope in self.env.scopes.iter().rev() {
            let mut items = Vec::new();
            for name in &names {
                items.extend(scope.values.iter().filter(|b| {
                    b.name == *name && b.defined && self.binding_is_visible_unqualified(id, b)
                }));
                items.extend(scope.callables.iter().filter(|b| {
                    b.name == *name && b.defined && self.binding_is_visible_unqualified(id, b)
                }));
            }
            if !items.is_empty() {
                return items;
            }
        }
        Vec::new()
    }

    fn lookup_all_unqualified_callables(&self, id: &Ident) -> Vec<&Binding> {
        let names = self.unqualified_lookup_names(id);
        let mut items = Vec::new();
        for scope in self.env.scopes.iter().rev() {
            for name in &names {
                items.extend(scope.callables.iter().filter(|b| {
                    b.name == *name && b.defined && self.binding_is_visible_unqualified(id, b)
                }));
            }
        }
        items
    }

    fn lookup_unqualified_callable_any(&self, id: &Ident) -> Option<&Binding> {
        let names = self.unqualified_lookup_names(id);
        for scope in self.env.scopes.iter().rev() {
            for name in &names {
                if let Some(binding) = scope.callables.iter().rev().find(|b| {
                    b.name == *name && b.defined && self.binding_is_visible_unqualified(id, b)
                }) {
                    return Some(binding);
                }
            }
        }
        None
    }

    fn lookup_unqualified_value_any(&self, id: &Ident) -> Option<&Binding> {
        let names = self.unqualified_lookup_names(id);
        for scope in self.env.scopes.iter().rev() {
            for name in &names {
                if let Some(binding) = scope
                    .values
                    .iter()
                    .rev()
                    .find(|b| b.name == *name && self.binding_is_visible_unqualified(id, b))
                {
                    return Some(binding);
                }
            }
        }
        None
    }

    fn lookup_unqualified_value_for_read(
        &self,
        id: &Ident,
        allow_undefined_nonmut: bool,
    ) -> Option<&Binding> {
        let names = self.unqualified_lookup_names(id);
        for scope in self.env.scopes.iter().rev() {
            for name in &names {
                if let Some(binding) = scope.values.iter().rev().find(|b| {
                    if b.name != *name || !self.binding_is_visible_unqualified(id, b) {
                        return false;
                    }
                    b.defined || (allow_undefined_nonmut && !b.mutable)
                }) {
                    return Some(binding);
                }
            }
        }
        None
    }

    fn validate_raw_body_effect(&mut self, body: &HirBody, span: Span) -> bool {
        if matches!(self.current_effect, Effect::Pure) {
            let memory_ops = raw_body_memory_operations(body);
            if !memory_ops.is_empty() && !self.raw_body_memory_operations_allowed(span) {
                let op = memory_ops
                    .first()
                    .map(String::as_str)
                    .unwrap_or("raw memory operation");
                self.diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "pure raw body cannot access raw memory instruction '{}'",
                            op
                        ),
                        span,
                    )
                    .with_id(DiagnosticId::TypePureCallsImpureFunction),
                );
                return false;
            }
            for callee in raw_body_direct_callees(body) {
                if raw_callee_is_raw_memory_effect(&callee) {
                    if !self.raw_body_memory_operations_allowed(span) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!("pure raw body cannot call raw memory helper '{}'", callee),
                                span,
                            )
                            .with_id(DiagnosticId::TypePureCallsImpureFunction),
                        );
                        return false;
                    }
                    continue;
                }
                if self.raw_callee_is_impure(&callee) {
                    self.diagnostics.push(
                        Diagnostic::error("pure context cannot call impure function", span)
                            .with_id(DiagnosticId::TypePureCallsImpureFunction),
                    );
                    return false;
                }
            }
        }
        true
    }

    fn raw_body_memory_operations_allowed(&self, span: Span) -> bool {
        let Some(source_map) = self.source_map else {
            return false;
        };
        source_map.raw_memory_boundary_allowed(span.file_id)
    }

    fn raw_memory_intrinsic_allowed(&self, name: &str, span: Span) -> bool {
        intrinsic_is_raw_memory_effect(name) && self.raw_body_memory_operations_allowed(span)
    }

    fn raw_callee_is_impure(&self, callee: &str) -> bool {
        if callee.starts_with("llvm.") {
            return false;
        }
        if let Some(effect) = self.raw_callee_declared_effect(callee) {
            return matches!(effect, Effect::Impure);
        }
        matches!(intrinsic_effect(callee), Effect::Impure)
    }

    fn raw_callee_declared_effect(&self, callee: &str) -> Option<Effect> {
        let mut saw_pure = false;
        for binding in self
            .env
            .lookup_all_callables(callee)
            .into_iter()
            .chain(self.env.lookup_all_callables_by_symbol(callee).into_iter())
        {
            if let BindingKind::Func { effect, .. } = &binding.kind {
                if matches!(effect, Effect::Impure) {
                    return Some(Effect::Impure);
                }
                saw_pure = true;
            }
        }
        if saw_pure {
            Some(Effect::Pure)
        } else {
            None
        }
    }

    fn select_target_raw_body(&mut self, block: &Block) -> Option<HirBody> {
        let mut pending_if: Option<bool> = None;
        let mut selected: Option<HirBody> = None;
        for stmt in &block.items {
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
            match stmt {
                Stmt::Wasm(w) => {
                    if selected.is_some() {
                        self.diagnostics.push(Diagnostic::error(
                            "multiple active raw bodies in one function",
                            w.span,
                        ));
                        return selected;
                    }
                    selected = Some(HirBody::Wasm(w.clone()));
                }
                Stmt::LlvmIr(l) => {
                    if selected.is_some() {
                        self.diagnostics.push(Diagnostic::error(
                            "multiple active raw bodies in one function",
                            l.span,
                        ));
                        return selected;
                    }
                    selected = Some(HirBody::LlvmIr(l.clone()));
                }
                Stmt::Directive(_) => {}
                _ => return None,
            }
        }
        selected
    }

    fn user_visible_arity(&self, func_expr: &HirExpr, params: &[TypeId]) -> usize {
        let total_param_len = params.len();
        if let HirExprKind::Var(name) = &func_expr.kind {
            let bindings = self.env.lookup_all_callables(name);
            if !bindings.is_empty() {
                let mut arity: Option<usize> = None;
                for b in bindings {
                    if let BindingKind::Func { arity: current, .. } = &b.kind {
                        match arity {
                            Some(prev) if prev != *current => return total_param_len,
                            Some(_) => {}
                            None => arity = Some(*current),
                        }
                    }
                }
                if let Some(arity) = arity {
                    return arity;
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
                        if let MatchPattern::Variant { bind: Some(b), .. } = &arm.pattern {
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
        &mut self,
        stack: &[StackEntry],
        inner_pos: usize,
        min_func_pos: usize,
    ) -> Option<usize> {
        for j in (min_func_pos..inner_pos).rev() {
            if self.is_unresolved_overloaded_callable_entry(&stack[j]) {
                continue;
            }
            if !stack[j].auto_call {
                continue;
            }
            let Some((params, _result, _effect)) = self.function_signature_for_entry(&stack[j])
            else {
                continue;
            };
            let total_arity = params.len();
            let arity = self.user_visible_arity(&stack[j].expr, &params);
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

    fn infer_expected_from_outer_consumer(
        &mut self,
        stack: &[StackEntry],
        inner_pos: usize,
        min_func_pos: usize,
    ) -> Option<TypeId> {
        for j in (min_func_pos..inner_pos).rev() {
            if self.is_unresolved_overloaded_callable_entry(&stack[j]) {
                continue;
            }
            if !stack[j].auto_call {
                continue;
            }
            let Some((params, _result, _effect)) = self.function_signature_for_entry(&stack[j])
            else {
                continue;
            };
            let total_arity = params.len();
            let arity = self.user_visible_arity(&stack[j].expr, &params);
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
            if self.has_unresolved_callable_between(stack, j + 1, inner_pos) {
                continue;
            }
            let capture_len = total_arity.saturating_sub(arity);
            let arg_idx = capture_len + user_arg_idx;
            if arg_idx >= total_arity {
                continue;
            }
            // Slots after the current argument may still be arguments to the
            // nested callable being reduced, not siblings of the outer call.
            // Only earlier outer arguments are known to be complete here.
            for k in 0..user_arg_idx {
                let outer_arg_pos = j + 1 + k;
                if outer_arg_pos >= stack.len() {
                    continue;
                }
                let pidx = capture_len + k;
                if pidx >= total_arity {
                    continue;
                }
                let pty = params[pidx];
                let aty = stack[outer_arg_pos].ty;
                let _ = self.ctx.unify(aty, pty);
            }
            return Some(self.ctx.resolve_id(params[arg_idx]));
        }
        None
    }

    fn infer_expected_from_outer_consumer_next_arg(
        &mut self,
        stack: &[StackEntry],
        inner_pos: usize,
        min_func_pos: usize,
    ) -> Option<TypeId> {
        for j in (min_func_pos..inner_pos).rev() {
            if self.is_unresolved_overloaded_callable_entry(&stack[j]) {
                continue;
            }
            if !stack[j].auto_call {
                continue;
            }
            let Some((params, _result, _effect)) = self.function_signature_for_entry(&stack[j])
            else {
                continue;
            };
            let total_arity = params.len();
            let arity = self.user_visible_arity(&stack[j].expr, &params);
            if inner_pos < j + 1 {
                continue;
            }
            let provided_user_args = inner_pos - (j + 1);
            if provided_user_args >= arity {
                continue;
            }
            if self.has_unresolved_callable_between(stack, j + 1, inner_pos) {
                continue;
            }
            let user_arg_idx = provided_user_args;
            let capture_len = total_arity.saturating_sub(arity);
            let arg_idx = capture_len + user_arg_idx;
            if arg_idx >= total_arity {
                continue;
            }
            for k in 0..provided_user_args {
                let outer_arg_pos = j + 1 + k;
                if outer_arg_pos >= stack.len() {
                    continue;
                }
                let pidx = capture_len + k;
                if pidx >= total_arity {
                    continue;
                }
                let pty = params[pidx];
                let aty = stack[outer_arg_pos].ty;
                let _ = self.ctx.unify(aty, pty);
            }
            return Some(self.ctx.resolve_id(params[arg_idx]));
        }
        None
    }

    fn is_unresolved_overloaded_callable_entry(&self, entry: &StackEntry) -> bool {
        let HirExprKind::Var(name) = &entry.expr.kind else {
            return false;
        };
        if !entry.type_args.is_empty() {
            return false;
        }
        self.env.lookup_all_callables(name).len() > 1
    }

    fn function_signature_for_entry(
        &mut self,
        entry: &StackEntry,
    ) -> Option<(Vec<TypeId>, TypeId, Effect)> {
        let rty = self.ctx.resolve_id(entry.ty);
        let TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        } = self.ctx.get(rty)
        else {
            return None;
        };
        if entry.type_args.is_empty() {
            if !type_params.is_empty() {
                let (inst_ty, _fresh_args, _mapping) = self.ctx.instantiate(rty);
                if let TypeKind::Function {
                    params,
                    result,
                    effect,
                    ..
                } = self.ctx.get(inst_ty)
                {
                    return Some((params, result, effect));
                }
                return None;
            }
            return Some((params, result, effect));
        }
        if type_params.len() != entry.type_args.len() {
            // The entry.ty is likely a fresh placeholder (0 type_params) created when
            // the callable was pushed with explicit type args. Look up the actual binding
            // type by name so we can apply the type args correctly.
            if let HirExprKind::Var(name) = &entry.expr.kind {
                let name = name.clone();
                let type_args = entry.type_args.clone();
                let binding_tys: Vec<TypeId> = self
                    .env
                    .lookup_all_callables(&name)
                    .into_iter()
                    .map(|b| b.ty)
                    .collect();
                for binding_ty in binding_tys {
                    let func_data = if let TypeKind::Function {
                        type_params: tps,
                        params: ps,
                        result: r,
                        effect: e,
                    } = self.ctx.get(binding_ty)
                    {
                        Some((tps, ps, r, e))
                    } else {
                        None
                    };
                    let Some((tps, ps, r, e)) = func_data else {
                        continue;
                    };
                    if tps.len() != type_args.len() {
                        continue;
                    }
                    let mut mapping = BTreeMap::new();
                    for (tp, ta) in tps.iter().zip(type_args.iter()) {
                        insert_substitution_mapping(self.ctx, &mut mapping, *tp, *ta);
                    }
                    let sub_params = ps
                        .iter()
                        .map(|p| self.ctx.substitute(*p, &mapping))
                        .collect::<Vec<_>>();
                    let sub_result = self.ctx.substitute(r, &mapping);
                    return Some((sub_params, sub_result, e));
                }
            }
            return None;
        }
        let mut mapping = BTreeMap::new();
        for (p, a) in type_params.iter().zip(entry.type_args.iter()) {
            insert_substitution_mapping(self.ctx, &mut mapping, *p, *a);
        }
        let substituted_params = params
            .iter()
            .map(|p| self.ctx.substitute(*p, &mapping))
            .collect::<Vec<_>>();
        let substituted_result = self.ctx.substitute(result, &mapping);
        Some((substituted_params, substituted_result, effect))
    }

    fn pipe_target_input_type(&mut self, entry: &StackEntry) -> Option<TypeId> {
        let Some((params, _result, _effect)) = self.function_signature_for_entry(entry) else {
            return None;
        };
        let total_arity = params.len();
        let arity = self.user_visible_arity(&entry.expr, &params);
        if arity == 0 {
            return None;
        }
        let capture_len = total_arity.saturating_sub(arity);
        let arg_idx = capture_len;
        if arg_idx >= total_arity {
            return None;
        }
        Some(self.ctx.resolve_id(params[arg_idx]))
    }

    fn reduce_pipe_pending_segment_with_target(
        &mut self,
        mut pending: Vec<StackEntry>,
        target: &StackEntry,
        fallback_expected: Option<TypeId>,
    ) -> Option<StackEntry> {
        if pending.is_empty() {
            return None;
        }
        let expected_input = self
            .pipe_target_input_type(target)
            .filter(|t| self.is_concrete_type(*t))
            .or(fallback_expected.map(|t| self.ctx.resolve_id(t)));
        let mut open_calls = Vec::new();
        for (i, entry) in pending.iter().enumerate() {
            let rty = self.ctx.resolve_id(entry.ty);
            if entry.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. }) {
                open_calls.push(i);
            }
        }
        self.reduce_calls(
            &mut pending,
            &mut open_calls,
            expected_input.map(|t| (t, 0)),
        );
        if pending.len() == 1 {
            pending.pop()
        } else {
            None
        }
    }

    fn pipe_pending_base(
        &mut self,
        stack: &[StackEntry],
        open_calls: &[usize],
        default_base: usize,
    ) -> usize {
        if stack.len() <= default_base + 1 {
            return default_base;
        }
        let top_idx = stack.len() - 1;
        let Some(_) = open_calls
            .iter()
            .rev()
            .copied()
            .find(|&idx| idx >= default_base && idx < top_idx)
        else {
            return default_base;
        };
        if self.pipe_segment_reduces_to_single_value(stack, default_base) {
            return default_base;
        }
        for idx in open_calls.iter().copied() {
            if idx < default_base || idx >= top_idx {
                continue;
            }
            if self.pipe_segment_reduces_to_single_value(stack, idx) {
                return idx;
            }
        }
        top_idx
    }

    fn pipe_segment_reduces_to_single_value(
        &mut self,
        stack: &[StackEntry],
        segment_base: usize,
    ) -> bool {
        if segment_base >= stack.len() {
            return false;
        }
        let checkpoint = self.ctx.checkpoint();
        let diagnostics_len = self.diagnostics.len();
        let trait_checks_len = self.pending_trait_bound_checks.len();
        let mut segment = stack[segment_base..].to_vec();
        let mut open_calls = segment
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                let rty = self.ctx.resolve_id(entry.ty);
                if entry.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. }) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        self.reduce_calls(&mut segment, &mut open_calls, None);
        let reduced = segment.len() == 1;
        self.pending_trait_bound_checks.truncate(trait_checks_len);
        self.diagnostics.truncate(diagnostics_len);
        self.ctx.rollback(checkpoint);
        reduced
    }

    fn has_unresolved_callable_between(
        &self,
        stack: &[StackEntry],
        start: usize,
        end_exclusive: usize,
    ) -> bool {
        if start >= end_exclusive || start >= stack.len() {
            return false;
        }
        let end = end_exclusive.min(stack.len());
        for i in start..end {
            if self.is_unresolved_overloaded_callable_entry(&stack[i]) {
                return true;
            }
        }
        false
    }

    fn unresolved_overloaded_entry_has_larger_arity(
        &mut self,
        stack: &[StackEntry],
        pos: usize,
    ) -> bool {
        if pos >= stack.len() {
            return false;
        }
        let entry = &stack[pos];
        if !self.is_unresolved_overloaded_callable_entry(entry) {
            return false;
        }
        let available_args = stack.len().saturating_sub(pos + 1);
        match &entry.expr.kind {
            HirExprKind::Var(name) => self.env.lookup_all_callables(name).iter().any(|b| match &b
                .kind
            {
                BindingKind::Func {
                    arity, captures, ..
                } => arity.saturating_sub(captures.len()) > available_args,
                _ => false,
            }),
            _ => false,
        }
    }

    fn should_defer_overloaded_nullary_entry(&mut self, stack: &[StackEntry], pos: usize) -> bool {
        if pos >= stack.len() {
            return false;
        }
        let entry = &stack[pos];
        if !self.is_unresolved_overloaded_callable_entry(entry) {
            return false;
        }
        let has_nullary_overload = match &entry.expr.kind {
            HirExprKind::Var(name) => self.env.lookup_all_callables(name).iter().any(|b| match &b
                .kind
            {
                BindingKind::Func {
                    arity, captures, ..
                } => arity.saturating_sub(captures.len()) == 0,
                _ => false,
            }),
            _ => false,
        };
        if !has_nullary_overload {
            return false;
        }
        stack.iter().skip(pos + 1).any(|entry| {
            let rty = self.ctx.resolve_id(entry.ty);
            entry.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. })
        })
    }

    fn choose_callable_type_by_available_arity(
        &mut self,
        name: &str,
        available_args: usize,
    ) -> Option<(usize, TypeId)> {
        let callables = self.env.lookup_all_callables(name);
        if callables.len() <= 1 {
            return None;
        }
        let mut has_mixed_arity = false;
        let mut first_arity: Option<usize> = None;
        for b in &callables {
            if let BindingKind::Func { arity, .. } = b.kind {
                if first_arity.is_none() {
                    first_arity = Some(arity);
                } else if first_arity != Some(arity) {
                    has_mixed_arity = true;
                }
            }
        }
        // In a pure context, prefer pure overloads over impure ones.
        // When selecting by arity, a pure lower-arity overload beats an impure
        // higher-arity one to prevent false D3025 errors from name collisions
        // across modules (e.g. math::add vs fenwick::add in a pure fold).
        let in_pure_context = matches!(self.current_effect, Effect::Pure);

        // Also proceed when arities are uniform but purity is mixed — e.g.
        // vec::with_capacity (pure) vs ringbuffer::with_capacity (impure) both
        // have arity 1.  In a pure context we must pick the pure variant to
        // avoid a spurious D3025 before full overload resolution runs.
        let has_mixed_purity_among_applicable = in_pure_context && {
            let mut has_pure = false;
            let mut has_impure = false;
            for b in &callables {
                if let BindingKind::Func { arity, .. } = b.kind {
                    if arity <= available_args {
                        if matches!(
                            self.ctx.get(self.ctx.resolve_id(b.ty)),
                            TypeKind::Function {
                                effect: Effect::Pure,
                                ..
                            }
                        ) {
                            has_pure = true;
                        } else {
                            has_impure = true;
                        }
                    }
                }
            }
            has_pure && has_impure
        };
        if !has_mixed_arity && !has_mixed_purity_among_applicable {
            return None;
        }

        let mut best: Option<(usize, TypeId, bool)> = None; // (arity, ty, is_pure)
        for b in callables {
            if let BindingKind::Func { arity, .. } = b.kind {
                if arity > available_args {
                    continue;
                }
                let is_pure = matches!(
                    self.ctx.get(self.ctx.resolve_id(b.ty)),
                    TypeKind::Function {
                        effect: Effect::Pure,
                        ..
                    }
                );
                let should_replace = match &best {
                    None => true,
                    Some((_best_arity, _, best_is_pure)) if in_pure_context => {
                        // Pure candidate always beats impure; among same purity prefer higher arity
                        (is_pure && !best_is_pure)
                            || (is_pure == *best_is_pure && arity > *_best_arity)
                    }
                    Some((best_arity, _, _)) => arity > *best_arity,
                };
                if should_replace {
                    best = Some((arity, b.ty, is_pure));
                }
            }
        }
        best.map(|(arity, ty, _)| (arity, ty))
    }

    fn is_concrete_type(&self, ty: TypeId) -> bool {
        !type_contains_unbound_var(self.ctx, ty)
    }

    fn type_param_has_bound_ref(
        &self,
        ty: TypeId,
        trait_base_name: &str,
        trait_args: &[TypeId],
    ) -> bool {
        let matches_bound = |b: &TraitBoundRef| {
            trait_application_matches(
                self.ctx,
                trait_base_name,
                trait_args,
                &b.trait_base_name,
                &b.trait_args,
            )
        };
        let resolved = self.ctx.resolve_id(ty);
        if let Some(bounds) = self.type_param_bounds.get(&resolved) {
            return bounds.iter().any(matches_bound);
        }

        // 型変数が他の型変数へ束縛された場合、resolve 後の TypeId が
        // 直接 type_param_bounds に存在しないことがあるため、正規化後 ID でも照合する。
        if self.type_param_bounds.iter().any(|(tp, bounds)| {
            self.ctx.resolve_id(*tp) == resolved && bounds.iter().any(matches_bound)
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
            same_label && bounds.iter().any(matches_bound)
        })
    }

    fn trait_bound_satisfied_by_ref(&self, bound: &TraitBoundRef, ty: TypeId) -> bool {
        if !self.is_concrete_type(ty) {
            return self.type_param_has_bound_ref(ty, &bound.trait_base_name, &bound.trait_args);
        }
        if crate::log::is_verbose() {
            typecheck_log!(
                "trait_bound_satisfied_by_ref: bound={} trait_self_ty={:?} ty={} ({:?})",
                bound.name,
                bound.trait_self_ty,
                self.ctx.type_to_string(ty),
                self.ctx.resolve_id(ty),
            );
            for imp in self.impls.iter().filter(|imp| {
                imp.trait_base_name
                    .as_deref()
                    .map(|base| {
                        trait_application_matches(
                            self.ctx,
                            &bound.trait_base_name,
                            &bound.trait_args,
                            base,
                            &imp.trait_args,
                        )
                    })
                    .unwrap_or(false)
            }) {
                typecheck_log!(
                    "  impl candidate target={} ({:?}) same_type={}",
                    self.ctx.type_to_string(imp.target_ty),
                    self.ctx.resolve_id(imp.target_ty),
                    self.ctx.same_type(imp.target_ty, ty),
                );
            }
        }
        self.impls.iter().any(|imp| {
            imp.trait_base_name
                .as_deref()
                .map(|base| {
                    trait_application_matches(
                        self.ctx,
                        &bound.trait_base_name,
                        &bound.trait_args,
                        base,
                        &imp.trait_args,
                    )
                })
                .unwrap_or(false)
                && self.ctx.type_pattern_matches(imp.target_ty, ty)
        })
    }

    fn infer_unique_type_param_for_trait_ref(
        &self,
        trait_base_name: &str,
        trait_args: &[TypeId],
    ) -> Option<TypeId> {
        let mut matched: Option<TypeId> = None;
        for (tp, bounds) in &self.type_param_bounds {
            if !bounds.iter().any(|b| {
                trait_application_matches(
                    self.ctx,
                    trait_base_name,
                    trait_args,
                    &b.trait_base_name,
                    &b.trait_args,
                )
            }) {
                continue;
            }
            let resolved = self.ctx.resolve_id(*tp);
            match matched {
                None => matched = Some(resolved),
                Some(prev) if self.ctx.same_type(prev, resolved) => {}
                Some(_) => return None,
            }
        }
        matched
    }

    fn infer_unique_type_param_for_trait(&self, trait_name: &str) -> Option<TypeId> {
        if let Some((base, args)) = parse_trait_ref_name(trait_name, self.ctx) {
            return self.infer_unique_type_param_for_trait_ref(&base, &args);
        }
        self.infer_unique_type_param_for_trait_ref(trait_name, &[])
    }

    fn infer_trait_application_name(
        &self,
        trait_name: &str,
        trait_info: &TraitInfo,
        sig: TypeId,
        args: &[StackEntry],
        expected_ret: Option<TypeId>,
    ) -> String {
        let inferred = self.infer_trait_application_args(trait_info, sig, args, expected_ret);
        format_trait_ref_name(trait_name, &inferred, self.ctx)
    }

    fn infer_trait_application_args(
        &self,
        trait_info: &TraitInfo,
        sig: TypeId,
        args: &[StackEntry],
        expected_ret: Option<TypeId>,
    ) -> Vec<TypeId> {
        if trait_info.type_params.is_empty() {
            return Vec::new();
        }
        let resolved_sig = self.ctx.resolve_id(sig);
        let TypeKind::Function { params, result, .. } = self.ctx.get(resolved_sig) else {
            return Vec::new();
        };
        let mut inferred = Vec::new();
        for tp in &trait_info.type_params {
            let label = match self.ctx.get(self.ctx.resolve_id(*tp)) {
                TypeKind::Var(v) => v.label.clone(),
                _ => None,
            };
            let mut found = None;
            for (param_ty, arg) in params.iter().zip(args.iter()) {
                found = merge_inferred_instantiation(
                    self.ctx,
                    found,
                    infer_type_param_from_instantiated_pair(
                        self.ctx,
                        *param_ty,
                        arg.ty,
                        *tp,
                        label.as_deref(),
                    ),
                );
            }
            if let Some(expected) = expected_ret {
                found = merge_inferred_instantiation(
                    self.ctx,
                    found,
                    infer_type_param_from_instantiated_pair(
                        self.ctx,
                        result,
                        expected,
                        *tp,
                        label.as_deref(),
                    ),
                );
            }
            inferred.push(found.unwrap_or(*tp));
        }
        inferred
    }

    fn resolve_field_access(
        &mut self,
        base_ty: TypeId,
        idx: FieldIdx,
        span: Span,
    ) -> Option<(TypeId, usize)> {
        self.resolve_field_access_with_mode(base_ty, idx, span, true)
    }

    fn resolve_field_access_with_mode(
        &mut self,
        base_ty: TypeId,
        idx: FieldIdx,
        span: Span,
        emit_diagnostics: bool,
    ) -> Option<(TypeId, usize)> {
        fn invalid_field(
            checker: &mut BlockChecker<'_>,
            emit_diagnostics: bool,
            span: Span,
            message: String,
        ) -> Option<(TypeId, usize)> {
            if emit_diagnostics {
                checker.diagnostics.push(
                    Diagnostic::error(message, span).with_id(DiagnosticId::TypeInvalidFieldAccess),
                );
            }
            None
        }

        let resolved_ty = self.ctx.resolve(base_ty);
        match self.ctx.get(resolved_ty) {
            TypeKind::Struct {
                fields,
                field_names,
                ..
            } => match idx {
                FieldIdx::Index(i) => {
                    if i < fields.len() {
                        Some((
                            fields[i],
                            composite_field_offset_bytes(self.ctx, &fields, i),
                        ))
                    } else {
                        invalid_field(
                            self,
                            emit_diagnostics,
                            span,
                            format!("struct index out of bounds: {}", i),
                        )
                    }
                }
                FieldIdx::Name(name) => {
                    if let Some(i) = field_names.iter().position(|n| *n == name) {
                        Some((
                            fields[i],
                            composite_field_offset_bytes(self.ctx, &fields, i),
                        ))
                    } else {
                        invalid_field(
                            self,
                            emit_diagnostics,
                            span,
                            format!("struct has no field {}", name),
                        )
                    }
                }
            },
            TypeKind::Tuple { items } => match idx {
                FieldIdx::Index(i) => {
                    if i < items.len() {
                        Some((items[i], composite_field_offset_bytes(self.ctx, &items, i)))
                    } else {
                        invalid_field(
                            self,
                            emit_diagnostics,
                            span,
                            format!("tuple index out of bounds: {}", i),
                        )
                    }
                }
                FieldIdx::Name(name) => {
                    if let Ok(i) = name.parse::<usize>() {
                        if i < items.len() {
                            Some((items[i], composite_field_offset_bytes(self.ctx, &items, i)))
                        } else {
                            invalid_field(
                                self,
                                emit_diagnostics,
                                span,
                                format!("tuple index out of bounds: {}", i),
                            )
                        }
                    } else {
                        invalid_field(
                            self,
                            emit_diagnostics,
                            span,
                            format!("invalid tuple field access: {}", name),
                        )
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
                                    Some((
                                        substituted_fields[i],
                                        composite_field_offset_bytes(
                                            self.ctx,
                                            &substituted_fields,
                                            i,
                                        ),
                                    ))
                                } else {
                                    invalid_field(
                                        self,
                                        emit_diagnostics,
                                        span,
                                        format!("generic struct index out of bounds: {}", i),
                                    )
                                }
                            }
                            FieldIdx::Name(name) => {
                                if let Some(i) = field_names.iter().position(|n| *n == name) {
                                    Some((
                                        substituted_fields[i],
                                        composite_field_offset_bytes(
                                            self.ctx,
                                            &substituted_fields,
                                            i,
                                        ),
                                    ))
                                } else {
                                    invalid_field(
                                        self,
                                        emit_diagnostics,
                                        span,
                                        format!("generic struct has no field {}", name),
                                    )
                                }
                            }
                        }
                    }
                    TypeKind::Named(type_name) => {
                        if let Some(info) = self.structs.get(&type_name) {
                            let type_params = info.type_params.clone();
                            let fields = info.fields.clone();
                            let field_names = info.field_names.clone();
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
                                        Some((
                                            substituted_fields[i],
                                            composite_field_offset_bytes(
                                                self.ctx,
                                                &substituted_fields,
                                                i,
                                            ),
                                        ))
                                    } else {
                                        invalid_field(
                                            self,
                                            emit_diagnostics,
                                            span,
                                            format!("generic struct index out of bounds: {}", i),
                                        )
                                    }
                                }
                                FieldIdx::Name(name) => {
                                    if let Some(i) = field_names.iter().position(|n| *n == name) {
                                        Some((
                                            substituted_fields[i],
                                            composite_field_offset_bytes(
                                                self.ctx,
                                                &substituted_fields,
                                                i,
                                            ),
                                        ))
                                    } else {
                                        invalid_field(
                                            self,
                                            emit_diagnostics,
                                            span,
                                            format!("generic struct has no field {}", name),
                                        )
                                    }
                                }
                            }
                        } else {
                            invalid_field(
                                self,
                                emit_diagnostics,
                                span,
                                "cannot access field on this type".to_string(),
                            )
                        }
                    }
                    _ => invalid_field(
                        self,
                        emit_diagnostics,
                        span,
                        "cannot access field on this type".to_string(),
                    ),
                }
            }
            TypeKind::Named(type_name) => {
                if let Some(info) = self.structs.get(&type_name) {
                    let fields = info.fields.clone();
                    let field_names = info.field_names.clone();
                    match idx {
                        FieldIdx::Index(i) => {
                            if i < fields.len() {
                                Some((
                                    fields[i],
                                    composite_field_offset_bytes(self.ctx, &fields, i),
                                ))
                            } else {
                                invalid_field(
                                    self,
                                    emit_diagnostics,
                                    span,
                                    format!("struct index out of bounds: {}", i),
                                )
                            }
                        }
                        FieldIdx::Name(name) => {
                            if let Some(i) = field_names.iter().position(|n| *n == name) {
                                Some((
                                    fields[i],
                                    composite_field_offset_bytes(self.ctx, &fields, i),
                                ))
                            } else {
                                invalid_field(
                                    self,
                                    emit_diagnostics,
                                    span,
                                    format!("struct has no field {}", name),
                                )
                            }
                        }
                    }
                } else {
                    invalid_field(
                        self,
                        emit_diagnostics,
                        span,
                        "cannot access field on this type".to_string(),
                    )
                }
            }
            _ => invalid_field(
                self,
                emit_diagnostics,
                span,
                "cannot access field on non-composite type".to_string(),
            ),
        }
    }

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

    fn check_prefix(
        &mut self,
        expr: &PrefixExpr,
        base_depth: usize,
        stack: &mut Vec<StackEntry>,
        expected_last_ty: Option<TypeId>,
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
        let mut pipe_pending: Option<Vec<StackEntry>> = None;
        let mut seen_pipe = false;
        // (target_type, stack_depth_when_annotation_appeared)
        let mut pending_ascription: Option<(TypeId, usize)> =
            expected_last_ty.map(|t| (t, base_depth));

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

        // pipe 直前でも、現在の引数式だけに付いた型注釈は次の pipe へ持ち越さない。
        fn is_local_pending_ascription(
            stack: &[StackEntry],
            pending: Option<(TypeId, usize)>,
            base_depth: usize,
        ) -> bool {
            pending
                .map(|(_, base)| {
                    base > base_depth
                        && stack
                            .get(base.saturating_sub(1))
                            .map(|entry| entry.assign.is_none())
                            .unwrap_or(true)
                })
                .unwrap_or(false)
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
                                    self.diagnostics
                                        .push(Diagnostic::error("invalid integer literal", *span));
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
                        Literal::Char(c) => {
                            let value = if *c <= i32::MAX as u32 {
                                *c as i32
                            } else {
                                self.diagnostics.push(Diagnostic::error(
                                    "char literal is outside current i32-backed codegen range",
                                    *span,
                                ));
                                0
                            };
                            (self.ctx.char(), HirExprKind::LiteralI32(value))
                        }
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
                        let qualified_bindings = self.lookup_qualified_bindings(id);
                        let in_let_self_init = stack.iter().rev().any(|e| {
                            matches!(e.assign, Some(AssignKind::Let))
                                && matches!(&e.expr.kind, HirExprKind::Var(n) if n == &id.name)
                        });
                        if let Some(entry) = self.resolve_dotted_field_symbol(id, *forced_value) {
                            stack.push(entry);
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        } else {
                            let selected_from_qualified = qualified_bindings.is_some();
                            let selected_binding = if let Some((_, qualified)) = &qualified_bindings
                            {
                                if qualified.len() == 1 {
                                    Some((qualified[0].clone(), false))
                                } else {
                                    None
                                }
                            } else {
                                let explicit_callable_candidate =
                                    if !*forced_value && !type_args.is_empty() {
                                        let mut matching = self
                                            .lookup_all_unqualified_callables(id)
                                            .into_iter()
                                            .filter(|binding| {
                                                let ty = self.ctx.resolve_id(binding.ty);
                                                match self.ctx.get(ty) {
                                                    TypeKind::Function { type_params, .. } => {
                                                        type_params.len() == type_args.len()
                                                    }
                                                    _ => false,
                                                }
                                            })
                                            .cloned()
                                            .collect::<Vec<_>>();
                                        if matching.len() == 1 {
                                            Some(matching.remove(0))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };
                                let expected_function_from_outer = self
                                    .infer_expected_from_outer_consumer_next_arg(
                                        &stack,
                                        stack.len(),
                                        0,
                                    )
                                    .map(|t| {
                                        let resolved = self.ctx.resolve(t);
                                        matches!(self.ctx.get(resolved), TypeKind::Function { .. })
                                    })
                                    .unwrap_or(false);
                                let expected_function_from_ascription = pending_ascription
                                    .and_then(|(target_ty, base_len)| {
                                        if stack.len() == base_len {
                                            let resolved = self.ctx.resolve(target_ty);
                                            match self.ctx.get(resolved) {
                                                TypeKind::Function { .. } => Some(true),
                                                _ => Some(false),
                                            }
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or(false);
                                let expected_function_from_outer = expected_function_from_outer
                                    || expected_function_from_ascription;
                                let value_candidate =
                                    self.lookup_unqualified_value_for_read(id, !in_let_self_init);
                                let has_any_value = self.lookup_unqualified_value_any(id).is_some();
                                let value_is_function = value_candidate
                                    .map(|b| {
                                        let rty = self.ctx.resolve_id(b.ty);
                                        matches!(self.ctx.get(rty), TypeKind::Function { .. })
                                    })
                                    .unwrap_or(false);
                                let preferred_callable = if !*forced_value
                                    && stack.is_empty()
                                    && expr.items.get(idx + 1).is_some()
                                    && (!has_any_value || !value_is_function)
                                {
                                    self.lookup_unqualified_callable_any(id)
                                } else {
                                    None
                                };
                                let selected = explicit_callable_candidate
                                    .as_ref()
                                    .or(preferred_callable)
                                    .or(value_candidate)
                                    .or_else(|| {
                                        if !has_any_value {
                                            self.lookup_unqualified_callable_any(id)
                                        } else {
                                            None
                                        }
                                    })
                                    .and_then(|binding| {
                                        let should_delay_overload_resolution = !*forced_value
                                            && !expected_function_from_outer
                                            && matches!(binding.kind, BindingKind::Func { .. })
                                            && type_args.is_empty()
                                            && self.lookup_all_unqualified_callables(id).len() > 1
                                            && expr.items.get(idx + 1).is_some();
                                        if should_delay_overload_resolution {
                                            None
                                        } else {
                                            Some(binding)
                                        }
                                    });
                                selected
                                    .cloned()
                                    .map(|binding| (binding, expected_function_from_outer))
                            };
                            if let Some((binding, expected_function_from_outer)) = selected_binding
                            {
                                if *forced_value {
                                    match &binding.kind {
                                        BindingKind::Func { captures, .. } => {
                                            if !captures.is_empty() {
                                                self.diagnostics.push(Diagnostic::error(
                                                    "capturing function cannot be used as a function value yet",
                                                    id.span,
                                                ).with_id(DiagnosticId::TypeCapturingFunctionValueUnsupported));
                                                return None;
                                            }
                                        }
                                        _ => {
                                            self.diagnostics.push(Diagnostic::error(
                                                "only callable symbols can be referenced with '@'",
                                                id.span,
                                            ).with_id(DiagnosticId::TypeAtRequiresCallable));
                                            return None;
                                        }
                                    }
                                }
                                let ty = binding.ty;
                                let auto_call = match &binding.kind {
                                    BindingKind::Func { .. } => {
                                        !*forced_value && !expected_function_from_outer
                                    }
                                    _ => !*forced_value,
                                };
                                let hir_kind = match &binding.kind {
                                    BindingKind::Func { symbol, .. }
                                        if *forced_value
                                            || expected_function_from_outer
                                            || selected_from_qualified
                                            || binding.name != id.name =>
                                    {
                                        HirExprKind::FnValue(symbol.clone())
                                    }
                                    _ => HirExprKind::Var(binding.name.clone()),
                                };
                                let explicit_args = match binding.kind {
                                    BindingKind::Func { .. } => {
                                        let mut args = Vec::new();
                                        for arg_expr in type_args {
                                            args.push(type_from_expr(
                                                self.ctx,
                                                self.labels,
                                                arg_expr,
                                            ));
                                        }
                                        args
                                    }
                                    _ => {
                                        if !type_args.is_empty() {
                                            self.diagnostics.push(
                                                Diagnostic::error(
                                                    "type arguments are not allowed for variables",
                                                    id.span,
                                                )
                                                .with_id(
                                                    DiagnosticId::TypeVariableTypeArgsNotAllowed,
                                                ),
                                            );
                                        }
                                        Vec::new()
                                    }
                                };
                                stack.push(StackEntry {
                                    ty,
                                    expr: HirExpr {
                                        ty,
                                        kind: hir_kind,
                                        span: id.span,
                                    },
                                    type_args: explicit_args,
                                    assign: None,
                                    auto_call,
                                });
                                last_expr = Some(stack.last().unwrap().expr.clone());
                            } else {
                                let mut lookup_name = id.name.clone();
                                let outer_expected_callable_arity = self
                                    .infer_expected_from_outer_consumer_next_arg(
                                        &stack,
                                        stack.len(),
                                        0,
                                    )
                                    .and_then(|expected_ty| {
                                        let resolved_target = self.ctx.resolve(expected_ty);
                                        match self.ctx.get(resolved_target) {
                                            TypeKind::Function { params, .. } => Some(
                                                if params.len() == 1
                                                    && matches!(
                                                        self.ctx
                                                            .get(self.ctx.resolve_id(params[0])),
                                                        TypeKind::Unit
                                                    )
                                                {
                                                    0
                                                } else {
                                                    params.len()
                                                },
                                            ),
                                            _ => None,
                                        }
                                    });
                                let mut bindings =
                                    if let Some((member, qualified)) = &qualified_bindings {
                                        lookup_name = member.clone();
                                        qualified.clone()
                                    } else {
                                        self.lookup_all_unqualified_any_defined(id)
                                            .into_iter()
                                            .cloned()
                                            .collect()
                                    };
                                if bindings.is_empty()
                                    && qualified_bindings.is_none()
                                    && !self.import_resolution.has_qualified_targets()
                                {
                                    if let Some((ns, member)) = parse_variant_name(&id.name) {
                                        if !self.enums.contains_key(ns)
                                            && !self.traits.contains_key(ns)
                                        {
                                            let member_id = Ident {
                                                name: member.to_string(),
                                                span: id.span,
                                            };
                                            let alt = self
                                                .lookup_all_unqualified_any_defined(&member_id)
                                                .into_iter()
                                                .cloned()
                                                .collect::<Vec<_>>();
                                            if !alt.is_empty() {
                                                lookup_name = member.to_string();
                                                bindings = alt;
                                            }
                                        }
                                    }
                                }
                                if !type_args.is_empty() {
                                    bindings.retain(|binding| {
                                        let ty = self.ctx.resolve_id(binding.ty);
                                        match self.ctx.get(ty) {
                                            TypeKind::Function { type_params, .. } => {
                                                type_params.len() == type_args.len()
                                            }
                                            _ => false,
                                        }
                                    });
                                }
                                if qualified_bindings.is_some() && bindings.len() == 1 {
                                    let binding = bindings.remove(0);
                                    let mut explicit_args = Vec::new();
                                    if matches!(binding.kind, BindingKind::Func { .. }) {
                                        for arg_expr in type_args {
                                            explicit_args.push(type_from_expr(
                                                self.ctx,
                                                self.labels,
                                                arg_expr,
                                            ));
                                        }
                                    } else if !type_args.is_empty() {
                                        self.diagnostics.push(
                                            Diagnostic::error(
                                                "type arguments are not allowed for variables",
                                                id.span,
                                            )
                                            .with_id(DiagnosticId::TypeVariableTypeArgsNotAllowed),
                                        );
                                    }
                                    let ty = binding.ty;
                                    let hir_kind = match &binding.kind {
                                        BindingKind::Func { symbol, .. } => {
                                            HirExprKind::FnValue(symbol.clone())
                                        }
                                        _ => HirExprKind::Var(lookup_name.clone()),
                                    };
                                    stack.push(StackEntry {
                                        ty,
                                        expr: HirExpr {
                                            ty,
                                            kind: hir_kind,
                                            span: id.span,
                                        },
                                        type_args: explicit_args,
                                        assign: None,
                                        auto_call: !*forced_value,
                                    });
                                    last_expr = Some(stack.last().unwrap().expr.clone());
                                    continue;
                                }
                                if !bindings.is_empty() {
                                    let call_name = if qualified_bindings.is_some() {
                                        id.name.clone()
                                    } else {
                                        lookup_name.clone()
                                    };
                                    let callable_overload_count = bindings
                                        .iter()
                                        .filter(|b| matches!(b.kind, BindingKind::Func { .. }))
                                        .count();
                                    let overloaded_callable_only = callable_overload_count > 1
                                        && bindings
                                            .iter()
                                            .all(|b| matches!(b.kind, BindingKind::Func { .. }));
                                    if overloaded_callable_only
                                        && !*forced_value
                                        && type_args.is_empty()
                                    {
                                        let remaining_items =
                                            expr.items.len().saturating_sub(idx + 1);
                                        let mut arities: Vec<usize> = bindings
                                            .iter()
                                            .filter_map(|b| match b.kind {
                                                BindingKind::Func { arity, .. } => Some(arity),
                                                _ => None,
                                            })
                                            .collect();
                                        arities.sort_unstable();
                                        arities.dedup();
                                        let inferred_arity = outer_expected_callable_arity
                                            .filter(|a| arities.contains(a))
                                            .or_else(|| {
                                                if matches!(
                                                    expr.items.get(idx + 1),
                                                    Some(PrefixItem::Pipe(_))
                                                ) && arities.contains(&0)
                                                {
                                                    return Some(0);
                                                }
                                                arities
                                                    .iter()
                                                    .copied()
                                                    .filter(|a| *a <= remaining_items)
                                                    .max()
                                            })
                                            .or_else(|| arities.first().copied())
                                            .unwrap_or(0);
                                        let mut params = Vec::new();
                                        for _ in 0..inferred_arity {
                                            params.push(self.ctx.fresh_var(None));
                                        }
                                        let result = self.ctx.fresh_var(None);
                                        let ty = self.ctx.function(
                                            Vec::new(),
                                            params,
                                            result,
                                            Effect::Pure,
                                        );
                                        stack.push(StackEntry {
                                            ty,
                                            expr: HirExpr {
                                                ty,
                                                kind: HirExprKind::Var(call_name.clone()),
                                                span: id.span,
                                            },
                                            type_args: Vec::new(),
                                            assign: None,
                                            auto_call: true,
                                        });
                                        last_expr = Some(stack.last().unwrap().expr.clone());
                                    } else if let Some(binding) = bindings
                                        .iter()
                                        .cloned()
                                        .find(|b| matches!(b.kind, BindingKind::Var))
                                    {
                                        if *forced_value {
                                            self.diagnostics.push(Diagnostic::error(
                                            "only callable symbols can be referenced with '@'",
                                            id.span,
                                        ).with_id(DiagnosticId::TypeAtRequiresCallable));
                                            return None;
                                        }
                                        if !type_args.is_empty() {
                                            self.diagnostics.push(
                                                Diagnostic::error(
                                                    "type arguments are not allowed for variables",
                                                    id.span,
                                                )
                                                .with_id(
                                                    DiagnosticId::TypeVariableTypeArgsNotAllowed,
                                                ),
                                            );
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
                                        if crate::log::is_verbose()
                                            && matches!(
                                                lookup_name.as_str(),
                                                "A" | "use_a" | "DefaultHash32" | "new" | "must_hm"
                                            )
                                        {
                                            typecheck_log!(
                                                "push value {} ty={} auto_call={}",
                                                lookup_name,
                                                self.ctx.type_to_string(ty),
                                                !*forced_value
                                            );
                                        }
                                        last_expr = Some(stack.last().unwrap().expr.clone());
                                    } else {
                                        let expected_callable_arity = pending_ascription
                                            .and_then(|(target_ty, base_len)| {
                                                if stack.len() == base_len {
                                                    let resolved_target =
                                                        self.ctx.resolve(target_ty);
                                                    match self.ctx.get(resolved_target) {
                                                        TypeKind::Function { params, .. } => Some(
                                                            if params.len() == 1
                                                                && matches!(
                                                                    self.ctx.get(
                                                                        self.ctx
                                                                            .resolve_id(params[0])
                                                                    ),
                                                                    TypeKind::Unit
                                                                )
                                                            {
                                                                0
                                                            } else {
                                                                params.len()
                                                            },
                                                        ),
                                                        _ => None,
                                                    }
                                                } else {
                                                    None
                                                }
                                            })
                                            .or(outer_expected_callable_arity);
                                        if let Some(exp_arity) = expected_callable_arity {
                                            let allow_fnvalue_selection =
                                                *forced_value || idx + 1 >= expr.items.len();
                                            if allow_fnvalue_selection {
                                                let mut arity_candidates: Vec<&Binding> = bindings
                                            .iter()
                                            .filter(|b| {
                                                matches!(
                                                    b.kind,
                                                    BindingKind::Func { arity, .. } if arity == exp_arity
                                                )
                                            })
                                            .collect();
                                                if arity_candidates.len() == 1 {
                                                    let binding = arity_candidates.remove(0);
                                                    if *forced_value {
                                                        if let BindingKind::Func {
                                                            captures, ..
                                                        } = &binding.kind
                                                        {
                                                            if !captures.is_empty() {
                                                                self.diagnostics.push(Diagnostic::error(
                                                            "capturing function cannot be used as a function value yet",
                                                            id.span,
                                                        ).with_id(DiagnosticId::TypeCapturingFunctionValueUnsupported));
                                                                return None;
                                                            }
                                                        }
                                                    }
                                                    let mut explicit_args = Vec::new();
                                                    if !type_args.is_empty() {
                                                        for arg_expr in type_args {
                                                            explicit_args.push(type_from_expr(
                                                                self.ctx,
                                                                self.labels,
                                                                arg_expr,
                                                            ));
                                                        }
                                                    }
                                                    let ty = binding.ty;
                                                    let fn_symbol = match &binding.kind {
                                                        BindingKind::Func { symbol, .. } => {
                                                            symbol.clone()
                                                        }
                                                        _ => lookup_name.clone(),
                                                    };
                                                    stack.push(StackEntry {
                                                        ty,
                                                        expr: HirExpr {
                                                            ty,
                                                            // 期待関数型で一意に選べた過負荷関数は
                                                            // ここで関数値として確定させる。
                                                            kind: HirExprKind::FnValue(fn_symbol),
                                                            span: id.span,
                                                        },
                                                        type_args: explicit_args,
                                                        assign: None,
                                                        auto_call: false,
                                                    });
                                                    last_expr =
                                                        Some(stack.last().unwrap().expr.clone());
                                                    continue;
                                                } else if arity_candidates.len() > 1 {
                                                    self.diagnostics.push(
                                                        Diagnostic::error(
                                                            "ambiguous overload",
                                                            id.span,
                                                        )
                                                        .with_id(
                                                            DiagnosticId::TypeAmbiguousOverload,
                                                        ),
                                                    );
                                                    return None;
                                                }
                                            }
                                        }
                                        let mut has_pure = false;
                                        let mut has_impure = false;
                                        let mut arity = None;
                                        for b in &bindings {
                                            if let BindingKind::Func {
                                                effect: e,
                                                arity: a,
                                                ..
                                            } = b.kind
                                            {
                                                match e {
                                                    Effect::Pure => has_pure = true,
                                                    Effect::Impure => has_impure = true,
                                                }
                                                if arity.is_none() {
                                                    arity = Some(a);
                                                }
                                            }
                                        }
                                        let arity = arity.unwrap_or(0);
                                        let effect = if has_pure || !has_impure {
                                            Effect::Pure
                                        } else {
                                            Effect::Impure
                                        };
                                        let has_captures = bindings.iter().any(|b| {
                                        matches!(
                                            &b.kind,
                                            BindingKind::Func { captures, .. } if !captures.is_empty()
                                        )
                                    });
                                        if *forced_value && has_captures {
                                            self.diagnostics.push(Diagnostic::error(
                                            "capturing function cannot be used as a function value yet",
                                            id.span,
                                        ).with_id(DiagnosticId::TypeCapturingFunctionValueUnsupported));
                                            return None;
                                        }
                                        let mut explicit_args = Vec::new();
                                        if !type_args.is_empty() {
                                            for arg_expr in type_args {
                                                explicit_args.push(type_from_expr(
                                                    self.ctx,
                                                    self.labels,
                                                    arg_expr,
                                                ));
                                            }
                                        }
                                        let mut params = Vec::new();
                                        for _ in 0..arity {
                                            params.push(self.ctx.fresh_var(None));
                                        }
                                        let result = self.ctx.fresh_var(None);
                                        let ty =
                                            self.ctx.function(Vec::new(), params, result, effect);
                                        stack.push(StackEntry {
                                            ty,
                                            expr: HirExpr {
                                                ty,
                                                kind: if *forced_value {
                                                    HirExprKind::FnValue(call_name.clone())
                                                } else {
                                                    HirExprKind::Var(call_name.clone())
                                                },
                                                span: id.span,
                                            },
                                            type_args: explicit_args,
                                            assign: None,
                                            auto_call: true,
                                        });
                                        if crate::log::is_verbose()
                                            && matches!(
                                                lookup_name.as_str(),
                                                "A" | "use_a" | "DefaultHash32" | "new" | "must_hm"
                                            )
                                        {
                                            typecheck_log!(
                                                "push callable {} ty={} auto_call=true",
                                                lookup_name,
                                                self.ctx.type_to_string(ty)
                                            );
                                        }
                                        last_expr = Some(stack.last().unwrap().expr.clone());
                                    }
                                } else if let Some((trait_name, method_name)) =
                                    parse_variant_name(&id.name)
                                {
                                    if let Some(trait_info) = self.traits.get(trait_name) {
                                        if !type_args.is_empty() {
                                            self.diagnostics.push(Diagnostic::error(
                                                "type arguments are not supported for trait methods yet",
                                                id.span,
                                            ).with_id(DiagnosticId::TypeTraitMethodTypeArgsNotSupported));
                                            return None;
                                        }
                                        if let Some(sig) = trait_info.methods.get(method_name) {
                                            let applied_trait_name = self
                                                .infer_trait_application_name(
                                                    trait_name,
                                                    trait_info,
                                                    *sig,
                                                    &[],
                                                    None,
                                                );
                                            let method_self = self
                                                .infer_unique_type_param_for_trait(
                                                    &applied_trait_name,
                                                )
                                                .unwrap_or_else(|| {
                                                    self.ctx.fresh_var(Some(String::from("Self")))
                                                });
                                            let mut mapping = BTreeMap::new();
                                            mapping.insert(
                                                self.ctx.resolve_id(trait_info.self_ty),
                                                method_self,
                                            );
                                            let inst_ty = self.ctx.substitute(*sig, &mapping);
                                            stack.push(StackEntry {
                                                ty: inst_ty,
                                                expr: HirExpr {
                                                    ty: inst_ty,
                                                    kind: if *forced_value {
                                                        HirExprKind::FnValue(id.name.clone())
                                                    } else {
                                                        HirExprKind::Var(id.name.clone())
                                                    },
                                                    span: id.span,
                                                },
                                                type_args: vec![method_self],
                                                assign: None,
                                                auto_call: !*forced_value,
                                            });
                                            last_expr = Some(stack.last().unwrap().expr.clone());
                                        } else {
                                            self.diagnostics.push(
                                                Diagnostic::error(
                                                    format!(
                                                        "unknown method '{}' for trait '{}'",
                                                        method_name, trait_name
                                                    ),
                                                    id.span,
                                                )
                                                .with_id(DiagnosticId::TypeTraitMethodNotFound),
                                            );
                                            return None;
                                        }
                                    } else {
                                        self.diagnostics.push(
                                            Diagnostic::error("undefined identifier", id.span)
                                                .with_id(DiagnosticId::TypeUndefinedIdentifier),
                                        );
                                    }
                                } else {
                                    self.diagnostics.push(
                                        Diagnostic::error("undefined identifier", id.span)
                                            .with_id(DiagnosticId::TypeUndefinedIdentifier),
                                    );
                                }
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
                        let ty = if let Some(b) = self.env.lookup_current_value(&name.name) {
                            if b.no_shadow && b.span != name.span {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "cannot shadow non-shadowable symbol '{}'",
                                            name.name
                                        ),
                                        name.span,
                                    )
                                    .with_id(DiagnosticId::TypeNoShadowViolation),
                                );
                                self.diagnostics.push(
                                    Diagnostic::error("non-shadowable declaration is here", b.span)
                                        .with_id(DiagnosticId::TypeNoShadowViolation)
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
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "cannot shadow non-shadowable symbol '{}'",
                                            name.name
                                        ),
                                        name.span,
                                    )
                                    .with_id(DiagnosticId::TypeNoShadowViolation),
                                );
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "non-shadowable declaration is here",
                                        blocked.span,
                                    )
                                    .with_id(DiagnosticId::TypeNoShadowViolation)
                                    .with_secondary_label(name.span, Some("shadow attempt".into())),
                                );
                                return None;
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
                        if let Some((binding, scope_index)) =
                            self.env.lookup_value_with_scope(&name.name)
                        {
                            if !binding.mutable {
                                self.diagnostics.push(
                                    Diagnostic::error("cannot set immutable variable", name.span)
                                        .with_id(DiagnosticId::TypeImmutableMutation),
                                );
                            }
                            let effect = if scope_index == 0 {
                                Effect::Impure
                            } else {
                                Effect::Pure
                            };
                            let func_ty = self.ctx.function(
                                Vec::new(),
                                vec![binding.ty],
                                self.ctx.unit(),
                                effect,
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
                            self.diagnostics.push(
                                Diagnostic::error("undefined variable", name.span)
                                    .with_id(DiagnosticId::TypeUndefinedVariable),
                            );
                        }
                    }
                    Symbol::AddrOf { span, mutable } => {
                        if crate::log::is_verbose() {
                            typecheck_log!("check_prefix: pushing AddrOf to stack");
                        }
                        let a = self.ctx.fresh_var(None);
                        let ref_a = self.ctx.reference(a, *mutable);
                        let func_ty = self.ctx.function(Vec::new(), vec![a], ref_a, Effect::Pure);
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var("&&addr_of".to_string()),
                                span: *span,
                            },
                            type_args: Vec::new(),
                            assign: Some(AssignKind::AddrOf(*mutable)),
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
                    let intrin_effect = intrinsic_effect(&intrin.name);
                    if matches!(self.current_effect, Effect::Pure)
                        && matches!(intrin_effect, Effect::Impure)
                        && !self.raw_memory_intrinsic_allowed(&intrin.name, *sp)
                    {
                        self.diagnostics.push(
                            Diagnostic::error("pure context cannot call impure function", *sp)
                                .with_id(DiagnosticId::TypePureCallsImpureFunction),
                        );
                        return None;
                    }

                    let mut type_args = Vec::new();
                    for t in &intrin.type_args {
                        type_args.push(type_from_expr(self.ctx, self.labels, t));
                    }

                    let mut args = Vec::new();
                    for arg in &intrin.args {
                        let mut arg_stack = Vec::new();
                        if let Some((hexpr, _)) = self.check_prefix(arg, 0, &mut arg_stack, None) {
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
                            self.diagnostics.push(
                                Diagnostic::error("callsite_span expects 1 type arg", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicTypeArgArityMismatch),
                            );
                            self.ctx.unit()
                        }
                    } else if intrin.name == "set_field" {
                        self.ctx.unit() // temporary, will continue below
                    } else if intrin.name == "unreachable" {
                        self.ctx.never()
                    } else if intrin.name == "i32_to_f32" {
                        self.ctx.f32()
                    } else if intrin.name == "i32_to_u8" {
                        self.ctx.u8()
                    } else if intrin.name == "i32_to_u32" {
                        self.ctx.lookup_named("u32").unwrap_or_else(|| {
                            self.ctx.register_named(
                                "u32".to_string(),
                                TypeKind::Named("u32".to_string()),
                            )
                        })
                    } else if intrin.name == "f32_to_i32" {
                        self.ctx.i32()
                    } else if intrin.name == "u8_to_i32" {
                        self.ctx.i32()
                    } else if intrin.name == "u32_to_i32" {
                        self.ctx.i32()
                    } else if intrin.name == "i64_to_u64" {
                        self.ctx.lookup_named("u64").unwrap_or_else(|| {
                            self.ctx.register_named(
                                "u64".to_string(),
                                TypeKind::Named("u64".to_string()),
                            )
                        })
                    } else if intrin.name == "u64_to_i64" {
                        self.ctx.lookup_named("i64").unwrap_or_else(|| {
                            self.ctx.register_named(
                                "i64".to_string(),
                                TypeKind::Named("i64".to_string()),
                            )
                        })
                    } else if intrin.name == "reinterpret_i32_f32" {
                        self.ctx.f32()
                    } else if intrin.name == "reinterpret_f32_i32" {
                        self.ctx.i32()
                    } else if intrin.name == "str_addr" {
                        self.ctx.i32()
                    } else if intrin.name == "str_from_addr_unchecked" {
                        self.ctx.str()
                    } else if intrin.name == "get_field" {
                        self.ctx.fresh_var(None)
                    } else if intrin.name == "get_field_ref" {
                        self.ctx.fresh_var(None)
                    } else if intrin.name == "set_field" {
                        self.ctx.unit()
                    } else {
                        self.diagnostics.push(
                            Diagnostic::error("unknown intrinsic", *sp)
                                .with_id(DiagnosticId::TypeUnknownIntrinsic),
                        );
                        self.ctx.unit()
                    };

                    if intrin.name == "get_field" {
                        let obj = args[0].clone();
                        let idx = &args[1];
                        let res = match &idx.kind {
                            HirExprKind::LiteralI32(val) => self.resolve_field_access(
                                obj.ty,
                                FieldIdx::Index(*val as usize),
                                *sp,
                            ),
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
                                            },
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
                    } else if intrin.name == "get_field_ref" {
                        let obj = args[0].clone();
                        let idx = &args[1];
                        let resolved_obj_ty = self.ctx.resolve(obj.ty);
                        let base_ty = match self.ctx.get(resolved_obj_ty) {
                            TypeKind::Reference(inner, _) => inner,
                            _ => {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "get_field_ref expects a reference to a composite value",
                                        obj.span,
                                    )
                                    .with_id(DiagnosticId::TypeInvalidFieldAccess),
                                );
                                self.ctx.never()
                            }
                        };
                        let res = match &idx.kind {
                            HirExprKind::LiteralI32(val) => self.resolve_field_access(
                                base_ty,
                                FieldIdx::Index(*val as usize),
                                *sp,
                            ),
                            HirExprKind::LiteralStr(sid) => {
                                let name = self.string_table.get(*sid).unwrap().clone();
                                self.resolve_field_access(base_ty, FieldIdx::Name(name), *sp)
                            }
                            _ => None,
                        };
                        if let Some((f_ty, offset)) = res {
                            let ref_ty = self.ctx.reference(f_ty, false);
                            let _ = self.ctx.unify(ty, ref_ty);
                            let addr_expr = if offset == 0 {
                                obj
                            } else {
                                HirExpr {
                                    ty: ref_ty,
                                    kind: HirExprKind::Intrinsic {
                                        name: "add".to_string(),
                                        type_args: vec![self.ctx.i32()],
                                        args: vec![
                                            obj,
                                            HirExpr {
                                                ty: self.ctx.i32(),
                                                kind: HirExprKind::LiteralI32(offset as i32),
                                                span: idx.span,
                                            },
                                        ],
                                    },
                                    span: *sp,
                                }
                            };
                            let hexpr = HirExpr {
                                ty: ref_ty,
                                kind: addr_expr.kind,
                                span: *sp,
                            };
                            stack.push(StackEntry {
                                ty: ref_ty,
                                expr: hexpr.clone(),
                                type_args: Vec::new(),
                                assign: None,
                                auto_call: true,
                            });
                            last_expr = Some(hexpr);
                            continue;
                        }
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
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "type mismatch in set_field: expected {}, found {}",
                                            self.ctx.type_to_string(f_ty),
                                            self.ctx.type_to_string(val.ty)
                                        ),
                                        *sp,
                                    )
                                    .with_id(DiagnosticId::TypeAssignmentTypeMismatch),
                                );
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
                                            },
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
                        || intrin.name == "i32_to_u32"
                    {
                        if args.len() != 1 {
                            self.diagnostics.push(
                                Diagnostic::error("intrinsic expects 1 argument", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicArgArityMismatch),
                            );
                        } else if let Err(_) = self.ctx.unify(args[0].ty, self.ctx.i32()) {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "intrinsic argument type mismatch (expected i32)",
                                    *sp,
                                )
                                .with_id(DiagnosticId::TypeIntrinsicArgTypeMismatch),
                            );
                        }
                    } else if intrin.name == "f32_to_i32" || intrin.name == "reinterpret_f32_i32" {
                        if args.len() != 1 {
                            self.diagnostics.push(
                                Diagnostic::error("intrinsic expects 1 argument", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicArgArityMismatch),
                            );
                        } else if let Err(_) = self.ctx.unify(args[0].ty, self.ctx.f32()) {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "intrinsic argument type mismatch (expected f32)",
                                    *sp,
                                )
                                .with_id(DiagnosticId::TypeIntrinsicArgTypeMismatch),
                            );
                        }
                    } else if intrin.name == "u8_to_i32" || intrin.name == "u32_to_i32" {
                        if args.len() != 1 {
                            self.diagnostics.push(
                                Diagnostic::error("intrinsic expects 1 argument", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicArgArityMismatch),
                            );
                        } else {
                            let expected = if intrin.name == "u8_to_i32" {
                                self.ctx.u8()
                            } else {
                                self.ctx.lookup_named("u32").unwrap_or_else(|| {
                                    self.ctx.register_named(
                                        "u32".to_string(),
                                        TypeKind::Named("u32".to_string()),
                                    )
                                })
                            };
                            if let Err(_) = self.ctx.unify(args[0].ty, expected) {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "intrinsic argument type mismatch (expected {})",
                                            self.ctx.type_to_string(expected)
                                        ),
                                        *sp,
                                    )
                                    .with_id(DiagnosticId::TypeIntrinsicArgTypeMismatch),
                                );
                            }
                        }
                    } else if intrin.name == "i64_to_u64" || intrin.name == "u64_to_i64" {
                        if args.len() != 1 {
                            self.diagnostics.push(
                                Diagnostic::error("intrinsic expects 1 argument", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicArgArityMismatch),
                            );
                        } else {
                            let expected = if intrin.name == "i64_to_u64" {
                                self.ctx.lookup_named("i64").unwrap_or_else(|| {
                                    self.ctx.register_named(
                                        "i64".to_string(),
                                        TypeKind::Named("i64".to_string()),
                                    )
                                })
                            } else {
                                self.ctx.lookup_named("u64").unwrap_or_else(|| {
                                    self.ctx.register_named(
                                        "u64".to_string(),
                                        TypeKind::Named("u64".to_string()),
                                    )
                                })
                            };
                            if let Err(_) = self.ctx.unify(args[0].ty, expected) {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "intrinsic argument type mismatch (expected {})",
                                            self.ctx.type_to_string(expected)
                                        ),
                                        *sp,
                                    )
                                    .with_id(DiagnosticId::TypeIntrinsicArgTypeMismatch),
                                );
                            }
                        }
                    } else if intrin.name == "str_addr" || intrin.name == "str_from_addr_unchecked"
                    {
                        if args.len() != 1 {
                            self.diagnostics.push(
                                Diagnostic::error("intrinsic expects 1 argument", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicArgArityMismatch),
                            );
                        } else {
                            let expected = if intrin.name == "str_addr" {
                                self.ctx.str()
                            } else {
                                self.ctx.i32()
                            };
                            if let Err(_) = self.ctx.unify(args[0].ty, expected) {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "intrinsic argument type mismatch (expected {})",
                                            self.ctx.type_to_string(expected)
                                        ),
                                        *sp,
                                    )
                                    .with_id(DiagnosticId::TypeIntrinsicArgTypeMismatch),
                                );
                            }
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
                        if let Some((hexpr, _)) = self.check_prefix(elem, 0, &mut elem_stack, None)
                        {
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
                    if let Some((hexpr, _)) = self.check_prefix(inner, 0, &mut group_stack, None) {
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
                    let (blk, val_ty) = self.check_block(b, 0, true, None)?;
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
                        self.diagnostics.push(
                            Diagnostic::error(
                                "pipe already pending; consecutive |> not allowed",
                                *sp,
                            )
                            .with_id(DiagnosticId::TypePipeError),
                        );
                        continue;
                    }
                    // Don't drain past any let/set binding on the stack.
                    let default_pipe_base = stack
                        .iter()
                        .rposition(|e| e.assign.is_some())
                        .map(|p| base_depth.max(p + 1))
                        .unwrap_or(base_depth);
                    let pipe_base =
                        self.pipe_pending_base(stack.as_slice(), &open_calls, default_pipe_base);
                    if stack.len() == pipe_base {
                        self.diagnostics.push(
                            Diagnostic::error("pipe requires a value on the stack", *sp)
                                .with_id(DiagnosticId::TypePipeError),
                        );
                        continue;
                    }
                    let pending = stack.drain(pipe_base..).collect::<Vec<_>>();
                    last_expr = pending.last().map(|se| se.expr.clone());
                    pipe_pending = Some(pending);
                    seen_pipe = true;
                }
            }

            if !matches!(item, PrefixItem::Pipe(_) | PrefixItem::TypeAnnotation(_, _)) {
                if let Some(pending) = pipe_pending.take() {
                    // The last pushed element should be a callable (function type)
                    if let Some(top) = stack.last() {
                        if top.auto_call
                            && matches!(self.ctx.get(top.ty), TypeKind::Function { .. })
                        {
                            let Some(lowered_val) = self.reduce_pipe_pending_segment_with_target(
                                pending,
                                top,
                                pending_ascription.map(|(target, _)| target),
                            ) else {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "pipe left-hand side did not reduce to a single value",
                                        expr.span,
                                    )
                                    .with_id(DiagnosticId::TypePipeError),
                                );
                                continue;
                            };
                            // pipe では「関数を積んだ直後に引数を注入」するため、
                            // 通常の末尾関数追跡だけでは open_calls に載らない。
                            let func_idx = stack.len() - 1;
                            if !open_calls.iter().any(|&i| i == func_idx) {
                                open_calls.push(func_idx);
                            }
                            stack.push(lowered_val);
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        } else {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "pipe target must be a callable expression",
                                    expr.span,
                                )
                                .with_id(DiagnosticId::TypePipeError),
                            );
                            stack.extend(pending);
                        }
                    } else {
                        self.diagnostics.push(
                            Diagnostic::error("pipe target missing", expr.span)
                                .with_id(DiagnosticId::TypePipeError),
                        );
                        stack.extend(pending);
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
            if !next_is_pipe
                || is_local_pending_ascription(stack.as_slice(), pending_ascription, base_depth)
            {
                try_apply_pending_ascription(self, stack, &mut pending_ascription);
            }

            let has_more_items = idx + 1 < expr.items.len();
            let defer_unresolved_overload = pipe_pending.is_none()
                && has_more_items
                && (0..stack.len()).rev().any(|i| {
                    let entry = &stack[i];
                    let rty = self.ctx.resolve_id(entry.ty);
                    entry.auto_call
                        && matches!(self.ctx.get(rty), TypeKind::Function { .. })
                        && self.unresolved_overloaded_entry_has_larger_arity(stack, i)
                });
            let delay_overloaded_nullary = pipe_pending.is_none()
                && stack.len() == base_depth + 1
                && stack
                    .last()
                    .map(|entry| {
                        if !self.is_unresolved_overloaded_callable_entry(entry) {
                            return false;
                        }
                        let rty = self.ctx.resolve_id(entry.ty);
                        let TypeKind::Function { params, .. } = self.ctx.get(rty) else {
                            return false;
                        };
                        self.user_visible_arity(&entry.expr, &params) == 0
                            && (next_is_pipe || has_more_items)
                    })
                    .unwrap_or(false);
            if !delay_overloaded_nullary && !defer_unresolved_overload {
                let mut pending_base = pending_ascription.map(|(_, base)| base);
                let mut pipe_guard = false;
                let reduction_expected = if next_is_pipe && seen_pipe {
                    pending_ascription.filter(|pending| {
                        is_local_pending_ascription(stack.as_slice(), Some(*pending), base_depth)
                    })
                } else {
                    pending_ascription
                };
                if next_is_pipe {
                    if let Some(assign_pos) = stack.iter().rposition(|e| e.assign.is_some()) {
                        let guard_pos = assign_pos + 1;
                        pending_base =
                            Some(pending_base.map_or(guard_pos, |base| base.max(guard_pos)));
                        pipe_guard = true;
                    }
                }
                if let Some(base_len) = pending_base {
                    self.reduce_calls_guarded(stack, &mut open_calls, base_len, reduction_expected);
                } else {
                    self.reduce_calls(stack, &mut open_calls, reduction_expected);
                }
                // std::eprintln!("  Stack after reduce: {:?}", stack.iter().map(|e| self.ctx.type_to_string(e.ty)).collect::<Vec<_>>());

                // Try applying pending ascription after call reduction.
                if !next_is_pipe
                    || is_local_pending_ascription(stack.as_slice(), pending_ascription, base_depth)
                {
                    try_apply_pending_ascription(self, stack, &mut pending_ascription);
                }

                if pending_base.is_some() && pending_ascription.is_none() && !pipe_guard {
                    self.reduce_calls(stack, &mut open_calls, reduction_expected);
                }
            }
        }

        if pipe_pending.is_some() {
            self.diagnostics.push(
                Diagnostic::error("pipe has no target", expr.span)
                    .with_id(DiagnosticId::TypePipeError),
            );
        }

        let leading_let = matches!(
            expr.items.first(),
            Some(PrefixItem::Symbol(Symbol::Let { .. }))
        );
        if leading_let {
            let mut pending_base = pending_ascription.map(|(_, base)| base);
            let mut open_calls: Vec<usize> = Vec::new();
            if let Some(base_len) = pending_base {
                self.reduce_calls_guarded(stack, &mut open_calls, base_len, pending_ascription);
            } else {
                self.reduce_calls(stack, &mut open_calls, pending_ascription);
            }
            try_apply_pending_ascription(self, stack, &mut pending_ascription);
            pending_base = pending_ascription.map(|(_, base)| base);
            if let Some(base_len) = pending_base {
                self.reduce_calls_guarded(stack, &mut open_calls, base_len, pending_ascription);
            } else {
                self.reduce_calls(stack, &mut open_calls, pending_ascription);
            }
        }
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
            if crate::log::is_verbose() {
                let tys = stack
                    .iter()
                    .map(|e| self.ctx.type_to_string(e.ty))
                    .collect::<Vec<_>>()
                    .join(", ");
                typecheck_log!("prefix final extras before trim [{}]", tys);
            }
            for _ in 0..extras {
                stack.pop();
            }
            dropped = true;
        }

        let result_expr = if leading_let && stack.len() >= base_depth + 2 {
            stack[base_depth + 1].expr.clone()
        } else if stack.len() == base_depth + 1 {
            if leading_let {
                stack.last().unwrap().expr.clone()
            } else {
                let top = stack.last_mut().unwrap();
                let placeholder = HirExpr {
                    ty: top.expr.ty,
                    kind: HirExprKind::Unit,
                    span: top.expr.span,
                };
                core::mem::replace(&mut top.expr, placeholder)
            }
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
                            if let Some(le) = last_expr.clone() {
                                le
                            } else {
                                result_expr.clone()
                            }
                        }
                        HirExprKind::Block(blk) => {
                            // Detect `if:` layout: block with exactly 3 lines and the
                            // original prefix contained an `if` symbol. In that case
                            // synthesize an `If` node from the three lines.
                            if blk.lines.len() == 3
                                && expr
                                    .items
                                    .iter()
                                    .any(|it| matches!(it, PrefixItem::Symbol(Symbol::If(_))))
                            {
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
            match self.char_literal_context_type(top, target) {
                Some(Ok(resolved)) => {
                    top.ty = resolved;
                    top.expr.ty = resolved;
                    return;
                }
                Some(Err(())) => {
                    self.diagnostics.push(
                        Diagnostic::error("char literal does not fit in u8", span)
                            .with_id(DiagnosticId::TypeAnnotationMismatch),
                    );
                    return;
                }
                None => {}
            }
            if let Err(_) = self.ctx.unify(top.ty, target) {
                let actual_ty = self.ctx.type_to_string(top.ty);
                let expected_ty = self.ctx.type_to_string(target);
                self.diagnostics.push(
                    Diagnostic::error(
                        format!(
                            "type annotation mismatch (expected {}, got {})",
                            expected_ty, actual_ty
                        ),
                        span,
                    )
                    .with_id(DiagnosticId::TypeAnnotationMismatch),
                );
            } else {
                let resolved = self.ctx.resolve_id(target);
                top.ty = resolved;
                top.expr.ty = resolved;
            }
        }
    }

    fn char_literal_value(&self, entry: &StackEntry) -> Option<i32> {
        if !self.ctx.same_type(entry.ty, self.ctx.char()) {
            return None;
        }
        match &entry.expr.kind {
            HirExprKind::LiteralI32(value) => Some(*value),
            _ => None,
        }
    }

    fn char_literal_context_type(
        &self,
        entry: &StackEntry,
        target: TypeId,
    ) -> Option<Result<TypeId, ()>> {
        let value = self.char_literal_value(entry)?;
        if self.ctx.same_type(target, self.ctx.i32()) {
            return Some(Ok(self.ctx.resolve_id(target)));
        }
        if self.ctx.same_type(target, self.ctx.u8()) {
            return Some(
                (0..=255)
                    .contains(&value)
                    .then(|| self.ctx.resolve_id(target))
                    .ok_or(()),
            );
        }
        None
    }

    fn char_literal_matches_context(&self, entry: &StackEntry, target: TypeId) -> bool {
        matches!(self.char_literal_context_type(entry, target), Some(Ok(_)))
    }

    fn stack_entry_is_open_call(&mut self, entry: &StackEntry) -> bool {
        let rty = self.ctx.resolve(entry.ty);
        entry.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. })
    }

    fn rebuild_open_calls(
        &mut self,
        stack: &[StackEntry],
        open_calls: &mut Vec<usize>,
        min_func_pos: usize,
    ) {
        open_calls.clear();
        for i in min_func_pos..stack.len() {
            if self.stack_entry_is_open_call(&stack[i]) {
                open_calls.push(i);
            }
        }
    }

    fn next_reducible_call_pos(
        &mut self,
        stack: &[StackEntry],
        open_calls: &mut Vec<usize>,
        min_func_pos: usize,
    ) -> Option<usize> {
        if open_calls.is_empty() {
            self.rebuild_open_calls(stack, open_calls, min_func_pos);
        }
        let mut cursor = open_calls.len();
        while cursor > 0 {
            cursor -= 1;
            let i = open_calls[cursor];
            if i < min_func_pos || i >= stack.len() || !self.stack_entry_is_open_call(&stack[i]) {
                open_calls.remove(cursor);
                continue;
            }
            if self.should_defer_overloaded_nullary_entry(stack, i) {
                continue;
            }
            return Some(i);
        }
        None
    }

    fn update_open_calls_after_reduction(
        &mut self,
        stack: &[StackEntry],
        open_calls: &mut Vec<usize>,
        func_pos: usize,
        args_to_take: usize,
    ) {
        let removed_end = func_pos + 1 + args_to_take;
        let first_removed = open_calls.partition_point(|&i| i < func_pos);
        let first_after_removed = open_calls.partition_point(|&i| i < removed_end);
        open_calls.drain(first_removed..first_after_removed);
        for i in &mut open_calls[first_removed..] {
            *i = i.saturating_sub(args_to_take);
        }
        if func_pos < stack.len() && self.stack_entry_is_open_call(&stack[func_pos]) {
            open_calls.insert(first_removed, func_pos);
        }
        open_calls.dedup();
    }

    fn call_reduction_state_key(&self, stack: &[StackEntry]) -> String {
        let mut out = String::new();
        for entry in stack {
            out.push_str(&self.ctx.type_to_string(entry.ty));
            out.push(':');
            match &entry.expr.kind {
                HirExprKind::Var(name) => {
                    out.push_str("var:");
                    out.push_str(name);
                }
                HirExprKind::FnValue(name) => {
                    out.push_str("fn:");
                    out.push_str(name);
                }
                HirExprKind::Call { callee, args } => {
                    out.push_str("call:");
                    out.push_str(&format!("{:?}/{}", callee, args.len()));
                }
                HirExprKind::CallIndirect { args, .. } => {
                    out.push_str("call_indirect:");
                    out.push_str(&args.len().to_string());
                }
                _ => out.push_str("expr"),
            }
            out.push('|');
        }
        out
    }

    fn reduce_calls_from(
        &mut self,
        stack: &mut Vec<StackEntry>,
        open_calls: &mut Vec<usize>,
        min_func_pos: usize,
        expected: Option<(TypeId, usize)>,
        label: &str,
    ) {
        let mut no_progress_states = BTreeSet::new();
        loop {
            dump!(
                "{}: stack=[{}]",
                label,
                stack
                    .iter()
                    .map(|e| match &e.expr.kind {
                        HirExprKind::Var(n) => n.clone(),
                        _ => "<expr>".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            );

            let Some(mut func_pos) = self.next_reducible_call_pos(stack, open_calls, min_func_pos)
            else {
                break;
            };
            if let Some(outer) = self.find_outer_function_consumer(stack, func_pos, min_func_pos) {
                func_pos = outer;
            }

            let available_args = stack.len().saturating_sub(func_pos + 1);
            let chosen_callable = match &stack[func_pos].expr.kind {
                HirExprKind::Var(name)
                    if stack[func_pos].type_args.is_empty()
                        && self.env.lookup_all_callables(name).len() > 1 =>
                {
                    self.choose_callable_type_by_available_arity(name, available_args)
                }
                HirExprKind::Var(name) if self.env.lookup_value(name).is_none() => {
                    self.choose_callable_type_by_available_arity(name, available_args)
                }
                _ => None,
            };
            let ty_for_infer = chosen_callable
                .map(|(_, ty)| ty)
                .unwrap_or(stack[func_pos].ty);
            let (inst_ty, _fresh_args) = if !stack[func_pos].type_args.is_empty() {
                (ty_for_infer, stack[func_pos].type_args.clone())
            } else {
                let (inst_ty, fresh_args, _mapping) = self.ctx.instantiate(ty_for_infer);
                (inst_ty, fresh_args)
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
                    self.diagnostics.push(
                        Diagnostic::error(
                            "call reduction found non-function after instantiation",
                            stack[func_pos].expr.span,
                        )
                        .with_id(DiagnosticId::TypeCallReductionLimitExceeded),
                    );
                    break;
                }
            };
            let needed_args = chosen_callable
                .map(|(arity, _)| arity)
                .unwrap_or_else(|| self.user_visible_arity(&stack[func_pos].expr, &params));
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
            let outer_expected =
                self.infer_expected_from_outer_consumer(stack, func_pos, min_func_pos);
            let expected_ret = expected_ret.or(outer_expected);

            let before_len = stack.len();
            let drained = stack
                .drain(func_pos..func_pos + 1 + args_to_take)
                .collect::<Vec<_>>();
            let mut drained = drained.into_iter();
            let Some(mut func_entry) = drained.next() else {
                break;
            };
            let args = drained.collect::<Vec<_>>();
            func_entry.ty = inst_ty;
            func_entry.expr.ty = inst_ty;
            let explicit_type_args = func_entry.type_args.clone();
            let debug_name = match &func_entry.expr.kind {
                HirExprKind::Var(name) => Some(name.clone()),
                _ => None,
            };
            if crate::log::is_verbose() {
                typecheck_log!(
                    "    Reducing {}: {} at pos {} with {} args, assign={:?}",
                    label,
                    self.ctx.type_to_string(inst_ty),
                    func_pos,
                    params.len(),
                    func_entry.assign
                );
                if label == "reduce_calls_guarded"
                    && matches!(
                        debug_name.as_deref(),
                        Some(
                            "get"
                                | "is_none"
                                | "must_hm"
                                | "make_hm"
                                | "new"
                                | "DefaultHash32"
                                | "A"
                                | "use_a"
                        )
                    )
                {
                    let before = stack
                        .iter()
                        .map(|e| self.ctx.type_to_string(e.ty))
                        .collect::<Vec<_>>()
                        .join(", ");
                    typecheck_log!("      stack before guarded apply [{}]", before);
                }
            }
            let applied = self.apply_function(
                func_entry,
                params,
                result,
                effect,
                args,
                explicit_type_args,
                expected_ret,
            );

            if let Some(val) = applied {
                if crate::log::is_verbose()
                    && label == "reduce_calls_guarded"
                    && matches!(
                        debug_name.as_deref(),
                        Some(
                            "get"
                                | "is_none"
                                | "must_hm"
                                | "make_hm"
                                | "new"
                                | "DefaultHash32"
                                | "A"
                                | "use_a"
                        )
                    )
                {
                    typecheck_log!("      guarded result {}", self.ctx.type_to_string(val.ty));
                }
                stack.insert(func_pos, val);
                self.update_open_calls_after_reduction(stack, open_calls, func_pos, args_to_take);
                if stack.len() >= before_len {
                    let state_key = self.call_reduction_state_key(stack);
                    if !no_progress_states.insert(state_key) {
                        let span = stack
                            .get(func_pos)
                            .map(|entry| entry.expr.span)
                            .unwrap_or_else(Span::dummy);
                        self.diagnostics.push(
                            Diagnostic::error("call reduction made no progress", span)
                                .with_id(DiagnosticId::TypeCallReductionLimitExceeded),
                        );
                        break;
                    }
                } else {
                    no_progress_states.clear();
                }
            } else {
                break;
            }
        }
    }

    fn reduce_calls(
        &mut self,
        stack: &mut Vec<StackEntry>,
        open_calls: &mut Vec<usize>,
        expected: Option<(TypeId, usize)>,
    ) {
        self.reduce_calls_from(stack, open_calls, 0, expected, "reduce_calls");
    }

    fn resolve_dotted_field_symbol(
        &mut self,
        id: &Ident,
        forced_value: bool,
    ) -> Option<StackEntry> {
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
        open_calls: &mut Vec<usize>,
        min_func_pos: usize,
        expected: Option<(TypeId, usize)>,
    ) {
        self.reduce_calls_from(
            stack,
            open_calls,
            min_func_pos,
            expected,
            "reduce_calls_guarded",
        );
    }

    /// マッチアームのバリアント名からスクルーティニーの期待型を推論する。
    /// 例: `Result::Ok`, `Result::Err` → `Result<fresh_A, fresh_B>` を返す。
    /// これにより `match with_capacity<.T> n:` のような式でオーバーロードが
    /// 解決できるようになる（スクルーティニーに期待型が伝播される）。
    fn infer_expected_type_from_match_arms(
        &mut self,
        arms: &[crate::ast::MatchArm],
    ) -> Option<TypeId> {
        for arm in arms {
            let variant_name = match &arm.pattern {
                MatchPattern::Variant { name, .. } => &name.name,
                _ => continue,
            };
            // "EnumName::VariantName" → "EnumName"
            let enum_name = if let Some(idx) = variant_name.rfind("::") {
                &variant_name[..idx]
            } else {
                continue;
            };
            let enum_info = self.enums.get(enum_name)?;
            let enum_ty = enum_info.ty;
            let type_params = enum_info.type_params.clone();
            return if type_params.is_empty() {
                Some(enum_ty)
            } else {
                let fresh_vars: Vec<TypeId> = type_params
                    .iter()
                    .map(|_| self.ctx.fresh_var(None))
                    .collect();
                Some(self.ctx.apply(enum_ty, fresh_vars))
            };
        }
        None
    }

    fn check_match_expr(&mut self, m: &MatchExpr) -> Option<(HirExpr, TypeId)> {
        // Infer the expected scrutinee type from the arm variant names.
        // e.g. `Result::Ok`, `Result::Err` → `Result<fresh_A, fresh_B>`
        // This allows disambiguation of overloaded scrutinee calls when the enum base
        // type is enough to select the right overload.
        let expected_scrut_ty = self.infer_expected_type_from_match_arms(&m.arms);
        // evaluate scrutinee
        let mut tmp_stack = Vec::new();
        if let Some((scrut_expr, _)) =
            self.check_prefix(&m.scrutinee, 0, &mut tmp_stack, expected_scrut_ty)
        {
            let scrut_ty = scrut_expr.ty;
            let resolved_ty = self.ctx.resolve(scrut_ty);
            if let Some(variants) = self.match_enum_variants_for_type(resolved_ty) {
                return self.check_enum_match_expr(m, scrut_expr, variants);
            }
            let scalar = match self.ctx.get(resolved_ty) {
                TypeKind::I32 => Some(ScalarMatchKind::I32),
                TypeKind::U8 => Some(ScalarMatchKind::U8),
                TypeKind::Bool => Some(ScalarMatchKind::Bool),
                TypeKind::Char => Some(ScalarMatchKind::Char),
                _ => None,
            };
            if let Some(kind) = scalar {
                return self.check_scalar_match_expr(m, scrut_expr, kind);
            }
            self.diagnostics.push(
                Diagnostic::error("match scrutinee must be an enum, bool, char, i32, or u8", m.span)
                    .with_id(DiagnosticId::TypeMatchScrutineeMustBeEnum),
            );
        }
        None
    }

    fn match_enum_variants_for_type(
        &mut self,
        resolved_ty: TypeId,
    ) -> Option<Vec<EnumVariantInfo>> {
        match self.ctx.get(resolved_ty) {
            TypeKind::Enum { variants, .. } => Some(variants.clone()),
            TypeKind::Apply { base, args } => {
                let base_ty = self.ctx.resolve(base);
                match self.ctx.get(base_ty) {
                    TypeKind::Enum {
                        type_params,
                        variants,
                        ..
                    } => {
                        if type_params.len() != args.len() {
                            return None;
                        }
                        let type_params = type_params.clone();
                        let variants = variants.clone();
                        let args = args.clone();
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
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn check_enum_match_expr(
        &mut self,
        m: &MatchExpr,
        scrut_expr: HirExpr,
        variants: Vec<EnumVariantInfo>,
    ) -> Option<(HirExpr, TypeId)> {
        let mut seen = alloc::collections::BTreeSet::new();
        let mut arms_hir = Vec::new();
        let mut result_ty: Option<TypeId> = None;
        for arm in &m.arms {
            let (variant, bind) = match &arm.pattern {
                MatchPattern::Variant { name, bind } => (name, bind),
                _ => {
                    self.diagnostics.push(
                        Diagnostic::error("unsupported match pattern for enum scrutinee", arm.span)
                            .with_id(DiagnosticId::TypeMatchPatternUnsupported),
                    );
                    continue;
                }
            };
            let arm_var_name = if let Some(pos) = variant.name.find("::") {
                &variant.name[pos + 2..]
            } else {
                &variant.name
            };
            if !seen.insert(arm_var_name.to_string()) {
                self.diagnostics.push(
                    Diagnostic::error("duplicate match arm", variant.span)
                        .with_id(DiagnosticId::TypeDuplicateMatchArm),
                );
                continue;
            }
            let var_info = variants.iter().find(|v| v.name == arm_var_name);
            if var_info.is_none() {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!("unknown enum variant '{}' in match", variant.name),
                        variant.span,
                    )
                    .with_id(DiagnosticId::TypeMatchUnknownVariant),
                );
                continue;
            }
            let var_info = var_info.unwrap();
            let bind_ty = bind.as_ref().and_then(|_| var_info.payload);
            self.env.push_scope();
            if let Some(bind) = bind {
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
                        span: bind.span,
                        kind: BindingKind::Var,
                    });
                } else {
                    self.diagnostics.push(
                        Diagnostic::error("variant has no payload to bind", bind.span)
                            .with_id(DiagnosticId::TypeMatchPayloadBindingInvalid),
                    );
                }
            }
            let (blk, val_ty) = self.check_block(&arm.body, 0, false, None)?;
            self.env.pop_scope();
            let body_ty = val_ty.unwrap_or(self.ctx.unit());
            self.check_match_arm_result_type(&mut result_ty, body_ty, arm.span);
            arms_hir.push(HirMatchArm {
                pattern: HirMatchPattern::Variant(variant.name.clone()),
                bind_local: bind.as_ref().map(|b| b.name.clone()),
                bind_ty,
                body: HirExpr {
                    ty: body_ty,
                    kind: HirExprKind::Block(blk),
                    span: arm.span,
                },
            });
        }
        for v in variants {
            if !seen.contains(&v.name) {
                self.diagnostics.push(
                    Diagnostic::error("non-exhaustive match", m.span)
                        .with_id(DiagnosticId::TypeNonExhaustiveMatch),
                );
                break;
            }
        }
        let rty = result_ty.unwrap_or(self.ctx.unit());
        Some((
            HirExpr {
                ty: rty,
                kind: HirExprKind::Match {
                    scrutinee: Box::new(scrut_expr),
                    arms: arms_hir,
                },
                span: m.span,
            },
            rty,
        ))
    }

    fn check_scalar_match_expr(
        &mut self,
        m: &MatchExpr,
        scrut_expr: HirExpr,
        kind: ScalarMatchKind,
    ) -> Option<(HirExpr, TypeId)> {
        let mut seen_i32 = alloc::collections::BTreeSet::new();
        let mut seen_true = false;
        let mut seen_false = false;
        let mut saw_wildcard = false;
        let mut arms_hir = Vec::new();
        let mut result_ty: Option<TypeId> = None;

        for (idx, arm) in m.arms.iter().enumerate() {
            let hir_pattern = match self.scalar_match_pattern(kind, arm, idx + 1 == m.arms.len()) {
                Some(p) => p,
                None => continue,
            };
            match &hir_pattern {
                HirMatchPattern::IntLiteral(v) => {
                    if !seen_i32.insert(*v) {
                        self.diagnostics.push(
                            Diagnostic::error("duplicate match arm", arm.span)
                                .with_id(DiagnosticId::TypeDuplicateMatchArm),
                        );
                        continue;
                    }
                }
                HirMatchPattern::BoolLiteral(true) => {
                    if seen_true {
                        self.diagnostics.push(
                            Diagnostic::error("duplicate match arm", arm.span)
                                .with_id(DiagnosticId::TypeDuplicateMatchArm),
                        );
                        continue;
                    }
                    seen_true = true;
                }
                HirMatchPattern::BoolLiteral(false) => {
                    if seen_false {
                        self.diagnostics.push(
                            Diagnostic::error("duplicate match arm", arm.span)
                                .with_id(DiagnosticId::TypeDuplicateMatchArm),
                        );
                        continue;
                    }
                    seen_false = true;
                }
                HirMatchPattern::Wildcard => {
                    if saw_wildcard {
                        self.diagnostics.push(
                            Diagnostic::error("duplicate match arm", arm.span)
                                .with_id(DiagnosticId::TypeDuplicateMatchArm),
                        );
                        continue;
                    }
                    saw_wildcard = true;
                }
                HirMatchPattern::Variant(_) => {}
            }

            let (blk, val_ty) = self.check_block(&arm.body, 0, false, None)?;
            let body_ty = val_ty.unwrap_or(self.ctx.unit());
            self.check_match_arm_result_type(&mut result_ty, body_ty, arm.span);
            arms_hir.push(HirMatchArm {
                pattern: hir_pattern,
                bind_local: None,
                bind_ty: None,
                body: HirExpr {
                    ty: body_ty,
                    kind: HirExprKind::Block(blk),
                    span: arm.span,
                },
            });
        }

        let exhaustive = match kind {
            ScalarMatchKind::Bool => saw_wildcard || (seen_true && seen_false),
            ScalarMatchKind::I32 | ScalarMatchKind::U8 | ScalarMatchKind::Char => saw_wildcard,
        };
        if !exhaustive {
            self.diagnostics.push(
                Diagnostic::error("non-exhaustive match", m.span)
                    .with_id(DiagnosticId::TypeNonExhaustiveMatch),
            );
        }
        let rty = result_ty.unwrap_or(self.ctx.unit());
        Some((
            HirExpr {
                ty: rty,
                kind: HirExprKind::Match {
                    scrutinee: Box::new(scrut_expr),
                    arms: arms_hir,
                },
                span: m.span,
            },
            rty,
        ))
    }

    fn scalar_match_pattern(
        &mut self,
        kind: ScalarMatchKind,
        arm: &MatchArm,
        is_last: bool,
    ) -> Option<HirMatchPattern> {
        match &arm.pattern {
            MatchPattern::IntLiteral { text, span } => {
                if kind == ScalarMatchKind::Bool {
                    self.diagnostics.push(
                        Diagnostic::error("integer literal match arm cannot match bool", *span)
                            .with_id(DiagnosticId::TypeMatchPatternUnsupported),
                    );
                    return None;
                }
                if kind == ScalarMatchKind::Char {
                    self.diagnostics.push(
                        Diagnostic::error("integer literal match arm cannot match char", *span)
                            .with_id(DiagnosticId::TypeMatchPatternUnsupported),
                    );
                    return None;
                }
                let Some(value) = parse_i32_literal(text) else {
                    self.diagnostics.push(
                        Diagnostic::error("invalid integer literal in match arm", *span)
                            .with_id(DiagnosticId::TypeMatchPatternUnsupported),
                    );
                    return None;
                };
                if kind == ScalarMatchKind::U8 && !(0..=255).contains(&value) {
                    self.diagnostics.push(
                        Diagnostic::error("u8 match literal must be in 0..=255", *span)
                            .with_id(DiagnosticId::TypeMatchPatternUnsupported),
                    );
                    return None;
                }
                Some(HirMatchPattern::IntLiteral(value))
            }
            MatchPattern::BoolLiteral { value, span } => {
                if kind != ScalarMatchKind::Bool {
                    self.diagnostics.push(
                        Diagnostic::error("bool literal match arm cannot match this scrutinee type", *span)
                            .with_id(DiagnosticId::TypeMatchPatternUnsupported),
                    );
                    return None;
                }
                Some(HirMatchPattern::BoolLiteral(*value))
            }
            MatchPattern::CharLiteral { value, span } => {
                if kind == ScalarMatchKind::Bool {
                    self.diagnostics.push(
                        Diagnostic::error("char literal match arm cannot match this scrutinee type", *span)
                            .with_id(DiagnosticId::TypeMatchPatternUnsupported),
                    );
                    return None;
                }
                if *value > i32::MAX as u32 {
                    self.diagnostics.push(
                        Diagnostic::error("char literal is outside current i32-backed codegen range", *span)
                            .with_id(DiagnosticId::TypeMatchPatternUnsupported),
                    );
                    return None;
                }
                if kind == ScalarMatchKind::U8 && *value > 255 {
                    self.diagnostics.push(
                        Diagnostic::error("char literal match arm does not fit in u8", *span)
                            .with_id(DiagnosticId::TypeMatchPatternUnsupported),
                    );
                    return None;
                }
                Some(HirMatchPattern::IntLiteral(*value as i32))
            }
            MatchPattern::Wildcard { span } => {
                if !is_last {
                    self.diagnostics.push(
                        Diagnostic::error("wildcard match arm must be last", *span)
                            .with_id(DiagnosticId::TypeMatchWildcardMustBeLast),
                    );
                }
                Some(HirMatchPattern::Wildcard)
            }
            MatchPattern::Variant { name, bind } => {
                if bind.is_some() {
                    self.diagnostics.push(
                        Diagnostic::error("literal match arms cannot bind payloads", name.span)
                            .with_id(DiagnosticId::TypeMatchPayloadBindingInvalid),
                    );
                    return None;
                }
                if kind == ScalarMatchKind::Bool {
                    match name.name.as_str() {
                        "true" => Some(HirMatchPattern::BoolLiteral(true)),
                        "false" => Some(HirMatchPattern::BoolLiteral(false)),
                        _ => {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "bool match arms must be true, false, or _",
                                    name.span,
                                )
                                .with_id(DiagnosticId::TypeMatchPatternUnsupported),
                            );
                            None
                        }
                    }
                } else if kind == ScalarMatchKind::Char {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "char match arms must be char literals or _",
                            name.span,
                        )
                        .with_id(DiagnosticId::TypeMatchPatternUnsupported),
                    );
                    None
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "integer match arms must be integer literals or _",
                            name.span,
                        )
                        .with_id(DiagnosticId::TypeMatchPatternUnsupported),
                    );
                    None
                }
            }
        }
    }

    fn check_match_arm_result_type(
        &mut self,
        result_ty: &mut Option<TypeId>,
        body_ty: TypeId,
        span: Span,
    ) {
        if let Some(t) = *result_ty {
            if let Err(_) = self.ctx.unify(t, body_ty) {
                self.diagnostics.push(
                    Diagnostic::error(
                        alloc::format!(
                            "match arms have incompatible types: {} and {}",
                            self.ctx.type_to_string(t),
                            self.ctx.type_to_string(body_ty)
                        ),
                        span,
                    )
                    .with_id(DiagnosticId::TypeMatchArmsTypeMismatch),
                );
            }
        } else {
            *result_ty = Some(body_ty);
        }
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
        if params.is_empty() && args.len() == 1 && matches!(args[0].expr.kind, HirExprKind::Unit) {
            args.clear();
        }

        if matches!(self.current_effect, Effect::Pure) && matches!(effect, Effect::Impure) {
            self.diagnostics.push(
                Diagnostic::error("pure context cannot call impure function", func.expr.span)
                    .with_id(DiagnosticId::TypePureCallsImpureFunction),
            );
            return None;
        }

        // Assignment operators
        if let Some(assign) = func.assign {
            if args.len() != 1 {
                self.diagnostics.push(
                    Diagnostic::error("assignment expects one argument", func.expr.span)
                        .with_id(DiagnosticId::TypeAssignmentArityMismatch),
                );
                return None;
            }
            if let AssignKind::AddrOf(mutable) = assign {
                if args.len() != 1 {
                    return None;
                }
                if crate::log::is_verbose() {
                    typecheck_log!(
                        "apply_function: Reducing AddrOf, inner={:?}",
                        args[0].expr.kind
                    );
                }
                let inner_ty = args[0].ty;
                let res_ty = self.ctx.reference(inner_ty, mutable);
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
                if args.len() != 1 {
                    return None;
                }
                let arg_ty = self.ctx.resolve(args[0].ty);
                let inner_ty = match self.ctx.get(arg_ty) {
                    TypeKind::Reference(inner, _) => inner,
                    _ => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "cannot dereference non-reference type: {}",
                                    self.ctx.type_to_string(arg_ty)
                                ),
                                args[0].expr.span,
                            )
                            .with_id(DiagnosticId::TypeInvalidDeref),
                        );
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
                    self.diagnostics.push(
                        Diagnostic::error("type mismatch in assignment", func.expr.span)
                            .with_id(DiagnosticId::TypeAssignmentTypeMismatch),
                    );
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
                            self.diagnostics.push(
                                Diagnostic::error("cannot set undefined variable", func.expr.span)
                                    .with_id(DiagnosticId::TypeUndefinedVariable),
                            );
                        }
                        if !b_mut {
                            self.diagnostics.push(
                                Diagnostic::error("variable is not mutable", func.expr.span)
                                    .with_id(DiagnosticId::TypeImmutableMutation),
                            );
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
                self.diagnostics.push(
                    Diagnostic::error(
                        format!("undefined variable for assignment: {}", name),
                        func.expr.span,
                    )
                    .with_id(DiagnosticId::TypeAssignmentUndefinedVariable),
                );
                return None;
            }
        }

        // Special-cased symbols (if / while)
        match &func.expr.kind {
            HirExprKind::Var(name) if name == "if" => {
                if args.len() != 3 {
                    self.diagnostics.push(
                        Diagnostic::error("if expects three arguments", func.expr.span)
                            .with_id(DiagnosticId::TypeIfArityMismatch),
                    );
                    return None;
                }
                if self.ctx.unify(args[0].ty, self.ctx.bool()).is_err() {
                    self.diagnostics.push(
                        Diagnostic::error("if condition must be bool", args[0].expr.span)
                            .with_id(DiagnosticId::TypeIfConditionTypeMismatch),
                    );
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
                    self.diagnostics.push(
                        Diagnostic::error("while expects two arguments", func.expr.span)
                            .with_id(DiagnosticId::TypeWhileArityMismatch),
                    );
                    return None;
                }
                if self.ctx.unify(args[0].ty, self.ctx.bool()).is_err() {
                    self.diagnostics.push(
                        Diagnostic::error("while condition must be bool", args[0].expr.span)
                            .with_id(DiagnosticId::TypeWhileConditionTypeMismatch),
                    );
                }
                if self.ctx.unify(args[1].ty, self.ctx.unit()).is_err() {
                    self.diagnostics.push(
                        Diagnostic::error("while body must be unit", args[1].expr.span)
                            .with_id(DiagnosticId::TypeWhileBodyTypeMismatch),
                    );
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

        // General call or let/set
        if let HirExprKind::Var(name) | HirExprKind::FnValue(name) = &func.expr.kind {
            if crate::log::is_verbose() && name.contains("Result") {
                typecheck_log!(
                    "apply_function debug: callee={} type={} args=[{}] explicit_type_args=[{}]",
                    name,
                    self.ctx.type_to_string(func.ty),
                    args.iter()
                        .map(|arg| self.ctx.type_to_string(arg.ty))
                        .collect::<Vec<_>>()
                        .join(", "),
                    type_args
                        .iter()
                        .map(|ty| self.ctx.type_to_string(*ty))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            let symbol_resolved = matches!(&func.expr.kind, HirExprKind::FnValue(_));
            let qualified_call = if symbol_resolved {
                None
            } else {
                self.lookup_qualified_bindings(&Ident {
                    name: name.clone(),
                    span: func.expr.span,
                })
            };
            let bindings = if symbol_resolved {
                self.env.lookup_all_callables_by_symbol(name)
            } else if let Some((_, qualified)) = &qualified_call {
                qualified.iter().collect()
            } else {
                self.env.lookup_all_callables(name)
            };
            let has_function_value_binding = if symbol_resolved {
                false
            } else if qualified_call.is_some() {
                false
            } else {
                self.env
                    .lookup_value(name)
                    .map(|b| {
                        let rty = self.ctx.resolve_id(b.ty);
                        matches!(self.ctx.get(rty), TypeKind::Function { .. })
                    })
                    .unwrap_or(false)
            };
            if !bindings.is_empty() && !has_function_value_binding {
                {
                    let explicit_type_args = type_args.clone();
                    let use_expected = expected_ret.is_some() && bindings.len() > 1;
                    if crate::log::is_verbose() && use_expected {
                        if let Some(expected) = expected_ret {
                            typecheck_log!(
                                "overload debug: '{}' using expected_ret={}",
                                name,
                                self.ctx.type_to_string(expected)
                            );
                        }
                    }
                    #[derive(Clone, Copy)]
                    struct OverloadCandidate<'b> {
                        binding: &'b Binding,
                        type_param_count: usize,
                        instantiated_specificity: usize,
                        declared_specificity: usize,
                        field_accessor: Option<FieldAccessorKind>,
                    }

                    let mut candidates: Vec<OverloadCandidate<'_>> = Vec::new();
                    let mut mismatch_count = false;
                    for binding in &bindings {
                        if crate::log::is_verbose() {
                            typecheck_log!(
                                "overload debug: consider '{}' candidate {}",
                                name,
                                function_signature_string(self.ctx, binding.ty)
                            );
                        }
                        let capture_len = match &binding.kind {
                            BindingKind::Func { captures, .. } => captures.len(),
                            _ => 0,
                        };
                        let checkpoint = self.ctx.checkpoint();
                        let inst_ty = if !explicit_type_args.is_empty() {
                            let func_data = if let TypeKind::Function {
                                type_params,
                                params,
                                result,
                                effect,
                            } = self.ctx.get(binding.ty)
                            {
                                Some((type_params, params, result, effect))
                            } else {
                                None
                            };
                            let Some((type_params, params, result, effect)) = func_data else {
                                if crate::log::is_verbose() {
                                    typecheck_log!(
                                        "overload debug: skip '{}' candidate {} reason=not_function_after_type_args",
                                        name,
                                        function_signature_string(self.ctx, binding.ty)
                                    );
                                }
                                self.ctx.rollback(checkpoint);
                                continue;
                            };
                            if type_params.len() != explicit_type_args.len() {
                                mismatch_count = true;
                                self.ctx.rollback(checkpoint);
                                continue;
                            }
                            let mut mapping = BTreeMap::new();
                            for (p, a) in type_params.iter().zip(explicit_type_args.iter()) {
                                insert_substitution_mapping(self.ctx, &mut mapping, *p, *a);
                            }
                            let substituted_params = params
                                .iter()
                                .map(|p| self.ctx.substitute(*p, &mapping))
                                .collect::<Vec<_>>();
                            let substituted_result = self.ctx.substitute(result, &mapping);
                            self.ctx.function(
                                Vec::new(),
                                substituted_params,
                                substituted_result,
                                effect,
                            )
                        } else {
                            let (inst_ty, _args, _mapping) = self.ctx.instantiate(binding.ty);
                            inst_ty
                        };

                        let func_ty = self.ctx.get(inst_ty);
                        let (c_params, c_result, _c_effect) = match func_ty {
                            TypeKind::Function {
                                params,
                                result,
                                effect,
                                ..
                            } => (params, result, effect),
                            _ => {
                                if crate::log::is_verbose() {
                                    typecheck_log!(
                                        "overload debug: skip '{}' candidate {} reason=not_function_instantiated",
                                        name,
                                        function_signature_string(self.ctx, binding.ty)
                                    );
                                }
                                self.ctx.rollback(checkpoint);
                                continue;
                            }
                        };
                        if c_params.len() < capture_len {
                            if crate::log::is_verbose() {
                                typecheck_log!(
                                    "overload debug: skip '{}' candidate {} reason=capture_len params={} capture={}",
                                    name,
                                    function_signature_string(self.ctx, binding.ty),
                                    c_params.len(),
                                    capture_len
                                );
                            }
                            self.ctx.rollback(checkpoint);
                            continue;
                        }
                        let user_params = &c_params[capture_len..];
                        if user_params.len() != args.len() {
                            if crate::log::is_verbose() {
                                typecheck_log!(
                                    "overload debug: skip '{}' candidate {} reason=arity user_params={} args={}",
                                    name,
                                    function_signature_string(self.ctx, binding.ty),
                                    user_params.len(),
                                    args.len()
                                );
                            }
                            self.ctx.rollback(checkpoint);
                            continue;
                        }
                        let mut ok = true;
                        for (arg, pty) in args.iter().zip(user_params.iter()) {
                            if !self.char_literal_matches_context(arg, *pty)
                                && self.ctx.unify(arg.ty, *pty).is_err()
                            {
                                if crate::log::is_verbose() {
                                    typecheck_log!(
                                        "overload debug: skip '{}' candidate {} reason=unify arg={} param={}",
                                        name,
                                        function_signature_string(self.ctx, binding.ty),
                                        self.ctx.type_to_string(arg.ty),
                                        self.ctx.type_to_string(*pty)
                                    );
                                }
                                ok = false;
                                break;
                            }
                        }
                        if ok && use_expected {
                            if let Some(expected) = expected_ret {
                                if self.ctx.unify(c_result, expected).is_err() {
                                    if crate::log::is_verbose() {
                                        typecheck_log!(
                                        "overload debug: skip '{}' candidate {} reason=expected_ret result={} expected={}",
                                        name,
                                        function_signature_string(self.ctx, binding.ty),
                                        self.ctx.type_to_string(c_result),
                                        self.ctx.type_to_string(expected)
                                    );
                                    }
                                    ok = false;
                                }
                            }
                        }
                        if ok {
                            if crate::log::is_verbose() {
                                typecheck_log!(
                                    "overload debug: accept '{}' candidate {}",
                                    name,
                                    function_signature_string(self.ctx, binding.ty)
                                );
                            }
                            let type_param_count =
                                match self.ctx.get(self.ctx.resolve_id(binding.ty)) {
                                    TypeKind::Function { type_params, .. } => type_params.len(),
                                    _ => 0,
                                };
                            let instantiated_specificity =
                                function_user_param_specificity(self.ctx, inst_ty, args.len());
                            let declared_specificity =
                                function_user_param_specificity(self.ctx, binding.ty, args.len());
                            candidates.push(OverloadCandidate {
                                binding,
                                type_param_count,
                                instantiated_specificity,
                                declared_specificity,
                                field_accessor: match &binding.kind {
                                    BindingKind::Func { field_accessor, .. } => *field_accessor,
                                    _ => None,
                                },
                            });
                        }
                        self.ctx.rollback(checkpoint);
                    }

                    // In a pure context, if both pure and impure candidates match,
                    // prefer pure ones to avoid false D3025 from name collisions
                    // between different modules' overloads of the same function.
                    if candidates.len() > 1 && matches!(self.current_effect, Effect::Pure) {
                        let pure_only: Vec<OverloadCandidate<'_>> = candidates
                            .iter()
                            .filter(|c| {
                                matches!(
                                    self.ctx.get(c.binding.ty),
                                    TypeKind::Function {
                                        effect: Effect::Pure,
                                        ..
                                    }
                                )
                            })
                            .cloned()
                            .collect();
                        if !pure_only.is_empty() {
                            candidates = pure_only;
                        }
                    }

                    if candidates.is_empty() {
                        if crate::log::is_verbose() {
                            let arg_tys = args
                                .iter()
                                .map(|a| self.ctx.type_to_string(a.ty))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let all = bindings
                                .iter()
                                .map(|b| {
                                    format!(
                                        "{}:{}",
                                        b.name,
                                        function_signature_string(self.ctx, b.ty)
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(" | ");
                            typecheck_log!(
                                "overload debug: no candidate for '{}' args=[{}] candidates=[{}]",
                                name,
                                arg_tys,
                                all
                            );
                        }
                        if mismatch_count {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "type arguments do not match any overload",
                                    func.expr.span,
                                )
                                .with_id(DiagnosticId::TypeOverloadTypeArgsMismatch),
                            );
                        } else {
                            self.diagnostics.push(
                                Diagnostic::error("no matching overload found", func.expr.span)
                                    .with_id(DiagnosticId::TypeNoMatchingOverload),
                            );
                        }
                        return None;
                    }
                    if candidates.len() > 1 {
                        let mut sig_seen: BTreeSet<String> = BTreeSet::new();
                        let mut dedup: Vec<OverloadCandidate<'_>> = Vec::new();
                        for c in candidates {
                            let sig = function_signature_string(self.ctx, c.binding.ty);
                            if sig_seen.insert(sig) {
                                dedup.push(c);
                            }
                        }
                        candidates = dedup;
                    }
                    if candidates.len() > 1 {
                        let ordinary: Vec<OverloadCandidate<'_>> = candidates
                            .iter()
                            .filter(|b| b.field_accessor.is_none())
                            .cloned()
                            .collect();
                        if !ordinary.is_empty() {
                            candidates = ordinary;
                        }
                    }
                    if candidates.len() > 1 {
                        let concrete: Vec<OverloadCandidate<'_>> = candidates
                            .iter()
                            .filter(|b| !type_contains_unbound_var(self.ctx, b.binding.ty))
                            .cloned()
                            .collect();
                        if !concrete.is_empty() {
                            candidates = concrete;
                        }
                    }
                    if candidates.len() > 1 {
                        let min_type_params = candidates
                            .iter()
                            .map(|b| b.type_param_count)
                            .min()
                            .unwrap_or(0);
                        let narrowed: Vec<OverloadCandidate<'_>> = candidates
                            .into_iter()
                            .filter(|b| b.type_param_count == min_type_params)
                            .collect();
                        candidates = narrowed;
                    }
                    if candidates.len() > 1 {
                        if crate::log::is_verbose() {
                            for candidate in &candidates {
                                typecheck_log!(
                                    "overload debug: specificity '{}' candidate {} score={}",
                                    name,
                                    function_signature_string(self.ctx, candidate.binding.ty),
                                    candidate.instantiated_specificity
                                );
                            }
                        }
                        let max_specificity = candidates
                            .iter()
                            .map(|b| b.instantiated_specificity)
                            .max()
                            .unwrap_or(0);
                        let narrowed: Vec<OverloadCandidate<'_>> = candidates
                            .into_iter()
                            .filter(|b| b.instantiated_specificity == max_specificity)
                            .collect();
                        candidates = narrowed;
                    }
                    if candidates.len() > 1 {
                        let max_declared_specificity = candidates
                            .iter()
                            .map(|b| b.declared_specificity)
                            .max()
                            .unwrap_or(0);
                        let narrowed: Vec<OverloadCandidate<'_>> = candidates
                            .into_iter()
                            .filter(|b| b.declared_specificity == max_declared_specificity)
                            .collect();
                        candidates = narrowed;
                    }
                    if candidates.len() > 1 {
                        self.diagnostics.push(
                            Diagnostic::error("ambiguous overload", func.expr.span)
                                .with_id(DiagnosticId::TypeAmbiguousOverload),
                        );
                        return None;
                    }

                    let binding = candidates[0].binding;
                    let selected_field_accessor = match &binding.kind {
                        BindingKind::Func { field_accessor, .. } => *field_accessor,
                        _ => None,
                    };
                    let (selected_symbol, selected_builtin) = match &binding.kind {
                        BindingKind::Func {
                            symbol, builtin, ..
                        } => (symbol.clone(), *builtin),
                        _ => (name.clone(), None),
                    };
                    let selected_def_id = match &binding.kind {
                        BindingKind::Func { def_id, .. } => *def_id,
                        _ => None,
                    };
                    let selected_type_snapshot = (!explicit_type_args.is_empty())
                        .then(|| self.ctx.snapshot_type_var_bindings(binding.ty));
                    let (inst_ty, mut resolved_args, type_arg_mapping) =
                        if !explicit_type_args.is_empty() {
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
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "type arguments do not match overload",
                                        func.expr.span,
                                    )
                                    .with_id(DiagnosticId::TypeOverloadTypeArgsMismatch),
                                );
                                return None;
                            }
                            let mut mapping = BTreeMap::new();
                            for (p, a) in type_params.iter().zip(explicit_type_args.iter()) {
                                insert_substitution_mapping(self.ctx, &mut mapping, *p, *a);
                            }
                            let substituted_params = params
                                .iter()
                                .map(|p| self.ctx.substitute(*p, &mapping))
                                .collect::<Vec<_>>();
                            let substituted_result = self.ctx.substitute(result, &mapping);
                            (
                                self.ctx.function(
                                    Vec::new(),
                                    substituted_params,
                                    substituted_result,
                                    effect,
                                ),
                                explicit_type_args.clone(),
                                mapping,
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
                        self.diagnostics.push(
                            Diagnostic::error("argument count mismatch", func.expr.span)
                                .with_id(DiagnosticId::TypeArgumentArityMismatch),
                        );
                        return None;
                    }
                    for (arg, param_ty) in args.iter_mut().zip(user_params.iter()) {
                        match self.char_literal_context_type(arg, *param_ty) {
                            Some(Ok(resolved)) => {
                                arg.ty = resolved;
                                arg.expr.ty = resolved;
                                continue;
                            }
                            Some(Err(())) => {
                                self.diagnostics.push(
                                    Diagnostic::error("argument type mismatch", arg.expr.span)
                                        .with_id(DiagnosticId::TypeArgumentTypeMismatch),
                                );
                                continue;
                            }
                            None => {}
                        }
                        if self.ctx.unify(arg.ty, *param_ty).is_err() {
                            self.diagnostics.push(
                                Diagnostic::error("argument type mismatch", arg.expr.span)
                                    .with_id(DiagnosticId::TypeArgumentTypeMismatch),
                            );
                        }
                    }
                    if matches!(self.current_effect, Effect::Pure)
                        && matches!(c_effect, Effect::Impure)
                    {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "pure context cannot call impure function",
                                func.expr.span,
                            )
                            .with_id(DiagnosticId::TypePureCallsImpureFunction),
                        );
                        return None;
                    }

                    if explicit_type_args.is_empty() {
                        resolved_args = resolved_args
                            .into_iter()
                            .map(|t| self.ctx.resolve_id(t))
                            .collect();
                        if let TypeKind::Function { type_params, .. } = self.ctx.get(binding.ty) {
                            if type_params.len() == resolved_args.len() {
                                for (idx, tp) in type_params.iter().enumerate() {
                                    if let Some(inferred) = infer_instantiated_type_arg(
                                        self.ctx, binding.ty, inst_ty, *tp,
                                    ) {
                                        resolved_args[idx] = self.ctx.resolve_id(inferred);
                                    }
                                }
                            }
                        }
                    }

                    if let Some(snapshot) = &selected_type_snapshot {
                        self.ctx.restore_type_var_bindings(snapshot);
                    }

                    if let BindingKind::Func {
                        type_param_bounds, ..
                    } = &binding.kind
                    {
                        if !type_param_bounds.is_empty() {
                            for (tp, bounds) in type_param_bounds.iter() {
                                let Some(raw_arg) = type_arg_mapping.get(tp) else {
                                    continue;
                                };
                                let resolved_arg = self.ctx.resolve_id(*raw_arg);
                                for b in bounds {
                                    let substituted_trait_args = b
                                        .trait_args
                                        .iter()
                                        .map(|arg| self.ctx.substitute(*arg, &type_arg_mapping))
                                        .collect::<Vec<_>>();
                                    let substituted_bound = TraitBoundRef {
                                        name: format_trait_ref_name(
                                            &b.trait_base_name,
                                            &substituted_trait_args,
                                            self.ctx,
                                        ),
                                        trait_base_name: b.trait_base_name.clone(),
                                        trait_args: substituted_trait_args,
                                        trait_self_ty: self
                                            .ctx
                                            .substitute(b.trait_self_ty, &type_arg_mapping),
                                    };
                                    if crate::log::is_verbose() {
                                        typecheck_log!(
                                            "trait-bound debug: callee='{}' tp={} raw_arg={} resolved_arg={} bound={} current_bounds={}",
                                            name,
                                            self.ctx.type_to_string(*tp),
                                            self.ctx.type_to_string(*raw_arg),
                                            self.ctx.type_to_string(resolved_arg),
                                            substituted_bound.name,
                                            self.type_param_bounds
                                                .iter()
                                                .map(|(bound_tp, bs)| {
                                                    format!(
                                                        "{}:[{}]",
                                                        self.ctx.type_to_string(*bound_tp),
                                                        bs.iter()
                                                            .map(|bb| bb.name.clone())
                                                            .collect::<Vec<_>>()
                                                            .join("|")
                                                    )
                                                })
                                                .collect::<Vec<_>>()
                                                .join(", ")
                                        );
                                    }
                                    if self
                                        .trait_bound_satisfied_by_ref(&substituted_bound, *raw_arg)
                                        || self.trait_bound_satisfied_by_ref(
                                            &substituted_bound,
                                            resolved_arg,
                                        )
                                    {
                                        continue;
                                    }
                                    let inferred_arg = infer_instantiated_type_arg(
                                        self.ctx, binding.ty, inst_ty, *tp,
                                    )
                                    .unwrap_or(resolved_arg);
                                    if self.trait_bound_satisfied_by_ref(
                                        &substituted_bound,
                                        inferred_arg,
                                    ) {
                                        continue;
                                    }
                                    if self.is_concrete_type(inferred_arg) {
                                        self.diagnostics.push(
                                            Diagnostic::error(
                                                format!(
                                                    "type does not satisfy trait bound '{}'",
                                                    substituted_bound.name
                                                ),
                                                func.expr.span,
                                            )
                                            .with_id(DiagnosticId::TypeTraitBoundUnsatisfied),
                                        );
                                    } else {
                                        self.pending_trait_bound_checks.push((
                                            substituted_bound,
                                            inferred_arg,
                                            func.expr.span,
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    if let Some(field_accessor) = selected_field_accessor {
                        if args.len() >= 2 {
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
                                let access_base_ty = if field_accessor == FieldAccessorKind::GetRef
                                {
                                    let resolved_obj_ty = self.ctx.resolve(obj.ty);
                                    match self.ctx.get(resolved_obj_ty) {
                                        TypeKind::Reference(inner, _) => inner,
                                        _ => {
                                            self.diagnostics.push(
                                                Diagnostic::error(
                                                    "get_ref expects a reference to a composite value",
                                                    obj.span,
                                                )
                                                .with_id(DiagnosticId::TypeInvalidFieldAccess),
                                            );
                                            self.ctx.never()
                                        }
                                    }
                                } else {
                                    obj.ty
                                };
                                if let Some((f_ty, offset)) = self.resolve_field_access_with_mode(
                                    access_base_ty,
                                    f_idx,
                                    func.expr.span,
                                    false,
                                ) {
                                    if field_accessor == FieldAccessorKind::Get && args.len() == 2 {
                                        let addr_expr = if let Some(raw_addr) =
                                            raw_aggregate_load_addr_expr(&obj, &self.ctx)
                                        {
                                            add_i32_offset_expr(
                                                raw_addr,
                                                offset,
                                                idx.span,
                                                func.expr.span,
                                                self.ctx.i32(),
                                            )
                                        } else {
                                            add_i32_offset_expr(
                                                obj,
                                                offset,
                                                idx.span,
                                                func.expr.span,
                                                self.ctx.i32(),
                                            )
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
                                    } else if field_accessor == FieldAccessorKind::GetRef
                                        && args.len() == 2
                                    {
                                        let ref_ty = self.ctx.reference(f_ty, false);
                                        let addr_expr = if offset == 0 {
                                            HirExpr {
                                                ty: ref_ty,
                                                kind: obj.kind,
                                                span: func.expr.span,
                                            }
                                        } else {
                                            HirExpr {
                                                ty: ref_ty,
                                                kind: HirExprKind::Intrinsic {
                                                    name: "add".to_string(),
                                                    type_args: vec![self.ctx.i32()],
                                                    args: vec![
                                                        obj,
                                                        HirExpr {
                                                            ty: self.ctx.i32(),
                                                            kind: HirExprKind::LiteralI32(
                                                                offset as i32,
                                                            ),
                                                            span: idx.span,
                                                        },
                                                    ],
                                                },
                                                span: func.expr.span,
                                            }
                                        };
                                        return Some(StackEntry {
                                            ty: ref_ty,
                                            expr: addr_expr,
                                            type_args: Vec::new(),
                                            assign: None,
                                            auto_call: true,
                                        });
                                    } else if field_accessor == FieldAccessorKind::Put
                                        && args.len() == 3
                                    {
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
                                                            kind: HirExprKind::LiteralI32(
                                                                offset as i32,
                                                            ),
                                                            span: idx.span,
                                                        },
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

                    // Enum constructors
                    if let Some((enm, var)) = parse_variant_name(name) {
                        if let Some(info) = self.enums.get(enm) {
                            if let Some(_vinfo) = info.variants.iter().find(|v| v.name == var) {
                                if crate::log::is_verbose() && enm == "Result" && var == "Ok" {
                                    typecheck_log!(
                                        "enum ctor debug: name={} resolved_args=[{}] user_params=[{}] arg_tys=[{}] c_result={}",
                                        name,
                                        resolved_args
                                            .iter()
                                            .map(|ty| self.ctx.type_to_string(*ty))
                                            .collect::<Vec<_>>()
                                            .join(", "),
                                        user_params
                                            .iter()
                                            .map(|ty| self.ctx.type_to_string(*ty))
                                            .collect::<Vec<_>>()
                                            .join(", "),
                                        args.iter()
                                            .map(|arg| self.ctx.type_to_string(arg.ty))
                                            .collect::<Vec<_>>()
                                            .join(", "),
                                        self.ctx.type_to_string(c_result)
                                    );
                                }
                                if c_params.len() == 1 && args.len() != 1 {
                                    self.diagnostics.push(
                                        Diagnostic::error(
                                            "constructor expects one argument",
                                            func.expr.span,
                                        )
                                        .with_id(DiagnosticId::TypeArgumentArityMismatch),
                                    );
                                    return None;
                                }
                                if c_params.is_empty() && !args.is_empty() {
                                    self.diagnostics.push(
                                        Diagnostic::error(
                                            "constructor takes no arguments",
                                            func.expr.span,
                                        )
                                        .with_id(DiagnosticId::TypeArgumentArityMismatch),
                                    );
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
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "struct constructor arity mismatch",
                                    func.expr.span,
                                )
                                .with_id(DiagnosticId::TypeArgumentArityMismatch),
                            );
                            return None;
                        }
                        let is_tag_unit_struct = s.fields.len() == 1
                            && s.field_names.len() == 1
                            && s.field_names[0] == "tag"
                            && matches!(
                                self.ctx.get(self.ctx.resolve_id(s.fields[0])),
                                TypeKind::Unit
                            );
                        let applied_ty = if resolved_args.is_empty() {
                            s.ty
                        } else {
                            self.ctx.apply(s.ty, resolved_args.clone())
                        };
                        let fields = if is_tag_unit_struct && args.is_empty() {
                            vec![HirExpr {
                                ty: self.ctx.unit(),
                                kind: HirExprKind::Unit,
                                span: func.expr.span,
                            }]
                        } else {
                            args.into_iter().map(|a| a.expr).collect()
                        };
                        return Some(StackEntry {
                            ty: applied_ty,
                            expr: HirExpr {
                                ty: applied_ty,
                                kind: HirExprKind::StructConstruct {
                                    name: name.clone(),
                                    type_args: resolved_args.clone(),
                                    fields,
                                },
                                span: func.expr.span,
                            },
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: true,
                        });
                    }

                    let mut trait_callee: Option<FuncRef> = None;
                    if let Some((trait_name, method_name)) = parse_variant_name(name) {
                        if let Some(trait_info) = self.traits.get(trait_name) {
                            if let Some(sig) = trait_info.methods.get(method_name) {
                                let applied_trait_args = self.infer_trait_application_args(
                                    trait_info,
                                    *sig,
                                    &args,
                                    expected_ret,
                                );
                                let mut inferred_self_ty = None;
                                if let (Some(self_hint), Some(first_param), Some(arg)) = (
                                    type_args.first().copied(),
                                    user_params.first().copied(),
                                    args.first(),
                                ) {
                                    if self.ctx.same_type(first_param, self_hint) {
                                        let candidate = self.ctx.resolve_id(arg.ty);
                                        let candidate_ok = self.type_param_has_bound_ref(
                                            candidate,
                                            trait_name,
                                            &applied_trait_args,
                                        ) || self.impls.iter().any(|imp| {
                                            imp.trait_base_name.as_deref() == Some(trait_name)
                                                && imp.trait_args.len() == applied_trait_args.len()
                                                && trait_application_matches(
                                                    self.ctx,
                                                    trait_name,
                                                    &applied_trait_args,
                                                    trait_name,
                                                    &imp.trait_args,
                                                )
                                                && self
                                                    .ctx
                                                    .type_pattern_matches(imp.target_ty, candidate)
                                        });
                                        if candidate_ok {
                                            inferred_self_ty = Some(candidate);
                                        }
                                    }
                                }
                                if inferred_self_ty.is_none() {
                                    if let Some(self_hint) = type_args.first().copied() {
                                        if let Some(expected) = expected_ret {
                                            let _ = self.ctx.unify(result, expected);
                                        }
                                        let resolved_hint = self.ctx.resolve_id(self_hint);
                                        inferred_self_ty = self
                                            .infer_unique_type_param_for_trait_ref(
                                                trait_name,
                                                &applied_trait_args,
                                            )
                                            .or_else(|| {
                                                if self.type_param_has_bound_ref(
                                                    resolved_hint,
                                                    trait_name,
                                                    &applied_trait_args,
                                                ) {
                                                    Some(resolved_hint)
                                                } else {
                                                    None
                                                }
                                            })
                                            .or(Some(resolved_hint));
                                    }
                                }
                                if inferred_self_ty.is_none() {
                                    if let Some(first) = args.first() {
                                        inferred_self_ty = Some(self.ctx.resolve_id(first.ty));
                                    }
                                }
                                if let Some(self_ty) = inferred_self_ty {
                                    trait_callee = Some(FuncRef::Trait {
                                        trait_name: trait_name.to_string(),
                                        trait_args: applied_trait_args,
                                        method: method_name.to_string(),
                                        self_ty,
                                    });
                                }
                            }
                        }
                    }
                    let callee = if selected_builtin.is_some() {
                        FuncRef::Builtin(selected_symbol.clone())
                    } else if let Some(tc) = trait_callee {
                        tc
                    } else {
                        if !resolved_args.is_empty()
                            && resolved_args
                                .iter()
                                .all(|t| !type_contains_unbound_var(self.ctx, *t))
                        {
                            self.instantiations
                                .entry(selected_symbol.clone())
                                .or_insert_with(Vec::new)
                                .push(resolved_args.clone());
                        }
                        FuncRef::User(
                            selected_symbol.clone(),
                            resolved_args.clone(),
                            selected_def_id,
                        )
                    };
                    let mut final_args: Vec<HirExpr> = Vec::new();
                    for (cap_name, cap_ty) in captures.iter() {
                        let resolved_cap_ty = self
                            .env
                            .lookup_value(cap_name)
                            .map(|b| self.ctx.resolve_id(b.ty))
                            .unwrap_or(*cap_ty);
                        final_args.push(HirExpr {
                            ty: resolved_cap_ty,
                            kind: HirExprKind::Var(cap_name.clone()),
                            span: func.expr.span,
                        });
                    }
                    for (arg, param_ty) in args.into_iter().zip(user_params.iter()) {
                        let arg_ty = arg.ty;
                        let mut arg_expr = arg.expr;
                        if let HirExprKind::Var(var_name) = &arg_expr.kind {
                            if self.env.lookup_value(var_name).is_none() {
                                let callables = self.env.lookup_all_callables(var_name);
                                if !callables.is_empty() {
                                    let mut matched_symbol: Option<String> = None;
                                    let mut ambiguous = false;
                                    for cb in callables {
                                        let (symbol, captures_len) = match &cb.kind {
                                            BindingKind::Func {
                                                symbol, captures, ..
                                            } => (symbol.clone(), captures.len()),
                                            _ => continue,
                                        };
                                        if captures_len != 0 {
                                            continue;
                                        }
                                        let checkpoint = self.ctx.checkpoint();
                                        let (cand_ty, _fresh, _mapping) =
                                            self.ctx.instantiate(cb.ty);
                                        let matched = self.ctx.unify(cand_ty, *param_ty).is_ok();
                                        self.ctx.rollback(checkpoint);
                                        if matched {
                                            if matched_symbol.is_some() {
                                                ambiguous = true;
                                                break;
                                            }
                                            matched_symbol = Some(symbol);
                                        }
                                    }
                                    if ambiguous {
                                        self.diagnostics.push(
                                            Diagnostic::error("ambiguous overload", arg_expr.span)
                                                .with_id(DiagnosticId::TypeAmbiguousOverload),
                                        );
                                        return None;
                                    }
                                    if let Some(symbol) = matched_symbol {
                                        arg_expr = HirExpr {
                                            ty: arg_ty,
                                            kind: HirExprKind::FnValue(symbol),
                                            span: arg_expr.span,
                                        };
                                    }
                                }
                            }
                        }
                        final_args.push(arg_expr);
                    }
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
            if self.env.lookup_all_callables(name).is_empty() {
                if let Some((trait_name, method_name)) = parse_variant_name(name) {
                    if let Some(trait_info) = self.traits.get(trait_name) {
                        if let Some(sig) = trait_info.methods.get(method_name) {
                            let applied_trait_name = self.infer_trait_application_name(
                                trait_name,
                                trait_info,
                                *sig,
                                &args,
                                expected_ret,
                            );
                            let applied_trait_args = self.infer_trait_application_args(
                                trait_info,
                                *sig,
                                &args,
                                expected_ret,
                            );
                            let mut inferred_self_ty = None;
                            if let (Some(self_hint), Some(first_param), Some(arg)) = (
                                type_args.first().copied(),
                                params.first().copied(),
                                args.first(),
                            ) {
                                if self.ctx.same_type(first_param, self_hint) {
                                    let candidate = self.ctx.resolve_id(arg.ty);
                                    let candidate_ok = self.type_param_has_bound_ref(
                                        candidate,
                                        trait_name,
                                        &applied_trait_args,
                                    ) || self.impls.iter().any(|imp| {
                                        imp.trait_base_name.as_deref() == Some(trait_name)
                                            && imp.trait_args.len() == applied_trait_args.len()
                                            && trait_application_matches(
                                                self.ctx,
                                                trait_name,
                                                &applied_trait_args,
                                                trait_name,
                                                &imp.trait_args,
                                            )
                                            && self
                                                .ctx
                                                .type_pattern_matches(imp.target_ty, candidate)
                                    });
                                    if candidate_ok {
                                        inferred_self_ty = Some(candidate);
                                    }
                                }
                            }
                            if inferred_self_ty.is_none() {
                                if let Some(self_hint) = type_args.first().copied() {
                                    if let Some(expected) = expected_ret {
                                        let _ = self.ctx.unify(result, expected);
                                    }
                                    let resolved_hint = self.ctx.resolve_id(self_hint);
                                    inferred_self_ty = self
                                        .infer_unique_type_param_for_trait_ref(
                                            trait_name,
                                            &applied_trait_args,
                                        )
                                        .or_else(|| {
                                            if self.type_param_has_bound_ref(
                                                resolved_hint,
                                                trait_name,
                                                &applied_trait_args,
                                            ) {
                                                Some(resolved_hint)
                                            } else {
                                                None
                                            }
                                        })
                                        .or(Some(resolved_hint));
                                }
                            }
                            let Some(self_ty) = inferred_self_ty else {
                                self.diagnostics.push(Diagnostic::error(
                                    "trait method call requires receiver argument or expected self type",
                                    func.expr.span,
                                ).with_id(DiagnosticId::TypeTraitBoundUnsatisfied));
                                return None;
                            };
                            let trait_ok = self.type_param_has_bound_ref(
                                self_ty,
                                trait_name,
                                &applied_trait_args,
                            ) || self.impls.iter().any(|imp| {
                                imp.trait_base_name.as_deref() == Some(trait_name)
                                    && imp.trait_args.len() == applied_trait_args.len()
                                    && trait_application_matches(
                                        self.ctx,
                                        trait_name,
                                        &applied_trait_args,
                                        trait_name,
                                        &imp.trait_args,
                                    )
                                    && self.ctx.type_pattern_matches(imp.target_ty, self_ty)
                            });
                            if !trait_ok {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "type does not satisfy trait bound '{}'",
                                            applied_trait_name
                                        ),
                                        func.expr.span,
                                    )
                                    .with_id(DiagnosticId::TypeTraitBoundUnsatisfied),
                                );
                                return None;
                            }
                            if matches!(self.current_effect, Effect::Pure)
                                && matches!(effect, Effect::Impure)
                            {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "pure context cannot call impure function",
                                        func.expr.span,
                                    )
                                    .with_id(DiagnosticId::TypePureCallsImpureFunction),
                                );
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
                                            trait_args: applied_trait_args,
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
                    self.diagnostics.push(
                        Diagnostic::error("variable is not callable", func.expr.span)
                            .with_id(DiagnosticId::TypeVariableNotCallable),
                    );
                    return None;
                }
            }
        }

        // Fallback: function value call (`call_indirect` in wasm backend)
        // This path is limited to actual function-typed values (including explicit `@fn`).
        let allow_indirect = match &func.expr.kind {
            HirExprKind::FnValue(name) => {
                let has_capture = self
                    .env
                    .lookup_all_callables_by_symbol(name)
                    .iter()
                    .any(|b| matches!(&b.kind, BindingKind::Func { captures, .. } if !captures.is_empty()));
                if has_capture {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "capturing function cannot be used as a function value yet",
                            func.expr.span,
                        )
                        .with_id(DiagnosticId::TypeCapturingFunctionValueUnsupported),
                    );
                    false
                } else {
                    true
                }
            }
            HirExprKind::Var(name) => {
                if !matches!(self.ctx.get(func.ty), TypeKind::Function { .. }) {
                    false
                } else {
                    let has_capture = self
                        .env
                        .lookup_all_callables(name)
                        .iter()
                        .any(|b| matches!(&b.kind, BindingKind::Func { captures, .. } if !captures.is_empty()));
                    if has_capture {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "capturing function cannot be passed as a function value yet",
                                func.expr.span,
                            )
                            .with_id(DiagnosticId::TypeCapturingFunctionValueUnsupported),
                        );
                        false
                    } else {
                        true
                    }
                }
            }
            _ => matches!(self.ctx.get(func.ty), TypeKind::Function { .. }),
        };
        if !allow_indirect {
            self.diagnostics.push(
                Diagnostic::error("indirect call requires a function value", func.expr.span)
                    .with_id(DiagnosticId::TypeIndirectCallRequiresFunctionValue),
            );
            return None;
        }

        let resolved_params: Vec<TypeId> = args.iter().map(|a| self.ctx.resolve_id(a.ty)).collect();
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
    span: Span,
    kind: BindingKind,
}

#[derive(Debug, Clone)]
enum BindingKind {
    Var,
    Func {
        def_id: Option<DefId>,
        symbol: String,
        effect: Effect,
        arity: usize,
        builtin: Option<BuiltinKind>,
        field_accessor: Option<FieldAccessorKind>,
        type_param_bounds: BTreeMap<TypeId, Vec<TraitBoundRef>>,
        captures: Vec<(String, TypeId)>,
    },
}

fn resolve_type_ids_in_function(ctx: &TypeCtx, function: &mut HirFunction) {
    function.func_ty = ctx.resolve_id(function.func_ty);
    function.result = ctx.resolve_id(function.result);
    for param in &mut function.params {
        param.ty = ctx.resolve_id(param.ty);
    }
    match &mut function.body {
        HirBody::Block(block) => resolve_type_ids_in_block(ctx, block),
        HirBody::Wasm(_) | HirBody::LlvmIr(_) => {}
    }
}

fn resolve_type_ids_in_block(ctx: &TypeCtx, block: &mut HirBlock) {
    block.ty = ctx.resolve_id(block.ty);
    for line in &mut block.lines {
        resolve_type_ids_in_expr(ctx, &mut line.expr);
    }
}

fn resolve_type_ids_in_expr(ctx: &TypeCtx, expr: &mut HirExpr) {
    let mut pending = Vec::new();
    pending.push(expr);
    while let Some(expr) = pending.pop() {
        expr.ty = ctx.resolve_id(expr.ty);
        match &mut expr.kind {
            HirExprKind::Call { callee, args } => {
                match callee {
                    FuncRef::User(_, type_args, _) => {
                        for ty in type_args {
                            *ty = ctx.resolve_id(*ty);
                        }
                    }
                    FuncRef::Trait {
                        trait_args,
                        self_ty,
                        ..
                    } => {
                        for ty in trait_args {
                            *ty = ctx.resolve_id(*ty);
                        }
                        *self_ty = ctx.resolve_id(*self_ty);
                    }
                    FuncRef::Builtin(_) => {}
                }
                for arg in args {
                    pending.push(arg);
                }
            }
            HirExprKind::CallIndirect {
                callee,
                params,
                result,
                args,
            } => {
                pending.push(callee);
                for ty in params {
                    *ty = ctx.resolve_id(*ty);
                }
                *result = ctx.resolve_id(*result);
                for arg in args {
                    pending.push(arg);
                }
            }
            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                pending.push(cond);
                pending.push(then_branch);
                pending.push(else_branch);
            }
            HirExprKind::While { cond, body } => {
                pending.push(cond);
                pending.push(body);
            }
            HirExprKind::Match { scrutinee, arms } => {
                pending.push(scrutinee);
                for arm in arms {
                    pending.push(&mut arm.body);
                }
            }
            HirExprKind::Block(block) => {
                block.ty = ctx.resolve_id(block.ty);
                for line in &mut block.lines {
                    pending.push(&mut line.expr);
                }
            }
            HirExprKind::Let { value, .. }
            | HirExprKind::Set { value, .. }
            | HirExprKind::AddrOf(value)
            | HirExprKind::Deref(value) => pending.push(value),
            HirExprKind::TupleConstruct { items } | HirExprKind::Intrinsic { args: items, .. } => {
                for item in items {
                    pending.push(item);
                }
            }
            HirExprKind::EnumConstruct {
                type_args, payload, ..
            } => {
                for ty in type_args {
                    *ty = ctx.resolve_id(*ty);
                }
                if let Some(payload) = payload {
                    pending.push(payload);
                }
            }
            HirExprKind::StructConstruct {
                type_args, fields, ..
            } => {
                for ty in type_args {
                    *ty = ctx.resolve_id(*ty);
                }
                for field in fields {
                    pending.push(field);
                }
            }
            HirExprKind::FnValue(_)
            | HirExprKind::Var(_)
            | HirExprKind::Unit
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Drop { .. } => {}
        }
    }
}

impl BindingKind {
    fn is_var(&self) -> bool {
        matches!(self, BindingKind::Var)
    }

    fn is_callable(&self) -> bool {
        matches!(self, BindingKind::Func { .. })
    }
}

#[derive(Debug, Default)]
struct Scope {
    values: Vec<Binding>,
    callables: Vec<Binding>,
}

#[derive(Debug)]
struct Env {
    scopes: Vec<Scope>,
}

impl Env {
    fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_into_scope(scope: &mut Scope, binding: Binding) {
        if binding.kind.is_var() {
            scope.values.push(binding);
        } else {
            scope.callables.push(binding);
        }
    }

    fn insert_global(&mut self, binding: Binding) {
        if let Some(scope) = self.scopes.first_mut() {
            Self::insert_into_scope(scope, binding);
        }
    }

    fn remove_duplicate_func(&mut self, name: &str, ty: TypeId, ctx: &TypeCtx) {
        if let Some(scope) = self.scopes.first_mut() {
            scope.callables.retain(|b| {
                if b.name != name || !b.kind.is_callable() {
                    return true;
                }
                !same_function_signature(ctx, b.ty, ty)
            });
        }
    }

    fn insert_local(&mut self, binding: Binding) -> Result<(), ()> {
        if let Some(scope) = self.scopes.last_mut() {
            let has_value = scope.values.iter().any(|b| b.name == binding.name);
            if binding.kind.is_var() {
                if has_value {
                    return Err(());
                }
                scope.values.push(binding);
            } else {
                if has_value {
                    return Err(());
                }
                scope.callables.push(binding);
            }
        }
        Ok(())
    }

    fn lookup_current_value(&self, name: &str) -> Option<&Binding> {
        self.scopes
            .last()
            .and_then(|scope| scope.values.iter().rev().find(|b| b.name == name))
    }

    fn lookup_any_defined(&self, name: &str) -> Option<&Binding> {
        // When resolving identifiers for reading, skip hoisted bindings
        // that are not yet defined. This prevents the RHS of a hoisted
        // `let` from accidentally seeing the placeholder binding.
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope
                .values
                .iter()
                .rev()
                .find(|b| b.name == name && b.defined)
                .or_else(|| {
                    scope
                        .callables
                        .iter()
                        .rev()
                        .find(|b| b.name == name && b.defined)
                })
            {
                return Some(b);
            }
        }
        None
    }

    fn lookup_all_any_defined(&self, name: &str) -> Vec<&Binding> {
        for scope in self.scopes.iter().rev() {
            let mut items: Vec<&Binding> = scope
                .values
                .iter()
                .filter(|b| b.name == name && b.defined)
                .collect();
            items.extend(
                scope
                    .callables
                    .iter()
                    .filter(|b| b.name == name && b.defined),
            );
            if !items.is_empty() {
                return items;
            }
        }
        Vec::new()
    }

    fn lookup_value(&self, name: &str) -> Option<&Binding> {
        self.lookup_all_any_defined(name)
            .into_iter()
            .find(|b| matches!(b.kind, BindingKind::Var))
    }

    fn lookup_value_with_scope(&self, name: &str) -> Option<(&Binding, usize)> {
        for idx in (0..self.scopes.len()).rev() {
            let scope = &self.scopes[idx];
            if let Some(b) = scope
                .values
                .iter()
                .rev()
                .find(|b| b.name == name && b.defined)
            {
                return Some((b, idx));
            }
        }
        None
    }

    fn lookup_all_callables(&self, name: &str) -> Vec<&Binding> {
        let mut items = Vec::new();
        for scope in self.scopes.iter().rev() {
            for b in scope
                .callables
                .iter()
                .filter(|b| b.name == name && b.defined)
            {
                items.push(b);
            }
        }
        items
    }

    fn lookup_all_callables_by_symbol(&self, symbol: &str) -> Vec<&Binding> {
        let mut items = Vec::new();
        for scope in self.scopes.iter().rev() {
            for b in scope.callables.iter().filter(|b| {
                b.defined
                    && matches!(
                        &b.kind,
                        BindingKind::Func { symbol: s, .. } if s == symbol
                    )
            }) {
                items.push(b);
            }
        }
        items
    }

    fn update_local_function_binding(
        &mut self,
        _ctx: &TypeCtx,
        name: &str,
        span: Span,
        ty: TypeId,
        captures_new: Vec<(String, TypeId)>,
    ) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            for binding in scope.callables.iter_mut().rev() {
                if binding.name != name || binding.span != span {
                    continue;
                }
                binding.ty = ty;
                if let BindingKind::Func { captures, .. } = &mut binding.kind {
                    *captures = captures_new.clone();
                }
                return true;
            }
        }
        false
    }

    /// 同名候補から型シグネチャ一致の関数シンボルを返す。
    ///
    /// typecheck 本体と HIR 生成で関数名決定ロジックを共有し、
    /// hoist した symbol と最終的な HIR 名の不整合を防ぐ。
    fn lookup_func_symbol(&self, name: &str, ty: TypeId, ctx: &TypeCtx) -> Option<String> {
        for binding in self.lookup_all_callables(name) {
            if let BindingKind::Func { symbol, .. } = &binding.kind {
                if same_function_signature(ctx, binding.ty, ty) {
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
            if let Some(binding) = scope
                .values
                .iter()
                .rev()
                .find(|b| b.name == name && b.defined)
                .or_else(|| {
                    scope
                        .callables
                        .iter()
                        .rev()
                        .find(|b| b.name == name && b.defined)
                })
            {
                return Some(binding);
            }
        }
        None
    }

    fn lookup_any(&self, name: &str) -> Option<&Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope
                .values
                .iter()
                .rev()
                .find(|b| b.name == name)
                .or_else(|| scope.callables.iter().rev().find(|b| b.name == name))
            {
                return Some(b);
            }
        }
        None
    }

    fn lookup_mut(&mut self, name: &str) -> Option<&mut Binding> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(pos) = scope.values.iter().rposition(|b| b.name == name) {
                return scope.values.get_mut(pos);
            }
        }
        None
    }
}

fn is_raw_memory_load_name(name: &str) -> bool {
    name == "load" || name.starts_with("load_")
}

fn raw_aggregate_load_addr_expr(expr: &HirExpr, ctx: &TypeCtx) -> Option<HirExpr> {
    if !is_aggregate_storage_type(ctx, expr.ty) {
        return None;
    }
    match &expr.kind {
        HirExprKind::Call { callee, args } if args.len() == 1 => match callee {
            FuncRef::User(name, _, _) | FuncRef::Builtin(name) if is_raw_memory_load_name(name) => {
                Some(args[0].clone())
            }
            _ => None,
        },
        HirExprKind::Intrinsic { name, args, .. } if name == "load" && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn add_i32_offset_expr(
    base: HirExpr,
    offset: usize,
    offset_span: Span,
    span: Span,
    i32_ty: TypeId,
) -> HirExpr {
    if offset == 0 {
        return base;
    }
    HirExpr {
        ty: i32_ty,
        kind: HirExprKind::Intrinsic {
            name: "add".to_string(),
            type_args: vec![i32_ty],
            args: vec![
                base,
                HirExpr {
                    ty: i32_ty,
                    kind: HirExprKind::LiteralI32(offset as i32),
                    span: offset_span,
                },
            ],
        },
        span,
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
        let message = format!("important symbol '{}' is shadowed by local {}", name, kind);
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
    env.lookup_any(name).and_then(|b| {
        if b.no_shadow && b.defined {
            Some(b)
        } else {
            None
        }
    })
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
    env.lookup_all_callables(name).into_iter().find(|b| {
        matches!(b.kind, BindingKind::Func { .. }) && same_function_signature(ctx, b.ty, ty)
    })
}

fn find_nonshadow_same_signature_func<'a>(
    env: &'a Env,
    name: &str,
    ty: TypeId,
    ctx: &TypeCtx,
) -> Option<&'a Binding> {
    env.lookup_all_callables(name).into_iter().find(|b| {
        b.no_shadow
            && b.defined
            && matches!(b.kind, BindingKind::Func { .. })
            && same_function_signature(ctx, b.ty, ty)
    })
}

fn find_invalid_same_file_overload<'a>(
    env: &'a Env,
    name: &str,
    arity: usize,
    span: Span,
) -> Option<&'a Binding> {
    if span.file_id.0 != 0 {
        return None;
    }
    env.lookup_all_callables(name).into_iter().find(|b| {
        if b.span.file_id != span.file_id {
            return false;
        }
        let BindingKind::Func {
            arity: existing_arity,
            ..
        } = b.kind
        else {
            return false;
        };
        existing_arity != arity
    })
}

fn type_shape_specificity(ctx: &TypeCtx, ty: TypeId) -> usize {
    match ctx.get(ctx.resolve_id(ty)) {
        TypeKind::Var(_) => 0,
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Named(_) => 2,
        TypeKind::Enum { .. } | TypeKind::Struct { .. } => 3,
        TypeKind::Apply { base, args } => {
            4 + type_shape_specificity(ctx, base)
                + args
                    .iter()
                    .map(|arg| type_shape_specificity(ctx, *arg))
                    .sum::<usize>()
        }
        TypeKind::Tuple { items } => {
            1 + items
                .iter()
                .map(|item| type_shape_specificity(ctx, *item))
                .sum::<usize>()
        }
        TypeKind::Function { params, result, .. } => {
            1 + params
                .iter()
                .map(|param| type_shape_specificity(ctx, *param))
                .sum::<usize>()
                + type_shape_specificity(ctx, result)
        }
        TypeKind::Box(inner) | TypeKind::Reference(inner, _) => {
            1 + type_shape_specificity(ctx, inner)
        }
    }
}

fn function_user_param_specificity(ctx: &TypeCtx, ty: TypeId, user_arity: usize) -> usize {
    match ctx.get(ctx.resolve_id(ty)) {
        TypeKind::Function { params, result, .. } => {
            let capture_len = params.len().saturating_sub(user_arity);
            let user_params = &params[capture_len..];
            user_params
                .iter()
                .map(|param| type_shape_specificity(ctx, *param))
                .sum::<usize>()
                + type_shape_specificity(ctx, result)
        }
        _ => 0,
    }
}

fn detect_field_accessor_fn(def: &FnDef) -> Option<FieldAccessorKind> {
    let FnBody::Parsed(block) = &def.body else {
        return None;
    };
    if block.items.len() != 1 {
        return None;
    }
    let expr = match &block.items[0] {
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => expr,
        _ => return None,
    };
    match expr.items.as_slice() {
        [PrefixItem::Intrinsic(intrin, _)] if intrin.name == "get_field" => {
            Some(FieldAccessorKind::Get)
        }
        [PrefixItem::Intrinsic(intrin, _)] if intrin.name == "get_field_ref" => {
            Some(FieldAccessorKind::GetRef)
        }
        [PrefixItem::Intrinsic(intrin, _)] if intrin.name == "set_field" => {
            Some(FieldAccessorKind::Put)
        }
        _ => None,
    }
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
    match t.as_unspanned() {
        TypeExpr::Unit => ctx.unit(),
        TypeExpr::I32 => ctx.i32(),
        TypeExpr::U8 => ctx.u8(),
        TypeExpr::F32 => ctx.f32(),
        TypeExpr::Bool => ctx.bool(),
        TypeExpr::Char => ctx.char(),
        TypeExpr::Str => ctx.str(),
        TypeExpr::Never => ctx.never(),
        TypeExpr::Named(name) => match name.as_str() {
            "i32" => ctx.i32(),
            "u8" => ctx.u8(),
            "f32" => ctx.f32(),
            "bool" => ctx.bool(),
            "char" => ctx.char(),
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
        },
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
        TypeExpr::Spanned(inner, _) => type_from_expr(ctx, labels, inner),
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
            type_params,
            params,
            result,
            effect,
        } => {
            let mut generics = BTreeMap::new();
            for (i, tp) in type_params.iter().enumerate() {
                let mut name = String::from("$T");
                name.push_str(&i.to_string());
                generics.insert(ctx.resolve_id(*tp), name);
            }
            let mut s = String::from("func");
            if !type_params.is_empty() {
                s.push_str("_gen_");
                s.push_str(&type_params.len().to_string());
            }
            s.push_str("__");
            if params.is_empty() {
                s.push_str("unit");
            } else {
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&signature_type_string(ctx, *p, &generics));
                }
            }
            s.push_str("__");
            s.push_str(&signature_type_string(ctx, result, &generics));
            match effect {
                Effect::Pure => s.push_str("__pure"),
                Effect::Impure => s.push_str("__imp"),
            }
            s
        }
        _ => ctx.type_to_string(resolved),
    }
}

fn same_function_signature(ctx: &TypeCtx, a: TypeId, b: TypeId) -> bool {
    let ra = ctx.resolve_id(a);
    let rb = ctx.resolve_id(b);
    let (tpa, pa, resa, ea) = match ctx.get(ra) {
        TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        } => (type_params, params, result, effect),
        _ => return ctx.same_type(ra, rb),
    };
    let (tpb, pb, resb, eb) = match ctx.get(rb) {
        TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        } => (type_params, params, result, effect),
        _ => return false,
    };
    if ea != eb || tpa.len() != tpb.len() || pa.len() != pb.len() {
        return false;
    }
    let mut map_ab: BTreeMap<TypeId, TypeId> = BTreeMap::new();
    let mut map_ba: BTreeMap<TypeId, TypeId> = BTreeMap::new();
    for (ta, tb) in tpa.iter().zip(tpb.iter()) {
        map_ab.insert(ctx.resolve_id(*ta), ctx.resolve_id(*tb));
        map_ba.insert(ctx.resolve_id(*tb), ctx.resolve_id(*ta));
    }
    let mut seen = BTreeSet::new();
    for (ta, tb) in pa.iter().zip(pb.iter()) {
        if !same_type_with_signature_generics(ctx, *ta, *tb, &map_ab, &map_ba, &mut seen) {
            return false;
        }
    }
    same_type_with_signature_generics(ctx, resa, resb, &map_ab, &map_ba, &mut seen)
}

fn same_type_with_signature_generics(
    ctx: &TypeCtx,
    a: TypeId,
    b: TypeId,
    map_ab: &BTreeMap<TypeId, TypeId>,
    map_ba: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<(TypeId, TypeId)>,
) -> bool {
    let ra = ctx.resolve_id(a);
    let rb = ctx.resolve_id(b);
    if ra == rb {
        return true;
    }
    if let Some(mapped) = map_ab.get(&ra) {
        return *mapped == rb;
    }
    if let Some(mapped) = map_ba.get(&rb) {
        return *mapped == ra;
    }
    let key = if ra <= rb { (ra, rb) } else { (rb, ra) };
    if !seen.insert(key) {
        return true;
    }
    let result = match (ctx.get(ra), ctx.get(rb)) {
        (TypeKind::Unit, TypeKind::Unit)
        | (TypeKind::I32, TypeKind::I32)
        | (TypeKind::U8, TypeKind::U8)
        | (TypeKind::F32, TypeKind::F32)
        | (TypeKind::Bool, TypeKind::Bool)
        | (TypeKind::Char, TypeKind::Char)
        | (TypeKind::Str, TypeKind::Str)
        | (TypeKind::Never, TypeKind::Never) => true,
        (TypeKind::Named(na), TypeKind::Named(nb)) => na == nb,
        (TypeKind::Box(ia), TypeKind::Box(ib)) => {
            same_type_with_signature_generics(ctx, ia, ib, map_ab, map_ba, seen)
        }
        (TypeKind::Reference(ia, ma), TypeKind::Reference(ib, mb)) => {
            ma == mb && same_type_with_signature_generics(ctx, ia, ib, map_ab, map_ba, seen)
        }
        (TypeKind::Tuple { items: ia }, TypeKind::Tuple { items: ib }) => {
            ia.len() == ib.len()
                && ia.iter().zip(ib.iter()).all(|(ta, tb)| {
                    same_type_with_signature_generics(ctx, *ta, *tb, map_ab, map_ba, seen)
                })
        }
        (TypeKind::Apply { base: ba, args: aa }, TypeKind::Apply { base: bb, args: ab }) => {
            aa.len() == ab.len()
                && same_type_with_signature_generics(ctx, ba, bb, map_ab, map_ba, seen)
                && aa.iter().zip(ab.iter()).all(|(ta, tb)| {
                    same_type_with_signature_generics(ctx, *ta, *tb, map_ab, map_ba, seen)
                })
        }
        (
            TypeKind::Function {
                type_params: tpa,
                params: pa,
                result: resa,
                effect: ea,
            },
            TypeKind::Function {
                type_params: tpb,
                params: pb,
                result: resb,
                effect: eb,
            },
        ) => {
            if ea != eb || tpa.len() != tpb.len() || pa.len() != pb.len() {
                false
            } else {
                let mut nested_ab = map_ab.clone();
                let mut nested_ba = map_ba.clone();
                for (ta, tb) in tpa.iter().zip(tpb.iter()) {
                    nested_ab.insert(ctx.resolve_id(*ta), ctx.resolve_id(*tb));
                    nested_ba.insert(ctx.resolve_id(*tb), ctx.resolve_id(*ta));
                }
                pa.iter().zip(pb.iter()).all(|(ta, tb)| {
                    same_type_with_signature_generics(ctx, *ta, *tb, &nested_ab, &nested_ba, seen)
                }) && same_type_with_signature_generics(
                    ctx, resa, resb, &nested_ab, &nested_ba, seen,
                )
            }
        }
        (TypeKind::Var(va), TypeKind::Var(vb)) => match (va.binding, vb.binding) {
            (Some(ba), Some(bb)) => {
                same_type_with_signature_generics(ctx, ba, bb, map_ab, map_ba, seen)
            }
            (Some(ba), None) => {
                same_type_with_signature_generics(ctx, ba, rb, map_ab, map_ba, seen)
            }
            (None, Some(bb)) => {
                same_type_with_signature_generics(ctx, ra, bb, map_ab, map_ba, seen)
            }
            (None, None) => va.label == vb.label,
        },
        (TypeKind::Var(va), _) => va
            .binding
            .map(|ba| same_type_with_signature_generics(ctx, ba, rb, map_ab, map_ba, seen))
            .unwrap_or(false),
        (_, TypeKind::Var(vb)) => vb
            .binding
            .map(|bb| same_type_with_signature_generics(ctx, ra, bb, map_ab, map_ba, seen))
            .unwrap_or(false),
        (
            TypeKind::Struct {
                name: na,
                type_params: tpa,
                fields: fa,
                field_names: fna,
                ..
            },
            TypeKind::Struct {
                name: nb,
                type_params: tpb,
                fields: fb,
                field_names: fnb,
                ..
            },
        ) => {
            na == nb
                && fna == fnb
                && tpa.len() == tpb.len()
                && fa.len() == fb.len()
                && tpa.iter().zip(tpb.iter()).all(|(ta, tb)| {
                    same_type_with_signature_generics(ctx, *ta, *tb, map_ab, map_ba, seen)
                })
                && fa.iter().zip(fb.iter()).all(|(ta, tb)| {
                    same_type_with_signature_generics(ctx, *ta, *tb, map_ab, map_ba, seen)
                })
        }
        (
            TypeKind::Enum {
                name: na,
                type_params: tpa,
                variants: va,
                ..
            },
            TypeKind::Enum {
                name: nb,
                type_params: tpb,
                variants: vb,
                ..
            },
        ) => {
            na == nb
                && tpa.len() == tpb.len()
                && va.len() == vb.len()
                && tpa.iter().zip(tpb.iter()).all(|(ta, tb)| {
                    same_type_with_signature_generics(ctx, *ta, *tb, map_ab, map_ba, seen)
                })
                && va.iter().zip(vb.iter()).all(|(a, b)| {
                    a.name == b.name
                        && match (a.payload, b.payload) {
                            (Some(pa), Some(pb)) => {
                                same_type_with_signature_generics(ctx, pa, pb, map_ab, map_ba, seen)
                            }
                            (None, None) => true,
                            _ => false,
                        }
                })
        }
        _ => false,
    };
    seen.remove(&key);
    result
}

fn signature_type_string(ctx: &TypeCtx, ty: TypeId, generics: &BTreeMap<TypeId, String>) -> String {
    let resolved = ctx.resolve_id(ty);
    if let Some(name) = generics.get(&resolved) {
        return name.clone();
    }
    match ctx.get(resolved) {
        TypeKind::Unit => String::from("unit"),
        TypeKind::I32 => String::from("i32"),
        TypeKind::U8 => String::from("u8"),
        TypeKind::F32 => String::from("f32"),
        TypeKind::Bool => String::from("bool"),
        TypeKind::Char => String::from("char"),
        TypeKind::Str => String::from("str"),
        TypeKind::Never => String::from("never"),
        TypeKind::Named(name) => name,
        TypeKind::Var(tv) => {
            if let Some(binding) = tv.binding {
                signature_type_string(ctx, binding, generics)
            } else {
                tv.label.unwrap_or_else(|| format!("var_{}", resolved.0))
            }
        }
        TypeKind::Tuple { items } => {
            let mut s = String::from("tuple_");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    s.push('_');
                }
                s.push_str(&signature_type_string(ctx, *item, generics));
            }
            s
        }
        TypeKind::Apply { base, args } => {
            let mut s = signature_type_string(ctx, base, generics);
            s.push('_');
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    s.push('_');
                }
                s.push_str(&signature_type_string(ctx, *arg, generics));
            }
            s
        }
        TypeKind::Box(inner) => {
            let mut s = String::from("box_");
            s.push_str(&signature_type_string(ctx, inner, generics));
            s
        }
        TypeKind::Reference(inner, is_mut) => {
            let mut s = String::from("ref_");
            if is_mut {
                s.push_str("mut_");
            }
            s.push_str(&signature_type_string(ctx, inner, generics));
            s
        }
        TypeKind::Function {
            params,
            result,
            effect,
            ..
        } => {
            let mut s = String::from("fn__");
            for (i, p) in params.iter().enumerate() {
                if i > 0 {
                    s.push('_');
                }
                s.push_str(&signature_type_string(ctx, *p, generics));
            }
            s.push_str("__");
            s.push_str(&signature_type_string(ctx, result, generics));
            match effect {
                Effect::Pure => s.push_str("__pure"),
                Effect::Impure => s.push_str("__imp"),
            }
            s
        }
        TypeKind::Enum {
            name, type_params, ..
        } => {
            if type_params.is_empty() {
                name
            } else {
                let mut s = name;
                s.push('_');
                for (i, tp) in type_params.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&signature_type_string(ctx, *tp, generics));
                }
                s
            }
        }
        TypeKind::Struct {
            name, type_params, ..
        } => {
            if type_params.is_empty() {
                name
            } else {
                let mut s = name;
                s.push('_');
                for (i, tp) in type_params.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&signature_type_string(ctx, *tp, generics));
                }
                s
            }
        }
    }
}

fn contains_same_type(ctx: &TypeCtx, list: &[TypeId], ty: TypeId) -> bool {
    list.iter().any(|t| ctx.same_type(*t, ty))
}

fn push_unique_type(ctx: &TypeCtx, list: &mut Vec<TypeId>, ty: TypeId) {
    if !contains_same_type(ctx, list, ty) {
        list.push(ctx.resolve_id(ty));
    }
}

fn type_contains_unbound_var(ctx: &TypeCtx, ty: TypeId) -> bool {
    let ty = ctx.resolve_id(ty);
    match ctx.get(ty) {
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
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
            params.iter().any(|p| type_contains_unbound_var(ctx, *p))
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

fn gate_allows(d: &Directive, target: CompileTarget, active_profile: BuildProfile) -> Option<bool> {
    crate::target_gate::directive_gate_allows(d, target, active_profile)
}

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
