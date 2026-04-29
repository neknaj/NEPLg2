use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::TypeParam;
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::diagnostics::type_error;
use super::type_expr::{type_from_expr, LabelEnv};

#[derive(Debug, Clone)]
pub(super) struct TraitInfo {
    pub(super) doc: Option<String>,
    pub(super) type_params: Vec<TypeId>,
    pub(super) capabilities: Vec<TraitCapability>,
    pub(super) methods: BTreeMap<String, TypeId>,
    pub(super) self_ty: TypeId,
    pub(super) span: Span,
}

#[derive(Debug, Clone, Default)]
pub(super) struct TraitSemantics {
    pub(super) copy_traits: Vec<(String, TypeId)>,
    pub(super) clone_traits: Vec<(String, TypeId)>,
    pub(super) drop_traits: Vec<(String, TypeId)>,
}

impl TraitSemantics {
    pub(super) fn detect(traits: &BTreeMap<String, TraitInfo>) -> Self {
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

    pub(super) fn has_any_copy_capability(&self) -> bool {
        !self.copy_traits.is_empty()
    }

    pub(super) fn has_copy_capability(&self, trait_id: Option<TypeId>) -> bool {
        match trait_id {
            Some(actual) => self.copy_traits.iter().any(|(_, id)| *id == actual),
            None => false,
        }
    }

    pub(super) fn has_clone_capability(&self, trait_id: Option<TypeId>) -> bool {
        match trait_id {
            Some(actual) => self.clone_traits.iter().any(|(_, id)| *id == actual),
            None => false,
        }
    }

    pub(super) fn has_drop_capability(&self, trait_id: Option<TypeId>) -> bool {
        match trait_id {
            Some(actual) => self.drop_traits.iter().any(|(_, id)| *id == actual),
            None => false,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ImplInfo {
    pub(super) trait_name: Option<String>,
    pub(super) trait_base_name: Option<String>,
    pub(super) trait_args: Vec<TypeId>,
    pub(super) trait_self_ty: Option<TypeId>,
    pub(super) target_ty: TypeId,
}

#[derive(Debug, Clone)]
pub(super) struct TraitBoundRef {
    pub(super) name: String,
    pub(super) trait_base_name: String,
    pub(super) trait_args: Vec<TypeId>,
    pub(super) trait_self_ty: TypeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TraitCapability {
    Copy,
    Clone,
    Drop,
}

pub(super) fn collect_type_params(
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
                    diags.push(type_error(
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
                diags.push(type_error(
                    TypeDiagnosticCode::TraitBoundUnknown,
                    format!("unknown trait bound '{}'", b.name.name),
                    p.name.span,
                ));
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

pub(super) fn format_trait_ref_name(base: &str, args: &[TypeId], ctx: &TypeCtx) -> String {
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

pub(super) fn trait_application_matches(
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

pub(super) fn type_param_has_trait_bound(
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

pub(super) fn parse_trait_ref_name(name: &str, ctx: &TypeCtx) -> Option<(String, Vec<TypeId>)> {
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

pub(super) fn merge_inferred_instantiation(
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

pub(super) fn infer_type_param_from_instantiated_pair(
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

pub(super) fn infer_instantiated_type_arg(
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

pub(super) fn insert_substitution_mapping(
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
