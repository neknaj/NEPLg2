use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::TypeParam;
use crate::ast::Visibility;
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::diagnostics::type_error;
use super::signature::signature_type_string;
use super::type_expr::{type_from_expr, LabelEnv};

#[derive(Debug, Clone)]
pub(super) struct TraitInfo {
    pub(super) doc: Option<String>,
    pub(super) visibility: Visibility,
    pub(super) type_params: Vec<TypeId>,
    pub(super) capabilities: Vec<TraitCapability>,
    pub(super) methods: BTreeMap<String, TypeId>,
    pub(super) self_ty: TypeId,
    pub(super) span: Span,
}

#[derive(Debug, Clone, Default)]
pub(super) struct TraitSemantics {
    pub(super) copy_traits: Vec<TypeId>,
    pub(super) clone_traits: Vec<TypeId>,
    pub(super) drop_traits: Vec<TypeId>,
}

impl TraitSemantics {
    pub(super) fn detect(traits: &BTreeMap<String, TraitInfo>) -> Self {
        let mut semantics = Self::default();

        for info in traits.values() {
            for cap in info.capabilities.iter().copied() {
                semantics.insert_trait(cap, info.self_ty);
            }
        }

        semantics
    }

    fn insert_trait(&mut self, capability: TraitCapability, trait_self_ty: TypeId) {
        let traits = match capability {
            TraitCapability::Copy => &mut self.copy_traits,
            TraitCapability::Clone => &mut self.clone_traits,
            TraitCapability::Drop => &mut self.drop_traits,
        };
        if !traits.contains(&trait_self_ty) {
            traits.push(trait_self_ty);
        }
    }

    pub(super) fn has_any_copy_capability(&self) -> bool {
        !self.copy_traits.is_empty()
    }

    pub(super) fn has_copy_capability(&self, trait_id: Option<TypeId>) -> bool {
        match trait_id {
            Some(actual) => self.copy_traits.contains(&actual),
            None => false,
        }
    }

    pub(super) fn has_clone_capability(&self, trait_id: Option<TypeId>) -> bool {
        match trait_id {
            Some(actual) => self.clone_traits.contains(&actual),
            None => false,
        }
    }

    pub(super) fn has_drop_capability(&self, trait_id: Option<TypeId>) -> bool {
        match trait_id {
            Some(actual) => self.drop_traits.contains(&actual),
            None => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TraitId(String);

impl TraitId {
    pub(super) fn from_name(name: &str) -> Self {
        Self(String::from(name))
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub(super) struct TraitApplication {
    pub(super) trait_id: TraitId,
    pub(super) args: Vec<TypeId>,
}

impl TraitApplication {
    pub(super) fn display_name(&self, ctx: &TypeCtx) -> String {
        format_trait_ref_name(self.trait_id.as_str(), &self.args, ctx)
    }

    pub(super) fn matches_parts(&self, ctx: &TypeCtx, trait_id: &TraitId, args: &[TypeId]) -> bool {
        trait_application_matches(ctx, &self.trait_id, &self.args, trait_id, args)
    }
}

#[derive(Debug, Clone)]
pub(super) enum ImplKind {
    Inherent,
    Trait {
        application: TraitApplication,
        self_ty: TypeId,
    },
}

#[derive(Debug, Clone)]
pub(super) struct ImplInfo {
    /// impl header が導入する generic binder。
    ///
    /// `.neplmeta` の public impl surface は、この binder を保持してから target type や
    /// trait application 内の generic reference を depth/index へ対応付ける。名前だけから
    /// binder を推測すると、別 scope の同名 generic parameter を誤って materialize する危険がある。
    pub(super) type_params: Vec<TypeId>,
    /// impl header の generic parameter bounds。
    ///
    /// bounds は trait lookup と materializer preflight の authority になる。ここに保持せず
    /// structured surface 側で再構築すると、private trait bound や trait identity の欠落を
    /// fail-closed に検出できなくなる。
    pub(super) type_param_bounds: BoundEnv,
    pub(super) kind: ImplKind,
    pub(super) target_ty: TypeId,
}

impl ImplInfo {
    pub(super) fn trait_self_ty(&self) -> Option<TypeId> {
        match &self.kind {
            ImplKind::Inherent => None,
            ImplKind::Trait { self_ty, .. } => Some(*self_ty),
        }
    }

    pub(super) fn matches_trait_application(
        &self,
        ctx: &TypeCtx,
        trait_id: &TraitId,
        args: &[TypeId],
    ) -> bool {
        match &self.kind {
            ImplKind::Inherent => false,
            ImplKind::Trait { application, .. } => application.matches_parts(ctx, trait_id, args),
        }
    }

    pub(super) fn matches_same_trait_impl(
        &self,
        ctx: &TypeCtx,
        application: &TraitApplication,
        self_ty: TypeId,
    ) -> bool {
        match &self.kind {
            ImplKind::Inherent => false,
            ImplKind::Trait {
                application: existing,
                self_ty: existing_self_ty,
            } => {
                *existing_self_ty == self_ty
                    && existing.matches_parts(ctx, &application.trait_id, &application.args)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct TraitBound {
    pub(super) application: TraitApplication,
    pub(super) trait_self_ty: TypeId,
}

impl TraitBound {
    pub(super) fn display_name(&self, ctx: &TypeCtx) -> String {
        self.application.display_name(ctx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TypeParamId(TypeId);

impl TypeParamId {
    pub(super) fn new(type_id: TypeId) -> Self {
        Self(type_id)
    }

    pub(super) fn type_id(self) -> TypeId {
        self.0
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct BoundEnv {
    bounds: BTreeMap<TypeParamId, Vec<TraitBound>>,
}

impl BoundEnv {
    pub(super) fn new() -> Self {
        Self {
            bounds: BTreeMap::new(),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.bounds.is_empty()
    }

    pub(super) fn insert(&mut self, type_param: TypeParamId, bounds: Vec<TraitBound>) {
        self.bounds.insert(type_param, bounds);
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (TypeParamId, &[TraitBound])> {
        self.bounds
            .iter()
            .map(|(type_param, bounds)| (*type_param, bounds.as_slice()))
    }

    pub(super) fn signature_equivalent(
        &self,
        ctx: &TypeCtx,
        type_params: &[TypeId],
        other: &Self,
        other_type_params: &[TypeId],
    ) -> bool {
        self.normalized_signature_bounds(ctx, type_params)
            == other.normalized_signature_bounds(ctx, other_type_params)
    }

    fn normalized_signature_bounds(&self, ctx: &TypeCtx, type_params: &[TypeId]) -> Vec<String> {
        let generics = signature_generic_names(ctx, type_params);
        let mut normalized = Vec::new();
        for (type_param, bounds) in self.iter() {
            let type_param_name = signature_type_string(ctx, type_param.type_id(), &generics);
            for bound in bounds {
                let args = bound
                    .application
                    .args
                    .iter()
                    .map(|arg| signature_type_string(ctx, *arg, &generics))
                    .collect::<Vec<_>>()
                    .join(",");
                let trait_self = signature_type_string(ctx, bound.trait_self_ty, &generics);
                normalized.push(format!(
                    "{}:{}<{}>:{}",
                    type_param_name,
                    bound.application.trait_id.as_str(),
                    args,
                    trait_self
                ));
            }
        }
        normalized.sort();
        normalized
    }

    pub(super) fn has_trait_application_bound(
        &self,
        ctx: &TypeCtx,
        ty: TypeId,
        trait_id: &TraitId,
        trait_args: &[TypeId],
    ) -> bool {
        let matches_bound = |b: &TraitBound| b.application.matches_parts(ctx, trait_id, trait_args);
        let resolved = ctx.resolve_id(ty);
        if let Some(bounds) = self.bounds.get(&TypeParamId::new(resolved)) {
            return bounds.iter().any(matches_bound);
        }
        self.bounds.iter().any(|(tp, bounds)| {
            ctx.resolve_id(tp.type_id()) == resolved && bounds.iter().any(matches_bound)
        })
    }
}

fn signature_generic_names(ctx: &TypeCtx, type_params: &[TypeId]) -> BTreeMap<TypeId, String> {
    let mut generics = BTreeMap::new();
    for (index, type_param) in type_params.iter().enumerate() {
        generics.insert(ctx.resolve_id(*type_param), format!("$T{index}"));
    }
    generics
}

#[derive(Debug, Clone)]
pub(super) struct PendingTraitCheck {
    pub(super) bound: TraitBound,
    pub(super) target_ty: TypeId,
    pub(super) span: Span,
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
) -> (Vec<TypeId>, Vec<Vec<TraitBound>>, BoundEnv) {
    let mut tps = Vec::new();
    let mut bounds_vec = Vec::new();
    let mut bounds_map = BoundEnv::new();
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
                bounds.push(TraitBound {
                    application: TraitApplication {
                        trait_id: TraitId::from_name(&b.name.name),
                        args: arg_tys,
                    },
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
        ctx.set_var_capabilities(id, copy_cap, clone_cap || copy_cap, drop_cap);
        if !bounds.is_empty() {
            bounds_map.insert(TypeParamId::new(id), bounds.clone());
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
    trait_id: &TraitId,
    args: &[TypeId],
    other_trait_id: &TraitId,
    other_args: &[TypeId],
) -> bool {
    if trait_id != other_trait_id || args.len() != other_args.len() {
        return false;
    }
    args.iter().zip(other_args.iter()).all(|(lhs, rhs)| {
        let lhs = ctx.resolve_id(*lhs);
        let rhs = ctx.resolve_id(*rhs);
        ctx.type_pattern_matches(lhs, rhs) || ctx.type_pattern_matches(rhs, lhs)
    })
}

pub(super) fn type_param_has_trait_application_bound(
    ctx: &TypeCtx,
    type_param_bounds: &BoundEnv,
    ty: TypeId,
    trait_id: &TraitId,
    trait_args: &[TypeId],
) -> bool {
    type_param_bounds.has_trait_application_bound(ctx, ty, trait_id, trait_args)
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
