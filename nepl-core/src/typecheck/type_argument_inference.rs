use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::traits::infer_type_param_from_instantiated_pair;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TypeArgumentConstraint {
    original: TypeId,
    actual: TypeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TypeArgumentConflict {
    type_param: TypeId,
    first: TypeId,
    second: TypeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TypeArgumentInference {
    NoEvidence,
    Unique(TypeId),
    Conflict(TypeArgumentConflict),
}

pub(super) struct TypeArgumentResolution {
    pub(super) resolved_args: Vec<TypeId>,
    pub(super) conflicts: Vec<TypeArgumentConflict>,
}

impl TypeArgumentConstraint {
    pub(super) fn new(original: TypeId, actual: TypeId) -> Self {
        Self { original, actual }
    }

    fn infer_for_type_param(
        self,
        ctx: &TypeCtx,
        target_tp: TypeId,
        target_label: Option<&str>,
    ) -> Option<TypeId> {
        infer_type_param_from_instantiated_pair(
            ctx,
            self.original,
            self.actual,
            target_tp,
            target_label,
        )
    }
}

impl TypeArgumentConflict {
    fn new(type_param: TypeId, first: TypeId, second: TypeId) -> Self {
        Self {
            type_param,
            first,
            second,
        }
    }

    pub(super) fn diagnostic_message(self, ctx: &TypeCtx) -> String {
        format!(
            "type parameter {} has inconsistent constraints ({} vs {})",
            ctx.type_to_string(self.type_param),
            ctx.type_to_string(self.first),
            ctx.type_to_string(self.second)
        )
    }
}

impl TypeArgumentInference {
    fn merge(self, ctx: &TypeCtx, candidate: Option<TypeId>, type_param: TypeId) -> Self {
        match (self, candidate) {
            (TypeArgumentInference::Conflict(conflict), _) => {
                TypeArgumentInference::Conflict(conflict)
            }
            (current, None) => current,
            (TypeArgumentInference::NoEvidence, Some(candidate)) => {
                TypeArgumentInference::Unique(ctx.resolve_id(candidate))
            }
            (TypeArgumentInference::Unique(current), Some(candidate)) => {
                let candidate = ctx.resolve_id(candidate);
                if ctx.same_type(current, candidate) {
                    TypeArgumentInference::Unique(ctx.resolve_id(current))
                } else {
                    TypeArgumentInference::Conflict(TypeArgumentConflict::new(
                        type_param, current, candidate,
                    ))
                }
            }
        }
    }
}

pub(super) fn resolve_type_arguments_from_constraints(
    ctx: &TypeCtx,
    type_params: &[TypeId],
    fallback_args: Vec<TypeId>,
    constraints: &[TypeArgumentConstraint],
) -> TypeArgumentResolution {
    let mut resolved_args = fallback_args
        .into_iter()
        .map(|ty| ctx.resolve_id(ty))
        .collect::<Vec<_>>();
    let mut conflicts = Vec::new();
    if type_params.len() != resolved_args.len() {
        return TypeArgumentResolution {
            resolved_args,
            conflicts,
        };
    }

    for (idx, tp) in type_params.iter().enumerate() {
        let label = match ctx.get(ctx.resolve_id(*tp)) {
            TypeKind::Var(v) => v.label.clone(),
            _ => None,
        };
        let mut inference = TypeArgumentInference::NoEvidence;
        for constraint in constraints.iter().copied() {
            inference = inference.merge(
                ctx,
                constraint.infer_for_type_param(ctx, *tp, label.as_deref()),
                *tp,
            );
        }
        match inference {
            TypeArgumentInference::NoEvidence => {}
            TypeArgumentInference::Unique(inferred) => {
                resolved_args[idx] = ctx.resolve_id(inferred);
            }
            TypeArgumentInference::Conflict(conflict) => conflicts.push(conflict),
        }
    }

    TypeArgumentResolution {
        resolved_args,
        conflicts,
    }
}
